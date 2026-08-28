//! Generates and validates the Anodized agent skill shipped under `plugins/anodized`.
//!
//! The skill is a distributed artifact whose factual tables are derived from this workspace.
//! This crate renders those tables ([`generate`]) and asserts, from the integration tests,
//! that nothing in the skill has drifted away from the code it documents.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use anodized_core::annotate::syntax::Keyword;

pub mod fences;
pub mod generate;
pub mod scratch;

/// The result type used throughout, so callers propagate with `?` instead of panicking.
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// The workspace version, inherited via `version.workspace = true`.
///
/// This is what couples the plugin manifests to the crates they document: the manifests carry
/// a version, and a test asserts it equals this one.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Plugin manifests, relative to the repository root.
pub const MANIFESTS: &[&str] = &[
    ".claude-plugin/marketplace.json",
    ".agents/plugins/marketplace.json",
    "plugins/anodized/.claude-plugin/plugin.json",
    "plugins/anodized/.codex-plugin/plugin.json",
];

/// The manifests carrying a `version` field, which must agree with [`VERSION`].
pub const VERSIONED_MANIFESTS: &[&str] = &[
    ".claude-plugin/marketplace.json",
    "plugins/anodized/.claude-plugin/plugin.json",
    "plugins/anodized/.codex-plugin/plugin.json",
];

/// Slash commands, relative to the plugin root.
pub const COMMANDS: &[&str] = &["check.md", "spec.md"];

/// Every file the skill directory is allowed to contain.
pub const SKILL_FILES: &[&str] = &[
    "SKILL.md",
    "reference.md",
    "examples.md",
    "diagnostics.md",
    "migration.md",
];

/// Documents outside the skill whose Rust blocks are compiled by the same harness.
///
/// `REFERENCE.md` is not doctested, and it drifted: before this was added it taught an
/// `ensures` form that stopped compiling in 0.6.
pub const VERIFIED_DOCS: &[&str] = &["crates/anodized/REFERENCE.md"];

/// The marker placed directly after the frontmatter, identifying the skill as ours.
pub const SENTINEL: &str = "<!-- anodized-skill -->";

/// The upper bound on `description` + `when_to_use`, past which the listing is truncated.
pub const FRONTMATTER_BUDGET: usize = 1536;

/// Build configuration flags, which must match the `check-cfg` declarations.
pub const CFG_FLAGS: &[&str] = &[
    "anodized_discard_specs",
    "anodized_panic",
    "anodized_print",
    "anodized_try",
];

/// Crate sources searched for the diagnostics in [`DIAGNOSTICS`].
pub const DIAGNOSTIC_SOURCES: &[&str] = &["crates/anodized-core/src", "crates/anodized-macros/src"];

/// Diagnostics the macro emits verbatim, which `diagnostics.md` must document.
///
/// Each is asserted to still exist in the sources listed by [`DIAGNOSTIC_SOURCES`], so that
/// rewording a message in the macro fails this crate's tests rather than silently turning the
/// documentation into a lie.
pub const DIAGNOSTICS: &[&str] = &[
    "unknown spec field",
    "fields are out of order: the expected order is",
    "at most one `captures` field is allowed",
    "no longer supported, use the following form instead",
    "postcondition closure must have exactly one input",
    "qualifier does not take a value",
    "this qualifier is redundant; remove it",
    "attributes are not supported here",
    "unsupported attribute; only `cfg` is allowed",
    "multiple `cfg` attributes are not supported",
    "`cfg` attribute is not supported here",
    "multiple `decreases` fields are not allowed",
    "multiple `#[spec]` attributes on a single item are not supported",
    "expected an expression",
    "expected an assignment",
    "expected an assignment or block",
    "expected a single expression",
    "not allowed here due to `#[spec]`",
    "or-pattern not allowed here due to `#[spec]`",
    "The enclosing trait must have a `#[spec]` annotation.",
    "Unsupported spec element on trait.",
    "Unsupported spec element on trait impl.",
    "Unsupported spec element on inherent impl.",
    "cannot be weaker than the qualifiers on the trait",
    "The #[spec] attribute doesn't yet support this item",
    "`try_call` needs the `anodized_try` build `cfg` to be enabled",
    "precondition failed",
    "postcondition failed",
    "preinvariant failed",
    "postinvariant failed",
];

/// Rustc errors reached indirectly through a spec, which `diagnostics.md` must document.
pub const RUSTC_ERRORS: &[&str] = &[
    "E0005", "E0015", "E0046", "E0308", "E0425", "E0562", "E0596",
];

