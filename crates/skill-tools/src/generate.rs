//! Renders the skill's factual tables from this workspace, and keeps them in sync.
//!
//! Only the tables are generated. The prose around them is written by hand, because the
//! reason a clause exists is not derivable from the enum that names it. Each table lives
//! between a pair of markers, and everything outside those markers is left untouched.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use anodized_core::qualifiers::FnQualifiers;

use crate::{
    CFG_FLAGS, DIAGNOSTICS, KeywordDoc, Result, all_keywords, documented_keyword, read,
    read_skill_file, repo_root, skill_dir,
};

/// A generated region: the file that holds it and the identifier that names it.
///
/// Each identifier appears exactly once. An earlier draft rendered the same table into two
/// files and added a test to prove the copies matched, which was machinery guarding a
/// duplication that told the reader nothing new the second time.
pub const REGIONS: &[(&str, &str)] = &[
    ("SKILL.md", "clauses"),
    ("SKILL.md", "qualifiers"),
    ("SKILL.md", "item-kinds"),
    ("SKILL.md", "cfg-flags"),
    ("reference.md", "ebnf"),
    ("diagnostics.md", "diagnostics"),
];

/// Build configurations, paired with what each one does.
pub const CFG_EFFECTS: &[(&str, &str)] = &[
    (
        "anodized_discard_specs",
        "Specs are dropped entirely: not embedded, not type-checked. Fastest builds.",
    ),
    (
        "anodized_print",
        "A violated condition reports to stderr and execution continues.",
    ),
    (
        "anodized_panic",
        "A violated condition panics, without naming the condition. Pair with \
         `anodized_print` to be told which expression failed.",
    ),
    (
        "anodized_try",
        "Enables `try_call!`, which turns a violation into a `Result`. Requires `anodized_panic`.",
    ),
];

/// Item kinds, with the clauses each accepts and how the attribute is applied.
pub const ITEM_KINDS: &[(&str, &str, &str)] = &[
    ("free `fn`", "all clauses", "`#[spec(...)]` on the `fn`"),
    (
        "inherent `impl`",
        "all clauses, per method",
        "bare `#[spec]` on the `impl`, clauses on each `fn`",
    ),
    (
        "`trait`",
        "all clauses, per method",
        "bare `#[spec]` on the `trait`, clauses on each `fn`",
    ),
    (
        "`impl Trait for T`",
        "all clauses, narrowing only",
        "bare `#[spec]` on the `impl`, optional clauses on each `fn`",
    ),
    (
        "`while` / `for`",
        "`maintains`, `decreases`",
        "`#[spec(...)]` on the loop, inside a spec'd `fn`",
    ),
    ("`struct`", "`maintains`", "`#[spec(...)]` on the `struct`"),
    ("`enum`", "`maintains`", "`#[spec(...)]` on the `enum`"),
];

