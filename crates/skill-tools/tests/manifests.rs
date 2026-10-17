//! Asserts the plugin manifests agree with each other and with the workspace.

use serde_json::Value;
use skill_tools::{
    COMMANDS, FRONTMATTER_BUDGET, MANIFESTS, Result, SENTINEL, SKILL_FILES, VERSION,
    VERSIONED_MANIFESTS, plugin_dir, read_json, read_skill_file, repo_root, skill_dir,
};

fn manifest(path: &str) -> Result<Value> {
    read_json(&repo_root()?.join(path))
}

fn field<'a>(value: &'a Value, key: &str, path: &str) -> Result<&'a Value> {
    value
        .get(key)
        .ok_or_else(|| format!("{path}: missing `{key}`").into())
}

fn text(value: &Value, key: &str, path: &str) -> Result<String> {
    field(value, key, path)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{path}: `{key}` is not a string").into())
}

fn claude_plugin() -> Result<Value> {
    manifest("plugins/anodized/.claude-plugin/plugin.json")
}

fn codex_plugin() -> Result<Value> {
    manifest("plugins/anodized/.codex-plugin/plugin.json")
}

fn claude_entry() -> Result<Value> {
    let marketplace = manifest(".claude-plugin/marketplace.json")?;
    field(&marketplace, "plugins", "marketplace")?
        .get(0)
        .cloned()
        .ok_or_else(|| "marketplace lists no plugins".into())
}

#[test]
fn all_manifests_are_valid_json() -> Result<()> {
    for path in MANIFESTS {
        let value = manifest(path)?;
        assert!(value.is_object(), "{path}: not a JSON object");
    }
    Ok(())
}

#[test]
fn all_manifests_share_the_workspace_version() -> Result<()> {
    assert_eq!(
        text(&claude_entry()?, "version", "marketplace entry")?,
        VERSION
    );
    for path in VERSIONED_MANIFESTS.iter().skip(1) {
        assert_eq!(text(&manifest(path)?, "version", path)?, VERSION, "{path}");
    }
    Ok(())
}

#[test]
fn all_manifests_share_a_description_prefix() -> Result<()> {
    let reference = text(&claude_plugin()?, "description", "claude plugin")?;
    let prefix: String = reference.chars().take(40).collect();
    for description in [
        text(&codex_plugin()?, "description", "codex plugin")?,
        text(&claude_entry()?, "description", "marketplace entry")?,
    ] {
        assert!(
            description.starts_with(&prefix),
            "descriptions diverge: {description}"
        );
    }
    Ok(())
}

#[test]
fn marketplace_sources_point_at_the_plugin_directory() -> Result<()> {
    assert_eq!(
        text(&claude_entry()?, "source", "marketplace entry")?,
        "./plugins/anodized"
    );

    let codex = manifest(".agents/plugins/marketplace.json")?;
    let entry = field(&codex, "plugins", "codex marketplace")?
        .get(0)
        .cloned()
        .ok_or("codex marketplace lists no plugins")?;
    let source = field(&entry, "source", "codex entry")?.clone();
    assert_eq!(text(&source, "path", "codex source")?, "./plugins/anodized");
    assert!(plugin_dir()?.is_dir());
    Ok(())
}

#[test]
fn claude_plugin_lists_the_skill_directory() -> Result<()> {
    let plugin = claude_plugin()?;
    let skills = field(&plugin, "skills", "claude plugin")?;
    assert_eq!(skills, &serde_json::json!(["./skills/anodized"]));
    assert!(skill_dir()?.is_dir());
    Ok(())
}

#[test]
fn codex_plugin_lists_the_skills_parent() -> Result<()> {
    assert_eq!(
        text(&codex_plugin()?, "skills", "codex plugin")?,
        "./skills/"
    );
    Ok(())
}

#[test]
fn claude_plugin_lists_every_command_file() -> Result<()> {
    let plugin = claude_plugin()?;
    let listed: Vec<String> = field(&plugin, "commands", "claude plugin")?
        .as_array()
        .ok_or("`commands` is not an array")?
        .iter()
        .filter_map(|value| value.as_str().map(str::to_owned))
        .collect();

    for command in COMMANDS {
        let entry = format!("./commands/{command}");
        assert!(listed.contains(&entry), "`{entry}` is not listed");
        assert!(plugin_dir()?.join("commands").join(command).is_file());
    }

    let on_disk = std::fs::read_dir(plugin_dir()?.join("commands"))?.count();
    assert_eq!(on_disk, listed.len(), "a command file is not listed");
    Ok(())
}

#[test]
fn every_skill_file_exists_and_nothing_else_does() -> Result<()> {
    for name in SKILL_FILES {
        let contents = read_skill_file(name)?;
        assert!(!contents.trim().is_empty(), "{name} is empty");
    }
    for entry in std::fs::read_dir(skill_dir()?)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        assert!(
            SKILL_FILES.contains(&name.as_str()),
            "unlisted file: {name}"
        );
    }
    Ok(())
}

#[test]
fn license_and_repository_match_the_workspace() -> Result<()> {
    for value in [claude_plugin()?, codex_plugin()?, claude_entry()?] {
        assert_eq!(text(&value, "license", "manifest")?, "MIT OR Apache-2.0");
        assert_eq!(
            text(&value, "repository", "manifest")?,
            "https://github.com/anodized-rs/anodized"
        );
    }
    Ok(())
}

#[test]
fn skill_frontmatter_is_well_formed() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    assert!(skill.starts_with("---\n"), "SKILL.md has no frontmatter");
    let frontmatter = frontmatter(&skill)?;
    assert!(frontmatter.contains("name: anodized"));
    for key in ["description:", "when_to_use:", "allowed-tools:"] {
        assert!(frontmatter.contains(key), "frontmatter lacks `{key}`");
    }
    Ok(())
}

#[test]
fn frontmatter_fits_the_listing_budget() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    let frontmatter = frontmatter(&skill)?;
    let described = folded(&frontmatter, "description:")? + &folded(&frontmatter, "when_to_use:")?;
    assert!(
        described.len() <= FRONTMATTER_BUDGET,
        "description plus when_to_use is {} characters, over the {FRONTMATTER_BUDGET} the \
         listing keeps; triggers past the cut are silently discarded",
        described.len()
    );
    Ok(())
}

#[test]
fn skill_carries_the_ownership_sentinel() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    let body = skill
        .split_once("\n---\n")
        .ok_or("SKILL.md frontmatter is unterminated")?
        .1;
    assert_eq!(body.trim_start().lines().next(), Some(SENTINEL));
    Ok(())
}

#[test]
fn sibling_files_do_not_carry_frontmatter() -> Result<()> {
    for name in SKILL_FILES.iter().filter(|name| **name != "SKILL.md") {
        let contents = read_skill_file(name)?;
        assert!(
            !contents.starts_with("---"),
            "{name} opens with frontmatter, which reads as a standalone skill"
        );
    }
    Ok(())
}

fn frontmatter(skill: &str) -> Result<String> {
    let rest = skill.strip_prefix("---\n").ok_or("no frontmatter")?;
    let end = rest.find("\n---\n").ok_or("unterminated frontmatter")?;
    Ok(rest[..end].to_owned())
}

fn folded(frontmatter: &str, key: &str) -> Result<String> {
    let start = frontmatter
        .find(key)
        .ok_or_else(|| format!("frontmatter lacks `{key}`"))?;
    let mut value = String::new();
    for line in frontmatter[start..].lines().skip(1) {
        if !line.starts_with("  ") {
            break;
        }
        value.push_str(line.trim());
        value.push(' ');
    }
    Ok(value)
}