/// How a [`Keyword`] is presented to the reader of the skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordDoc {
    /// A bare qualifier in the current syntax.
    Qualifier {
        /// The qualifier as written in a spec.
        name: &'static str,
    },
    /// A value clause in the current syntax, documented in `SKILL.md`.
    Current {
        /// The clause name as written in a spec.
        name: &'static str,
        /// What the clause takes.
        takes: &'static str,
        /// Where the clause may appear.
        items: &'static str,
    },
    /// A clause removed from the syntax, documented only as an error to recognize.
    Removed {
        /// The clause name as it used to be written.
        name: &'static str,
        /// What replaced it.
        replacement: &'static str,
    },
    /// Not a clause at all: the parser's catch-all for an unrecognized field.
    NotAClause,
}

/// Classifies a keyword for documentation.
///
/// The `match` is exhaustive with no wildcard arm on purpose. Adding a variant to [`Keyword`]
/// breaks the build here, before any test runs, so a new clause cannot reach a release
/// undocumented.
#[must_use]
pub fn documented_keyword(keyword: &Keyword) -> KeywordDoc {
    match keyword {
        Keyword::Unknown(_) => KeywordDoc::NotAClause,
        Keyword::Functional => qualifier("functional"),
        Keyword::Pure => qualifier("pure"),
        Keyword::Total => qualifier("total"),
        Keyword::Deterministic => qualifier("deterministic"),
        Keyword::Effectfree => qualifier("effectfree"),
        Keyword::Infallible => qualifier("infallible"),
        Keyword::Terminating => qualifier("terminating"),
        Keyword::Requires => KeywordDoc::Current {
            name: "requires",
            takes: "one condition, or a list",
            items: "`fn`",
        },
        Keyword::Maintains => KeywordDoc::Current {
            name: "maintains",
            takes: "one condition, or a list",
            items: "`fn`, loop, `struct`, `enum`",
        },
        Keyword::Captures => KeywordDoc::Current {
            name: "captures",
            takes: "`pattern = expression`, or a list; at most one field",
            items: "`fn`",
        },
        Keyword::Binds => KeywordDoc::Removed {
            name: "binds",
            replacement: "`ensures: |PAT| [EXPR, ...]`",
        },
        Keyword::Inspects => KeywordDoc::Removed {
            name: "inspects",
            replacement: "`ensures: |PAT| [EXPR, ...]`",
        },
        Keyword::Ensures => KeywordDoc::Current {
            name: "ensures",
            takes: "a condition, `\\|pattern\\| condition`, or a list of either",
            items: "`fn`",
        },
        Keyword::Decreases => KeywordDoc::Current {
            name: "decreases",
            takes: "one expression; at most one field",
            items: "loop only",
        },
    }
}

fn qualifier(name: &'static str) -> KeywordDoc {
    KeywordDoc::Qualifier { name }
}

/// Every keyword the parser recognizes, in declaration order.
///
/// Declaration order is the parser's mandatory clause order, enforced by `Ord`.
#[must_use]
pub fn all_keywords() -> Vec<Keyword> {
    vec![
        Keyword::Functional,
        Keyword::Pure,
        Keyword::Total,
        Keyword::Deterministic,
        Keyword::Effectfree,
        Keyword::Infallible,
        Keyword::Terminating,
        Keyword::Requires,
        Keyword::Maintains,
        Keyword::Captures,
        Keyword::Binds,
        Keyword::Inspects,
        Keyword::Ensures,
        Keyword::Decreases,
    ]
}

/// The repository root, derived from this crate's location.
pub fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot locate the repository root above CARGO_MANIFEST_DIR".into())
}

/// The directory holding the skill.
pub fn skill_dir() -> Result<PathBuf> {
    Ok(repo_root()?.join("plugins/anodized/skills/anodized"))
}

/// The plugin root.
pub fn plugin_dir() -> Result<PathBuf> {
    Ok(repo_root()?.join("plugins/anodized"))
}

/// Reads a file, reporting the path when it cannot be read.
pub fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|err| format!("cannot read {}: {err}", path.display()).into())
}

/// Reads and parses a JSON file.
pub fn read_json(path: &Path) -> Result<serde_json::Value> {
    let text = read(path)?;
    serde_json::from_str(&text)
        .map_err(|err| format!("cannot parse {} as JSON: {err}", path.display()).into())
}

/// Reads one of the skill's markdown files.
pub fn read_skill_file(name: &str) -> Result<String> {
    read(&skill_dir()?.join(name))
}

/// Concatenates every source file under the given repository-relative directories.
pub fn read_sources(dirs: &[&str]) -> Result<String> {
    let root = repo_root()?;
    let mut combined = String::new();
    for dir in dirs {
        collect_rust_sources(&root.join(dir), &mut combined)?;
    }
    Ok(combined)
}

fn collect_rust_sources(dir: &Path, out: &mut String) -> Result<()> {
    let entries =
        fs::read_dir(dir).map_err(|err| format!("cannot read {}: {err}", dir.display()))?;
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            collect_rust_sources(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push_str(&read(&path)?);
            out.push('\n');
        }
    }
    Ok(())
}
