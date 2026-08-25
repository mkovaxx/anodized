[![crates.io](https://img.shields.io/crates/v/anodized.svg)](https://crates.io/crates/anodized)
[![docs.rs](https://docs.rs/anodized/badge.svg)](https://docs.rs/anodized)
[![CI tests](https://github.com/anodized-rs/anodized/actions/workflows/ci.yml/badge.svg)](https://github.com/anodized-rs/anodized/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/anodized-rs/anodized/blob/main/LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/anodized-rs/anodized/blob/main/LICENSE-APACHE)

<img width="100" alt="Anodized Logo" src="https://raw.githubusercontent.com/anodized-rs/anodized/main/assets/logo.svg">

> Harden your Rust with **specifications**.

# Anodized

Anodized aims to be for specifications what [Serde](https://serde.rs) is for serialization: a common layer for writing specs once and validating them with many tools.

Write preconditions, postconditions, loop invariants, and type refinements directly in standard Rust. Validate them with runtime checks, fuzzers, model checkers, or formal provers - without rewriting your specs when you switch or combine tools.

In the verification landscape, Anodized is in the spirit of behavioral specification languages like
[SPARK](<https://en.wikipedia.org/wiki/SPARK_(programming_language)>),
[JML](https://en.wikipedia.org/wiki/Java_Modeling_Language), and
[ACSL](https://en.wikipedia.org/wiki/ANSI/ISO_C_Specification_Language). Unlike JML and ACSL,
Anodized specs are not comments but attributes, so they are checked by the Rust compiler for syntax, scoping, and types, and understood by tools like `rust-analyzer` out of the box.

## The `spec` Attribute

<img style="max-width:630px;" alt="editor integration demo" src="https://raw.githubusercontent.com/anodized-rs/anodized/main/assets/anodized-editor-integration.gif">

- **highly expressive**: A condition is any Rust expression that evaluates to `bool`. Call functions, write `if` and `match` expressions, use macros, and so on.
- **deeply integrated**: No special toolchain components needed. The ordinary Rust compiler checks your specs for syntax, scoping, and type errors, and `rust-analyzer` offers type hints, completion, and navigation inside them.
- **widely compatible**: One spec, many enforcers. Start with the built-in runtime checks, then add fuzzers, model checkers, or formal provers for deeper assurance - in any combination, without rewriting anything.

## Quickstart

**1. Add Anodized to your project.**

```toml
[dependencies]
anodized = "0.6.0"
```

**2. Extend your code with specs.**

Use the `#[spec]` attribute to attach preconditions and postconditions to functions, invariants to loops, and refinements to data types. Each _condition_ is a standard Rust expression that evaluates to `bool`.

```rust,no_run
use anodized::spec;

#[spec(
    requires: [
        part >= 0.0,
        part <= whole,
        whole > 0.0,
    ],
    ensures: |output| [
        output >= 0.0,
        output <= 100.0,
    ],
)]
fn calculate_percentage(part: f64, whole: f64) -> f64 {
    100.0 * part / whole
}
```

**3. Validate your code against the specs.**

Use one or more enforcement tools.

The easiest is runtime checks, which Anodized provides out of the box. All you need is tests that make function calls.

```rust,no_run
#[test]
fn percentage_25_over_100() {
    // This call satisfies the spec and runs fine.
    println!("25 out of 100 = {}%", calculate_percentage(25.0, 100.0));
}

#[test]
fn percentage_10_over_0() {
    // This call violates the precondition and will panic.
    println!("10 out of 0 = {}%", calculate_percentage(10.0, 0.0));
}
```

Use the `anodized_panic` setting to instrument the code with runtime checks. Add
`anodized_print` to have the failing condition named as well.

```bash
RUSTFLAGS="--cfg anodized_print --cfg anodized_panic" cargo test
```

A spec violation reports the condition, then panics:

```text
precondition failed: part <= whole
precondition failed: whole > 0.0
thread 'main' panicked at src/main.rs:3:1:
precondition failed
```

Conditions are checked independently rather than short-circuiting, so every violated one is
reported before the panic.

For more details and other approaches, see [The Anodized Reference](https://github.com/anodized-rs/anodized/blob/main/crates/anodized/REFERENCE.md).

## Using Anodized with a Coding Agent

Anodized ships an agent skill, so an assistant writes current `#[spec]` syntax instead of guessing from other contract systems.

```sh
# Claude Code
claude plugin marketplace add anodized-rs/anodized
claude plugin install anodized@anodized

# Codex
codex plugin marketplace add anodized-rs/anodized
codex plugin add anodized@anodized
```

The skill loads by itself when you mention Anodized or `#[spec]`. It also adds two commands: `/anodized:spec` to add specs to code you point it at, and `/anodized:check` to run your tests under every build configuration.

Using an agent with no plugin support? Point it at [the skill directory](https://github.com/anodized-rs/anodized/tree/main/plugins/anodized/skills/anodized).

## Why Anodized

Rust has a growing ecosystem of excellent verification tools - Aeneas, Creusot, Flux, Hax, Kani, Prusti, Verus, and more. Each one brings unique strengths. Anodized provides a shared specification layer so that these tools, and the developers who use them, do not have to go it alone.

**For developers writing specs:**

- Write your specs once in standard Rust, using the language you already know.
- Validate them with runtime checks right away, without learning any new tools.
- When you are ready for deeper assurance, connect the same specs to a fuzzer, model checker, or formal verifier - without rewriting anything.
- Switch or combine tools freely. Your specs are yours, not locked to any single tool.

**For developers building verification tools:**

- Consume Anodized specs instead of building and maintaining a custom spec frontend.
- Focus your effort on the analysis that makes your tool unique.
- Reach developers who are already writing specs, lowering the barrier to trying your tool.

### Relationship to Rust Native Contracts

The Rust Team is building [native contract support](https://github.com/rust-lang/rust/issues/128044) into the language. Anodized is a library-level experiment that lets developers start writing specs on stable Rust today. We hope to actively participate in the native contracts effort and contribute useful data points from practical experience with Anodized. When Rust-native contracts stabilize, we plan to offer a migration tool so that Anodized users can transition smoothly.

## Why Write Specs as Rust Code

A core design principle of Anodized is that a spec uses **standard Rust syntax**. This is a deliberate choice that provides key benefits over using a custom language.

- **The Language You Already Know**: No need to learn yet another language to write the specs. Write them in the one you already know: standard Rust. Call functions, use macros (like `matches!`), or write `if` and `match` expressions, and so on. As long as it syntax- and type-checks, it is good to go.

- **An Integral Part of Your Code**: Specs are not special comments or strings; they are real Rust expressions, fully integrated with your code. The Rust compiler checks every spec for syntax and type errors, just like any other part of your code. If you misspell a variable, compare incompatible types, or make any other mistake, you will get a familiar compiler error pointing directly to the spec element that needs fixing.

## Why "Spec" Instead of "Contract"

The choice of "specification" (or "spec") over "contract" is deliberate. While Design by Contract has a rich history, the term "contract" is now strongly associated with blockchain. This is particularly true in Rust, which has become a leading language for smart contract development.

This naming collision hurts discoverability. Searching for "Rust contract" yields many blockchain results, not just correctness tools.

Using "specification" instead:

- **Improves discoverability**: Developers find correctness tools when searching for them.
- **Reduces confusion**: The distinction from blockchain is immediately clear.
- **Maintains clarity**: "Specification" accurately describes these formal behavior annotations.

The term "spec" is already familiar from test specs, API specs, and formal specifications. It conveys the same meaning as Design by Contract while avoiding modern ambiguity.

## Prior Art and Motivation

The idea of adding contracts to Rust is not new, and Anodized builds upon the great work and ideas from several other projects and discussions in the community. It is a fresh take with a strong focus on ergonomics and a forward-looking vision for an integrated ecosystem.

**The `contracts` Crate**

The most direct and popular predecessor is the [`contracts`](https://crates.io/crates/contracts) crate. It is a mature and feature-rich library that also provides `#[requires]`, `#[ensures]`, and `#[invariant]` attributes. It has been a major inspiration for Anodized.

Anodized differentiates itself with a few key design choices:

- **Unified Attribute**: Anodized uses a single, comprehensive `#[spec]` attribute, presenting each specification as one cohesive block.

- **Ergonomic Focus**: The design process has been heavily focused on refining the user-facing syntax (e.g. keyword choices, return value binding) to be as intuitive, approachable, and powerful as possible.

- **Ecosystem Vision**: While `contracts` is an excellent tool for runtime checking, Anodized is designed from the ground up to be a foundational layer for a wider ecosystem of diverse correctness tools.

**Other Crates**

Older crates like `libhoare` (a compiler plugin from before procedural macros were stabilized) and `dbc` explored similar ideas, proving the long-standing interest in Design by Contract within the Rust community. Anodized benefits from the modern procedural macro system, which allows for much better integration with the compiler and toolchain.

**Inspiration from Other Languages**

Anodized is also inspired by languages where contracts are a first-class feature, not just a library. Languages like [Whiley](https://whiley.org), [Eiffel](https://eiffel.org), and [Ada/SPARK](https://adacore.com/about-spark) demonstrate the power of deeply integrating formal specifications into the syntax, type system, and toolchain. The Anodized ecosystem begins with one library, but shares the great ambition of those languages: to bring a similar level of integration and ergonomic feel to Rust.

## License

Anodized is distributed under the terms of the MIT License and the Apache License (Version 2.0). Users can choose either license, and contributors must license their changes under both.

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## Technical Documentation

For detailed technical documentation including the formal specification grammar and runtime check implementation details, see the [`anodized-core`](https://docs.rs/anodized-core) documentation.
