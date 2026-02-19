use anodized_fmt::{Config, format_file};
use pretty_assertions::assert_eq;

#[test]
fn test_format_simple_function() {
    let input = include_str!("fixtures/input/simple_function.rs");
    let expected = include_str!("fixtures/expected/simple_function.rs");

    let config = Config::default();
    let formatted = format_file(input, &config).expect("Failed to format");

    assert_eq!(formatted, expected);
}

#[test]
fn test_format_complex_spec() {
    let input = include_str!("fixtures/input/complex_spec.rs");
    let expected = include_str!("fixtures/expected/complex_spec.rs");

    let config = Config::default();
    let formatted = format_file(input, &config).expect("Failed to format");

    assert_eq!(formatted, expected);
}

#[test]
fn test_format_matches_and_functions() {
    let input = include_str!("fixtures/input/matches_and_functions.rs");
    let expected = include_str!("fixtures/expected/matches_and_functions.rs");

    let config = Config::default();
    let formatted = format_file(input, &config).expect("Failed to format");

    assert_eq!(formatted, expected);
}

#[test]
fn test_format_basic_traits() {
    let input = include_str!("fixtures/input/basic_traits.rs");
    let expected = include_str!("fixtures/expected/basic_traits.rs");

    let config = Config::default();
    let formatted = format_file(input, &config).expect("Failed to format");

    assert_eq!(formatted, expected);
}

#[test]
fn test_format_binds() {
    let input = include_str!("fixtures/input/with_binds.rs");
    let expected = include_str!("fixtures/expected/with_binds.rs");

    let config = Config::default();
    let formatted = format_file(input, &config).expect("Failed to format");

    assert_eq!(formatted, expected);
}

#[test]
fn test_format_capture_patterns() {
    let input = include_str!("fixtures/input/capture_patterns.rs");
    let expected = include_str!("fixtures/expected/capture_patterns.rs");

    let config = Config::default();
    let formatted = format_file(input, &config).expect("Failed to format");

    assert_eq!(formatted, expected);
}

#[test]
fn test_format_is_idempotent() {
    let input = include_str!("fixtures/input/simple_function.rs");

    let config = Config::default();

    // Format once
    let formatted1 = format_file(input, &config).expect("Failed to format first time");

    // Format again
    let formatted2 = format_file(&formatted1, &config).expect("Failed to format second time");

    // Should be the same
    assert_eq!(formatted1, formatted2, "Formatting should be idempotent");
}

#[test]
fn test_format_preserves_other_code() {
    let input = r#"
        // Some comment
        use anodized::spec;

        const VALUE: i32 = 42;

        #[spec(requires: x > 0)]
        fn foo(x: i32) -> i32 {
            x + VALUE
        }

        #[derive(Debug)]
        struct MyStruct {
            field: i32,
        }
        "#;

    let config = Config::default();
    let formatted = format_file(input, &config).expect("Failed to format");

    // Check that other code is preserved
    assert!(formatted.contains("// Some comment"));
    assert!(formatted.contains("const VALUE: i32 = 42;"));
    assert!(formatted.contains("#[derive(Debug)]"));
    assert!(formatted.contains("struct MyStruct"));
}
