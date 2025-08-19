# Anodized

> Harden your Rust with explicit contracts.

[![crates.io](https://img.shields.io/crates/v/anodized.svg)](https://crates.io/crates/anodized)
[![docs.rs](https://docs.rs/anodized/badge.svg)](https://docs.rs/anodized)
[![CI](https://github.com/mkovaxx/anodized/actions/workflows/ci.yml/badge.svg)](https://github.com/mkovaxx/anodized/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/mkovaxx/anodized/blob/main/LICENSE-Apache-2.0)
[![License: MIT](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/mkovaxx/anodized/blob/main/LICENSE-MIT)

Anodized is a pragmatic suite of tools to improve the correctness of your Rust code. It provides a procedural macro to annotate your functions and methods with **explicit contracts**, which are checked at runtime in debug builds to catch logical bugs early.

These annotations serve as the foundation for a larger **ecosystem of correctness tools**, creating a unified language for many different tools, including fuzz testing, formal verification, and more.

***

## Quick Start

### 1. Add Anodized to your project

```toml
[dependencies]
anodized = "0.1.0"
```

### 2. Add contracts to your functions

Use the `#[contract]` attribute to define `requires` (precondition), `ensures` (postcondition), and `maintains` (invariant) clauses. Each clause is a standard Rust expressions that evaluates to `bool` (i.e. a predicate). In an `ensures` clause, the function's return value is available as `output`.

```rust
use anodized::contract;

#[contract(
    requires: divisor != 0,
    ensures: output < dividend,
)]
fn checked_divide(dividend: i32, divisor: i32) -> i32 {
    dividend / divisor
}

fn main() {
    // This call satisfies the contract and runs fine.
    println!("10 / 2 = {}", checked_divide(10, 2));

    // This call violates the precondition and will panic in debug builds.
    println!("10 / 0 = {}", checked_divide(10, 0));
}
```

### 3. Run your code

In a **debug build** (`cargo run`), your code is automatically instrumented to check the contracts. A contract violation will cause a panic with a descriptive error message:

```
thread 'main' panicked at 'Precondition failed: divisor != 0', src/main.rs:17:5
```

In a **release build** (`cargo run --release`), all contract-checking overhead is compiled out, resulting in **zero performance cost** in your production code.

## The Vision: An Ecosystem for Correctness

Anodized is more than just a macro for runtime assertions; it's the foundation for a future suite of interoperable correctness tools. The `#[contract]` annotations provide a **single, unified language** that other tools can use to understand your code's intent.

The long-term vision includes developing a suite of `anodized-*` tools, such as:

- `anodized-docs`: A `cargo` subcommand that renders your explicit contracts as part of the generated documentation, making intended behavior clear to users.

- `anodized-fuzz`: A `cargo` subcommand that generates fuzz tests, using `requires` clauses to generate valid inputs, making fuzzing effortless and efficient.

- `anodized-verify`: A `cargo` subcommand that uses formal methods to prove at compile-time that contracts are upheld both by implementations and at call sites, providing mathematical guarantees of correctness.

This creates a spectrum of correctness tools, allowing you to choose the right combination for the job. From simple runtime checks to full formal proofs, all using the same contract annotations.

## Annotation Syntax

The `#[contract]` attribute provides a rich syntax for defining contracts, designed to be both powerful and ergonomic.

### Clauses

Contracts are built from three flavors of clauses:

- `requires: <predicate>`: Defines a **precondition**. This predicate must be true when the function is called.

- `ensures: <predicate>`: Defines a **postcondition**. This predicate must be true when the function returns.

- `maintains: <predicate>`: Defines an **invariant**. A convenience to add a predicate as both a pre- and a postcondition. It's most useful for expressing properties of `self` that a method must preserve.

You can include zero, one, or many clauses of each flavor. In terms of the meaning (semantics), multiple clauses of the same flavor are combined with a logical **AND** (`&&`).

```rust
#[contract(
    requires: self.is_initialized,
    requires: !self.is_locked, // equivalent to `self.is_initialized && !self.is_locked`
    maintains: self.len() <= self.capacity(),
)]
fn push(&mut self, value: T) { /* ... */ }
```

### The Return Value

In `ensures` clauses, you can refer to the function's return value using the default name `output`.

```rust
#[contract(
    ensures: output > 0,
)]
fn get_positive_value() -> i32 { /* ... */ }
```

If the name `output` collides with an existing identifier, you can rename it in two ways:

**1. Global Override**: Use the `returns` key to set a new default name for all `ensures` clauses within the annotation.

```rust
#[contract(
    returns: new_value,
    ensures: new_value > old_value,
)]
fn increment(old_value: i32) -> i32 { old_value + 1 }
```

**2. Per-Clause Override**: Use a closure-style syntax on a specific `ensures` clause. This has the highest precedence and only affects that single clause.

```rust
#[contract(
    // This clause uses the default name `output`.
    ensures: output.is_valid(),
    // This clause uses a specific local name `val`.
    ensures: |val| val.id() != 0,
)]
fn create_data() -> Data { /* ... */ }
```

**3. Multiple Overrides**: When used together, the per-clause override always takes precedence for its specific clause, while other clauses fall back to the global override.

```rust
// A function where 'output' is an argument name, requiring a global override.
#[contract(
    // Globally rename the return value to `result`.
    returns: result,
    // This clause uses the global name `result`.
    ensures: result > output,
    // This clause uses a per-clause override `val`, which takes precedence.
    ensures: |val| val % 2 == 0,
)]
fn calculate_even_result(output: i32) -> i32 { /* ... */ }
```

### Rust Expressions as Predicates

A core design principle of Anodized is that a predicate is written as a **standard Rust expression** that evaluates to `bool`. This is a deliberate choice that provides key benefits over using a custom language.

- **The Language You Already Know**: No need to learn yet another language to write the contracts. Write them in the one you already know: standard Rust. Call functions, macros (like `matches!`), or write `if` and `match` expressions, and so on. As long as it all evaluates to a `bool`, you're good to go.

- **An Integral Part of Your Code**: Your contracts aren't special comments or strings; they are real Rust expressions, fully integrated with your code. The Rust compiler checks every predicate for syntax and type errors, just like any other part of your code. If you misspell a variable, compare incompatible types, or make any other mistake, you'll get a familiar compile-time error pointing directly to the predicate that needs fixing.

## License

Anodized is distributed under the terms of both the MIT License and the Apache License (Version 2.0).

See LICENSE-MIT and LICENSE-APACHE for details.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.