/// Diagnostics, each with what provoked it and what to do about it.
///
/// The message column is verbatim, so a test can assert both that `diagnostics.md` documents
/// it and that the macro still emits it.
pub const DIAGNOSTIC_DOCS: &[(&str, &str, &str)] = &[
    (
        "unknown spec field",
        "A clause name the parser does not recognize, often a guess from another crate.",
        "Use one of `requires`, `maintains`, `captures`, `ensures`, `decreases`, or a qualifier.",
    ),
    (
        "fields are out of order: the expected order is",
        "Clauses are present but in the wrong sequence.",
        "Reorder to qualifiers, `requires`, `maintains`, `captures`, `ensures`.",
    ),
    (
        "at most one `captures` field is allowed",
        "Two `captures:` fields in one spec.",
        "Merge them: `captures: [first = a, second = b]`.",
    ),
    (
        "no longer supported, use the following form instead",
        "`binds:` or `inspects:`, both removed.",
        "Write `ensures: |PAT| [EXPR, ...]` instead.",
    ),
    (
        "postcondition closure must have exactly one input",
        "An `ensures` closure taking zero or several inputs.",
        "Take exactly the return value: `ensures: |output| ...`.",
    ),
    (
        "qualifier does not take a value",
        "A qualifier written as `pure: true`.",
        "Write it bare: `pure,`.",
    ),
    (
        "this qualifier is redundant; remove it",
        "A composite qualifier alongside one of its components.",
        "Keep the composite alone; `pure` already implies `deterministic` and `effectfree`.",
    ),
    (
        "attributes are not supported here",
        "An attribute on a qualifier.",
        "Remove it; only conditions accept `#[cfg]`.",
    ),
    (
        "unsupported attribute; only `cfg` is allowed",
        "An attribute other than `#[cfg]` on a clause.",
        "Remove it.",
    ),
    (
        "multiple `cfg` attributes are not supported",
        "Two `#[cfg]` attributes on one clause.",
        "Combine them: `#[cfg(all(test, debug_assertions))]`.",
    ),
    (
        "`cfg` attribute is not supported here",
        "A `#[cfg]` on `captures`.",
        "Gate the postconditions that read the capture instead.",
    ),
    (
        "multiple `decreases` fields are not allowed",
        "Two loop variants.",
        "Keep one expression.",
    ),
    (
        "multiple `#[spec]` attributes on a single item are not supported",
        "Two `#[spec]` attributes stacked on one item.",
        "Merge the clauses into one attribute.",
    ),
    (
        "expected an expression",
        "A clause whose value is not an expression.",
        "Supply a `bool` expression, or a list of them.",
    ),
    (
        "expected an assignment",
        "A `captures` entry that is not `pattern = expression`.",
        "Write `captures: name = expr`.",
    ),
    (
        "expected an assignment or block",
        "A `captures` value that is neither an assignment nor a block.",
        "Write `captures: name = expr`, or a list of assignments.",
    ),
    (
        "expected a single expression",
        "`decreases` given a list.",
        "Supply one expression.",
    ),
    (
        "not allowed here due to `#[spec]`",
        "A trait function parameter pattern the macro cannot forward.",
        "Name the parameter; avoid `_`, `..`, `ref`, and macro patterns.",
    ),
    (
        "or-pattern not allowed here due to `#[spec]`",
        "An or-pattern in a trait function parameter.",
        "Name the parameter and match inside the body.",
    ),
    (
        "The enclosing trait must have a `#[spec]` annotation.",
        "Clauses on a trait function whose trait carries no `#[spec]`.",
        "Add a bare `#[spec]` to the trait.",
    ),
    (
        "Unsupported spec element on trait.",
        "Clauses on the `#[spec]` attached to a trait.",
        "Keep the trait's attribute bare; put clauses on its functions.",
    ),
    (
        "Unsupported spec element on trait impl.",
        "Clauses on the `#[spec]` attached to an `impl Trait for T`.",
        "Keep it bare; put clauses on the functions.",
    ),
    (
        "Unsupported spec element on inherent impl.",
        "Clauses on the `#[spec]` attached to an inherent `impl`.",
        "Keep it bare; put clauses on the methods.",
    ),
    (
        "cannot be weaker than the qualifiers on the trait",
        "An impl claiming fewer guarantees than the trait promised.",
        "An impl may only narrow; restore the trait's qualifiers.",
    ),
    (
        "The #[spec] attribute doesn't yet support this item",
        "`#[spec]` on an item kind with no support, such as `mod` or `type`.",
        "Move it to a supported item.",
    ),
    (
        "`try_call` needs the `anodized_try` build `cfg` to be enabled",
        "`try_call!` without the build configuration that generates its entry points.",
        "Build with `--cfg anodized_try` and `--cfg anodized_panic`.",
    ),
    (
        "precondition failed",
        "At runtime: a `requires` was false on entry. The panic raised without \
         `anodized_print` also uses this for the whole entry group, invariants included.",
        "Fix the call site, or the precondition if it was wrong.",
    ),
    (
        "postcondition failed",
        "At runtime: an `ensures` was false on exit. The panic raised without \
         `anodized_print` also uses this for the whole exit group, invariants included.",
        "Fix the body, or the postcondition if it was wrong.",
    ),
    (
        "preinvariant failed",
        "At runtime under `anodized_print`: a `maintains` was already false on entry.",
        "The state was wrong before the call; look at whoever produced it.",
    ),
    (
        "postinvariant failed",
        "At runtime under `anodized_print`: a `maintains` was false on exit.",
        "The body broke an invariant it was required to preserve.",
    ),
];

/// Renders the table named by `id`.
pub fn render(id: &str) -> Result<String> {
    match id {
        "clauses" => Ok(render_clauses()),
        "qualifiers" => Ok(render_qualifiers()),
        "item-kinds" => Ok(render_item_kinds()),
        "cfg-flags" => Ok(render_cfg_flags()),
        "diagnostics" => Ok(render_diagnostics()),
        "ebnf" => render_ebnf(),
        other => Err(format!("no renderer for generated region `{other}`").into()),
    }
}

fn render_clauses() -> String {
    let mut out = String::from("| Clause | Takes | Allowed on |\n| --- | --- | --- |\n");
    for keyword in all_keywords() {
        if let KeywordDoc::Current { name, takes, items } = documented_keyword(&keyword) {
            let _ = writeln!(out, "| `{name}` | {takes} | {items} |");
        }
    }
    out
}

fn render_qualifiers() -> String {
    let composites = [
        ("functional", FnQualifiers::FUNCTIONAL),
        ("pure", FnQualifiers::PURE),
        ("total", FnQualifiers::TOTAL),
    ];
    let primitives = [
        (
            "deterministic",
            FnQualifiers::DETERMINISTIC,
            "the return value depends only on the arguments",
        ),
        ("effectfree", FnQualifiers::EFFECTFREE, "no side effects"),
        (
            "infallible",
            FnQualifiers::INFALLIBLE,
            "does not panic or abort",
        ),
        (
            "terminating",
            FnQualifiers::TERMINATING,
            "does not run forever",
        ),
    ];

    let mut out = String::from("| Qualifier | Guarantees |\n| --- | --- |\n");
    for (name, bits) in composites {
        let parts: Vec<_> = primitives
            .iter()
            .filter(|(_, bit, _)| bits.contains(*bit))
            .map(|(part, _, _)| format!("`{part}`"))
            .collect();
        let _ = writeln!(out, "| `{name}` | {} |", parts.join(", "));
    }
    for (name, _, meaning) in primitives {
        let _ = writeln!(out, "| `{name}` | {meaning} |");
    }
    out
}

