#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use pretty_assertions::assert_eq;

use anodized_fmt::{Config, format_file};
use test_util::fmt::{Template, Variation};

static TEMPLATE: OnceLock<Template> = OnceLock::new();

/// Build a template for the following fragment:
///
///     #[spec(
///         // preconditions
///         requires: [
///             // precond 1
///             x > 0,
///             // precond 2
///             y < 2 * z
///         ],
///         // invariants
///         maintains: x + y == z,
///         // captures
///         captures: [
///             // capture 1
///             [first, second, third] = values,
///             State { active, count } = state.clone(),
///         ],
///         // return value binding
///         inspects: ret_val,
///         // postconditions
///         ensures: [
///             *output > x,
///             // postcond 2
///             *output < 100,
///         ],
///     )]
///     fn func(x: i32) -> i32 { todo!() }
///
#[rustfmt::skip]
fn make_template() -> Template {
    Template::new()
        .fixed("#[spec(\n")

        .z().fixed("// preconditions\n")
        .z().tokens("requires : [").fixed("\n")
        .z().fixed("// precond 1\n")
        .z().tokens("x > 0 ,").fixed("\n")
        .z().fixed("// precond 2\n")
        .z().tokens("y < 2 * x").fixed("\n")
        .z().tokens("] ,").fixed("\n")

        .z().fixed("// invariants\n")
        .z().tokens("maintains : x + y == z ,").fixed("\n")

        .z().fixed("// captures\n")
        .z().tokens("captures : [").fixed("\n")
        .z().fixed("// capture 1\n")
        .z().tokens("[ first , second , third ] =").p().fixed("values ,\n")
        .z().tokens("State { active , count } =").p().tokens("state . clone ( ) ,").fixed("\n")
        .z().tokens("] ,").fixed("\n")

        .z().fixed("// return value binding\n")
        .z().tokens("inspects : ret_val ,").fixed("\n")

        .z().fixed("// postconditions\n")
        .z().tokens("ensures : [").fixed("\n")
        .z().tokens("* output > x ,").fixed("\n")
        .z().fixed("// postcond 2\n")
        .z().tokens("* output < 100 ,").fixed("\n")
        .z().tokens("] ,").fixed("\n")

        .z().fixed(")]\n")
        .fixed("fn func(x: i32) -> i32 { todo!() }\n")
}

fuzz_target!(
    init: {
        TEMPLATE.set(make_template()).unwrap();
    },
    |variation: Variation| {
        let template = TEMPLATE.get().unwrap();
        let default_input = template.generate(Variation::default());
        let variant_input = template.generate(variation);

        let config = Config::default();
        let fmt_default = format_file(&default_input, &config).expect("formatting default");
        let fmt_variant = format_file(&variant_input, &config).expect("formatting variant");

        assert_eq!(fmt_variant, fmt_default);
    }
);
