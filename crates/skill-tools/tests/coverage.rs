//! Asserts the skill still documents what the crates actually do.

use skill_tools::{
    CFG_FLAGS, DIAGNOSTIC_SOURCES, DIAGNOSTICS, KeywordDoc, RUSTC_ERRORS, Result, all_keywords,
    documented_keyword, generate, read, read_skill_file, read_sources, repo_root,
};

#[test]
fn the_curated_order_matches_the_parser() -> Result<()> {
    // `Keyword` derives `Ord` over its declaration order, and the parser rejects fields that
    // are out of that order. The curated list drives the generated table, so if the two ever
    // disagree the table teaches an order the compiler refuses.
    let curated = all_keywords();
    let mut sorted = curated.clone();
    sorted.sort();
    assert_eq!(curated, sorted, "all_keywords() is out of parser order");

    let mut seen = std::collections::BTreeSet::new();
    for keyword in &curated {
        assert!(
            seen.insert(format!("{keyword:?}")),
            "duplicate: {keyword:?}"
        );
    }
    Ok(())
}

#[test]
fn every_current_keyword_is_documented() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    for keyword in all_keywords() {
        if let KeywordDoc::Current { name, .. } = documented_keyword(&keyword) {
            assert!(
                skill.contains(&format!("`{name}`")),
                "SKILL.md does not document `{name}`"
            );
        }
    }
    Ok(())
}

#[test]
fn every_removed_keyword_is_documented_as_removed() -> Result<()> {
    let diagnostics = read_skill_file("diagnostics.md")?;
    let migration = read_skill_file("migration.md")?;
    let reference = read_skill_file("reference.md")?;
    for keyword in all_keywords() {
        if let KeywordDoc::Removed { name, .. } = documented_keyword(&keyword) {
            assert!(
                migration.contains(name) && reference.contains(name),
                "`{name}` is removed but migration.md and reference.md do not both say so"
            );
        }
    }
    assert!(diagnostics.contains("no longer supported"));
    Ok(())
}

#[test]
fn every_qualifier_is_documented() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    for keyword in all_keywords() {
        if let KeywordDoc::Qualifier { name } = documented_keyword(&keyword) {
            assert!(
                skill.contains(&format!("`{name}`")),
                "SKILL.md does not document the `{name}` qualifier"
            );
        }
    }
    Ok(())
}

#[test]
fn every_cfg_flag_is_documented() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    for flag in CFG_FLAGS {
        assert!(skill.contains(flag), "SKILL.md does not document `{flag}`");
    }
    assert_eq!(
        generate::documented_cfg_flags(),
        generate::tracked_cfg_flags()
    );
    Ok(())
}

#[test]
fn cfg_flag_list_matches_the_crate_manifests() -> Result<()> {
    let root = repo_root()?;
    let mut declared = std::collections::BTreeSet::new();
    for manifest in [
        "crates/anodized/Cargo.toml",
        "crates/anodized-macros/Cargo.toml",
    ] {
        let text = read(&root.join(manifest))?;
        let mut rest = text.as_str();
        while let Some(start) = rest.find("cfg(anodized_") {
            rest = &rest[start + "cfg(".len()..];
            let end = rest.find(')').ok_or("unterminated cfg(")?;
            declared.insert(rest[..end].to_owned());
        }
    }
    let known: std::collections::BTreeSet<String> =
        CFG_FLAGS.iter().map(|flag| (*flag).to_owned()).collect();
    assert_eq!(declared, known, "check-cfg and CFG_FLAGS disagree");
    Ok(())
}

#[test]
fn every_macro_diagnostic_appears_in_the_docs() -> Result<()> {
    let diagnostics = read_skill_file("diagnostics.md")?;
    for message in DIAGNOSTICS {
        assert!(
            diagnostics.contains(message),
            "diagnostics.md does not document: {message}"
        );
    }
    assert_eq!(
        generate::documented_diagnostics(),
        generate::tracked_diagnostics()
    );
    Ok(())
}

#[test]
fn every_macro_diagnostic_still_exists_in_the_source() -> Result<()> {
    let sources = read_sources(DIAGNOSTIC_SOURCES)?;
    for message in DIAGNOSTICS {
        assert!(
            sources.contains(message),
            "no source emits this any more, so the documentation is now a lie: {message}"
        );
    }
    Ok(())
}

#[test]
fn indirect_rustc_errors_are_documented() -> Result<()> {
    let diagnostics = read_skill_file("diagnostics.md")?;
    for code in RUSTC_ERRORS {
        assert!(diagnostics.contains(code), "diagnostics.md omits {code}");
    }
    Ok(())
}

#[test]
fn ebnf_grammar_matches_anodized_core_readme() -> Result<()> {
    let reference = read_skill_file("reference.md")?;
    let source = read(&repo_root()?.join("crates/anodized-core/README.md"))?;
    let grammar = skill_tools::fences::extract_one("README.md", &source, "ebnf")?;
    assert!(
        reference.contains(grammar.trim()),
        "reference.md's grammar has drifted from anodized-core's"
    );
    Ok(())
}

#[test]
fn every_item_kind_is_documented() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    for kind in [
        "free `fn`",
        "inherent `impl`",
        "`trait`",
        "`while`",
        "`struct`",
        "`enum`",
    ] {
        assert!(skill.contains(kind), "SKILL.md omits {kind}");
    }
    Ok(())
}

#[test]
fn skill_states_the_absence_of_old_and_implicit_output() -> Result<()> {
    let skill = read_skill_file("SKILL.md")?;
    assert!(skill.contains("There is no `old()`"));
    assert!(skill.contains("There is no implicit `output`"));
    Ok(())
}

#[test]
fn commands_reference_real_cargo_invocations() -> Result<()> {
    let commands = repo_root()?.join("plugins/anodized/commands");
    for name in skill_tools::COMMANDS {
        let body = read(&commands.join(name))?;
        assert!(body.contains("cargo"), "{name} invokes no cargo command");
    }
    let check = read(&commands.join("check.md"))?;
    for flag in CFG_FLAGS {
        assert!(check.contains(flag), "check.md omits the `{flag}` pass");
    }
    Ok(())
}
