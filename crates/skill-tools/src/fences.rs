//! Extracts fenced code blocks from the skill's markdown.
//!
//! Every Rust block in the skill is compiled by the integration tests, so the info string on a
//! fence is a contract rather than a hint: it declares what the block is for, and an
//! unrecognized one is an error instead of a silent skip.

use std::fmt;

use crate::Result;

/// What a fence's info string declares about its block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Rust that must compile, and — when it carries a `fn main` — must also run with checks
    /// enabled without tripping one of its own conditions.
    Pass,
    /// Rust that must compile. Identical treatment to [`Kind::Pass`]; the tag exists because
    /// `REFERENCE.md` already uses it.
    CompileOnly,
    /// Rust that must fail to compile, with the error code the block declares.
    CompileFail,
    /// A deliberate fragment, excluded from compilation.
    Fragment,
    /// Not Rust.
    Other,
}

/// A fenced block, with its position for error reporting.
#[derive(Debug, Clone)]
pub struct Fence {
    /// The markdown file the block came from.
    pub file: String,
    /// The line the opening fence sits on, 1-based.
    pub line: usize,
    /// The info string, normalized.
    pub info: String,
    /// What the info string declares.
    pub kind: Kind,
    /// The block's contents, without the fences.
    pub body: String,
    /// The nearest `##` heading above the block.
    pub heading: String,
}

impl fmt::Display for Fence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

impl Fence {
    /// The rustc error code a `compile_fail` block declares on its first line.
    ///
    /// The declaration is what keeps the block honest: without it, a block that fails for an
    /// unrelated reason would pass just as happily as one that fails for the documented one.
    pub fn expected_error(&self) -> Result<String> {
        let first = self.body.lines().next().unwrap_or_default().trim();
        first
            .strip_prefix("// EXPECT:")
            .map(|code| code.trim().to_owned())
            .filter(|code| !code.is_empty())
            .ok_or_else(|| {
                format!("{self}: a `compile_fail` block must start with `// EXPECT: <code>`").into()
            })
    }

    /// Whether a fragment declares itself as one.
    #[must_use]
    pub fn is_labelled_fragment(&self) -> bool {
        self.body
            .lines()
            .next()
            .is_some_and(|line| line.trim_start().starts_with("// fragment"))
    }

    /// The nearest `##` heading above this block, used to key its driver.
    #[must_use]
    pub fn heading(&self) -> &str {
        &self.heading
    }

    /// Whether the block exercises itself, and so can be run rather than only compiled.
    ///
    /// Compiling proves a spec is well-formed. Only running it proves the spec is *true of the
    /// code*: an example once shipped here promised `output >= items.len()` from a sum that is
    /// zero for an all-zero slice, which compiled perfectly and panicked on the first honest
    /// input.
    #[must_use]
    pub fn is_self_exercising(&self) -> bool {
        self.body.contains("fn main(")
    }

    /// The block as a compilable unit, with the import a snippet omits.
    ///
    /// No `fn main` is appended: blocks are compiled as a library, because they are item-level
    /// and a binary crate would fail with `E0601` before reaching the spec.
    #[must_use]
    pub fn as_program(&self) -> String {
        // Two lints are artifacts of a snippet being illustrative rather than complete: an
        // example item nobody calls, and a `todo!()` body that never reads its parameters.
        // Every other warning fails the build, so an example cannot teach code that warns —
        // which is how the tautological `output >= 0` on a `u64` was caught.
        let mut source = String::from("#![allow(dead_code, unused_variables)]\n");
        if !self.body.contains("use anodized::spec;") {
            source.push_str("use anodized::spec;\n");
        }
        source.push_str(&self.body);
        source
    }
}

/// Classifies an info string, normalizing whitespace first.
///
/// Normalization matters: `REFERENCE.md` writes `rust, no_run` with a space, and rejecting it
/// over the space would mean rewriting a dozen fences to gain nothing.
pub fn classify(info: &str) -> Result<Kind> {
    let normalized: String = info.chars().filter(|c| !c.is_whitespace()).collect();
    match normalized.as_str() {
        "rust" => Ok(Kind::Pass),
        "rust,no_run" => Ok(Kind::CompileOnly),
        "rust,compile_fail" => Ok(Kind::CompileFail),
        "rust,ignore" => Ok(Kind::Fragment),
        "" | "text" | "toml" | "bash" | "sh" | "json" | "yaml" | "ebnf" | "diff" | "console" => {
            Ok(Kind::Other)
        }
        other => Err(format!(
            "unknown fence tag `{other}`; extend `skill_tools::fences::classify` deliberately \
             rather than adding an untracked tag"
        )
        .into()),
    }
}

/// Extracts every fenced block from a markdown document.
///
/// Blocks inside the YAML frontmatter are not fences and are skipped along with it.
pub fn extract(file: &str, text: &str) -> Result<Vec<Fence>> {
    let mut fences = Vec::new();
    let mut lines = text.lines().enumerate().peekable();

    if text.starts_with("---\n") {
        lines.next();
        for (_, line) in lines.by_ref() {
            if line.trim_end() == "---" {
                break;
            }
        }
    }

    let mut heading = String::new();
    while let Some((index, line)) = lines.next() {
        if let Some(title) = line.strip_prefix("## ") {
            heading = title.trim().to_owned();
        }
        let Some(info) = line.strip_prefix("```") else {
            continue;
        };
        let kind = classify(info)?;
        let mut body = String::new();
        let mut closed = false;
        for (_, inner) in lines.by_ref() {
            if inner.starts_with("```") {
                closed = true;
                break;
            }
            body.push_str(inner);
            body.push('\n');
        }
        if !closed {
            return Err(format!("{file}:{}: unclosed code fence", index + 1).into());
        }
        fences.push(Fence {
            file: file.to_owned(),
            line: index + 1,
            info: info.trim().to_owned(),
            kind,
            body,
            heading: heading.clone(),
        });
    }

    Ok(fences)
}

/// Extracts the single block carrying the given info string.
pub fn extract_one(file: &str, text: &str, info: &str) -> Result<String> {
    let matching: Vec<_> = extract(file, text)?
        .into_iter()
        .filter(|fence| fence.info == info)
        .collect();
    match matching.as_slice() {
        [fence] => Ok(fence.body.clone()),
        [] => Err(format!("{file}: expected one `{info}` block, found none").into()),
        many => Err(format!("{file}: expected one `{info}` block, found {}", many.len()).into()),
    }
}
