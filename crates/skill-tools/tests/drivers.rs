//! Drivers that exercise the skill's examples over their admitted inputs.
//!
//! A `fn main` inside an example proves the spec only on the values that `main` happens to
//! pass, and only for the examples an author remembered to write one for. Every defect found
//! in review so far hid at a boundary a tame driver would miss: a sum bound false only for an
//! all-zero slice, a trait promise false only where the product overflows, an increment false
//! only at `u32::MAX`. These drivers sweep the boundaries instead, and live here rather than in
//! the documents so the shipped examples stay readable.

/// The documents whose examples must every one be driven or declared undriven.
///
/// `reference.md`, `diagnostics.md`, and `migration.md` carry no runnable examples;
/// `REFERENCE.md` is excluded because most of its illustrations have `todo!()` bodies, which a
/// driver cannot execute.
pub const DRIVEN_FILES: &[&str] = &["examples.md", "SKILL.md"];

/// Prepended to every driven program.
///
/// `rejects` is what lets a driver test a precondition rather than merely respect it. Sweeping
/// only admitted inputs proves a spec is not violated; it cannot prove the spec is strong
/// enough, because the input that would break the body is exactly the one the driver avoids.
/// Asserting that an out-of-domain call fails *as a precondition* — and not as an overflow or
/// a divide-by-zero from the body — is what catches a precondition that admits too much.
pub const PRELUDE: &str = r#"
fn rejects<T>(call: impl FnOnce() -> T + std::panic::UnwindSafe) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(call);
    std::panic::set_hook(previous);
    let payload = outcome.err().expect("the spec accepted an input it should reject");
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or_default();
    assert!(
        message.contains("precondition failed"),
        "expected a precondition to reject this input; the body failed first with: {message}"
    );
}
"#;

/// A driver for one example.
///
/// Keyed by file and by a substring the example defines, rather than by its heading: two
/// examples can sit under one heading, and headings get rewritten.
pub struct Driver {
    /// The document the example lives in.
    pub file: &'static str,
    /// A substring that appears in this example and no other in the same file.
    pub defines: &'static str,
    /// A `fn main` appended to the example's own source.
    pub main: &'static str,
}

/// Examples whose bodies are illustrative rather than real, so nothing can be run.
///
/// These are verified by reading. Keeping them named here is what stops the set growing
/// silently: a new example must either be driven or be listed as deliberately undriven.
pub const UNDRIVEN: &[(&str, &str)] = &[
    // Data specs: `self` is in scope but there is nothing to call, and neither a `struct` nor
    // an `enum` invariant is checked at runtime, so a driver would exercise nothing.
    ("examples.md", "struct Bounds"),
    ("examples.md", "enum Temperature"),
    ("SKILL.md", "struct Range"),
];

/// The drivers, one per runnable example.
pub const DRIVERS: &[Driver] = &[
    Driver {
        file: "examples.md",
        defines: "fn nth",
        main: r#"fn main() {
            for slice in [&[7u32][..], &[1, 2, 3][..], &[0, 0][..]] {
                for index in 0..slice.len() {
                    assert_eq!(nth(slice, index), slice[index]);
                }
                rejects(move || nth(slice, slice.len()));
            }
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn bump",
        main: r#"fn main() {
            for counter in [0u32, 1, u32::MAX - 1] {
                assert_eq!(bump(counter), counter + 1);
            }
            rejects(|| bump(u32::MAX));
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn overwrite_first",
        main: r#"fn main() {
            let mut buffer = vec![1u32];
            overwrite_first(&mut buffer, 9);
            assert_eq!(buffer[0], 9);
            let mut longer = vec![1u32, 2, 3];
            overwrite_first(&mut longer, 0);
            let mut empty: Vec<u32> = vec![];
            rejects(move || overwrite_first(&mut empty, 9));
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn divide",
        main: r#"fn main() {
            for dividend in [0u32, 1, 7, u32::MAX] {
                for divisor in [1u32, 2, 7, u32::MAX] {
                    let _ = divide(dividend, divisor);
                }
            }
            rejects(|| divide(1, 0));
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn checked_divide",
        main: r#"fn main() {
            for divisor in [0u32, 1, u32::MAX] {
                let _ = checked_divide(10, divisor);
            }
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn larger",
        main: r#"fn main() {
            for left in [0u32, 1, u32::MAX] {
                for right in [0u32, 1, u32::MAX] {
                    let _ = larger(left, right);
                }
            }
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn power_of_two",
        main: r#"fn main() {
            for exponent in 0..32u32 {
                let _ = power_of_two(exponent);
            }
            rejects(|| power_of_two(32));
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "struct Counter",
        main: r#"fn main() {
            let mut counter = Counter::new();
            for _ in 0..3 {
                let _ = counter.increment();
            }
            let mut maxed = Counter::new();
            maxed.value = u32::MAX;
            rejects(move || maxed.increment());
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "trait Scale",
        main: r#"fn main() {
            for factor in 1..=8u32 {
                for value in [0, 1, u32::MAX / factor] {
                    assert!(Doubling.scale(value, factor) >= value);
                    assert!(Exact.scale(value, factor) >= value);
                }
            }
            assert_eq!(Doubling.scale(u32::MAX, 1), u32::MAX);
            rejects(|| Doubling.scale(u32::MAX, 2));
            rejects(|| Doubling.scale(5, 0));
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn maximum",
        main: r#"fn main() {
            assert_eq!(maximum(&[5]), 5);
            assert_eq!(maximum(&[1, 9, 3]), 9);
            assert_eq!(maximum(&[0, 0]), 0);
            rejects(|| maximum(&[]));
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn first",
        main: r#"fn main() {
            assert_eq!(first(&[4]), 4);
            assert_eq!(first(&[7, 8]), 7);
            rejects(|| first(&[]));
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn safe_divide",
        main: r#"fn main() {
            for divisor in [0u32, 1, 3] {
                let _ = safe_divide(9, divisor);
            }
        }"#,
    },
    Driver {
        file: "examples.md",
        defines: "fn reversed",
        main: r#"fn main() {
            for items in [vec![], vec![7u32], vec![1, 2, 3], vec![0, 0]] {
                let out = reversed(items.clone());
                assert_eq!(out.len(), items.len());
                assert_eq!(out.last().copied(), items.first().copied());
            }
        }"#,
    },
    Driver {
        file: "SKILL.md",
        defines: "captures: before = count",
        main: r#"fn main() {
            for count in [0u32, 1, u32::MAX - 1] {
                assert_eq!(increment(count), count + 1);
            }
            rejects(|| increment(u32::MAX));
        }"#,
    },
    Driver {
        file: "SKILL.md",
        defines: "fn one()",
        main: r#"fn main() {
            assert_eq!(one(), 1);
        }"#,
    },
    Driver {
        file: "SKILL.md",
        defines: "fn sorted",
        main: r#"fn main() {
            for items in [vec![], vec![5u32], vec![3, 1, 2], vec![0, 0]] {
                let sorted_items = sorted(items.clone());
                assert_eq!(sorted_items.len(), items.len());
                assert!(sorted_items.windows(2).all(|w| w[0] <= w[1]));
            }
        }"#,
    },
    Driver {
        file: "SKILL.md",
        defines: "fn maximum",
        main: r#"fn main() {
            assert_eq!(maximum(&[5]), 5);
            assert_eq!(maximum(&[1, 9, 3]), 9);
            assert_eq!(maximum(&[0, 0]), 0);
            rejects(|| maximum(&[]));
        }"#,
    },
];