fn render_item_kinds() -> String {
    let mut out = String::from("| Item | Clauses | How |\n| --- | --- | --- |\n");
    for (item, clauses, how) in ITEM_KINDS {
        let _ = writeln!(out, "| {item} | {clauses} | {how} |");
    }
    out
}

fn render_cfg_flags() -> String {
    let mut out = String::from("| Build configuration | Effect |\n| --- | --- |\n");
    let _ = writeln!(
        out,
        "| *(none)* | Specs embedded and type-checked; no check runs. `captures` still \
         evaluate. |"
    );
    for (flag, effect) in CFG_EFFECTS {
        let _ = writeln!(out, "| `--cfg {flag}` | {effect} |");
    }
    out
}

fn render_diagnostics() -> String {
    let mut out = String::from("| Message | Cause | Fix |\n| --- | --- | --- |\n");
    for (message, cause, fix) in DIAGNOSTIC_DOCS {
        let _ = writeln!(out, "| ``{message}`` | {cause} | {fix} |");
    }
    out
}

fn render_ebnf() -> Result<String> {
    let path = repo_root()?.join("crates/anodized-core/README.md");
    let grammar =
        crate::fences::extract_one("crates/anodized-core/README.md", &read(&path)?, "ebnf")?;
    Ok(format!("```ebnf\n{grammar}```\n"))
}

fn open_marker(id: &str) -> String {
    format!("<!-- anodized:generated:{id} -->")
}

fn close_marker(id: &str) -> String {
    format!("<!-- /anodized:generated:{id} -->")
}

/// Replaces a region's contents, leaving everything outside the markers untouched.
///
/// A missing or duplicated marker is an error rather than a skip: a region deleted by a bad
/// merge would otherwise look exactly like a region that is up to date.
pub fn splice(text: &str, id: &str, body: &str) -> Result<String> {
    let (open, close) = (open_marker(id), close_marker(id));
    let opens = text.matches(&open).count();
    let closes = text.matches(&close).count();
    if opens != 1 || closes != 1 {
        return Err(format!(
            "expected exactly one `{id}` region, found {opens} opening and {closes} closing markers"
        )
        .into());
    }
    let start = text.find(&open).ok_or("marker vanished")? + open.len();
    let end = text.find(&close).ok_or("marker vanished")?;
    if end < start {
        return Err(format!("the `{id}` region's markers are in the wrong order").into());
    }
    Ok(format!("{}\n{}{}", &text[..start], body, &text[end..]))
}

/// The files that hold generated regions.
#[must_use]
pub fn region_files() -> BTreeSet<&'static str> {
    REGIONS.iter().map(|(file, _)| *file).collect()
}

/// Renders every region and returns the resulting file contents, without writing.
pub fn rendered_files() -> Result<Vec<(String, String, String)>> {
    let mut results = Vec::new();
    for file in region_files() {
        let current = read_skill_file(file)?;
        let mut updated = current.clone();
        for (region_file, id) in REGIONS {
            if *region_file == file {
                updated = splice(&updated, id, &render(id)?)?;
            }
        }
        results.push((file.to_owned(), current, updated));
    }
    Ok(results)
}

/// Rewrites every generated region in place, returning the files that changed.
pub fn write() -> Result<Vec<String>> {
    let mut changed = Vec::new();
    for (file, current, updated) in rendered_files()? {
        if current != updated {
            std::fs::write(skill_dir()?.join(&file), &updated)?;
            changed.push(file);
        }
    }
    Ok(changed)
}

/// Returns the stale regions, as `file` and marker id.
pub fn check() -> Result<Vec<String>> {
    let mut stale = Vec::new();
    for (file, current, _) in rendered_files()? {
        for (region_file, id) in REGIONS {
            if *region_file != file {
                continue;
            }
            if splice(&current, id, &render(id)?)? != current {
                stale.push(format!("{file}:{id}"));
            }
        }
    }
    Ok(stale)
}

/// The diagnostics documented in the generated table.
#[must_use]
pub fn documented_diagnostics() -> BTreeSet<&'static str> {
    DIAGNOSTIC_DOCS
        .iter()
        .map(|(message, _, _)| *message)
        .collect()
}

/// The diagnostics the tests assert against the sources.
#[must_use]
pub fn tracked_diagnostics() -> BTreeSet<&'static str> {
    DIAGNOSTICS.iter().copied().collect()
}

/// The build configurations documented in the generated table.
#[must_use]
pub fn documented_cfg_flags() -> BTreeSet<&'static str> {
    CFG_EFFECTS.iter().map(|(flag, _)| *flag).collect()
}

/// The build configurations the tests assert against the manifests.
#[must_use]
pub fn tracked_cfg_flags() -> BTreeSet<&'static str> {
    CFG_FLAGS.iter().copied().collect()
}
