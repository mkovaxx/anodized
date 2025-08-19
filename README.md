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

Use the `#[logic]` attribute to define `requires` (precondition), `ensures` (postcondition), and `maintains` (invariant) clauses. Each clause is a standard Rust expressions that evaluates to `bool` (i.e. a predicate). In an `ensures` clause, the function's return value is available as `output`.

```rust
use anodized::logic;

#[logic(
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

Anodized is more than just a macro for runtime assertions; it's the foundation for a future suite of interoperable correctness tools. The `#[logic]` annotations provide a **single, unified language** that other tools can use to understand your code's intent.

The long-term vision includes developing a suite of `anodized-*` tools, such as:

- `anodized-docs`: A `cargo` subcommand that renders your explicit contracts as part of the generated documentation, making intended behavior clear to users.

- `anodized-fuzz`: A `cargo` subcommand that generates fuzz tests, using `requires` clauses to generate valid inputs, making fuzzing effortless and efficient.

- `anodized-verify`: A `cargo` subcommand that uses formal methods to prove at compile-time that contracts are upheld both by implementations and at call sites, providing mathematical guarantees of correctness.

This creates a spectrum of correctness tools, allowing you to choose the right combination for the job. From simple runtime checks to full formal proofs, all using the same contract annotations.

## License

Anodized is distributed under the terms of both the MIT License and the Apache License (Version 2.0).

See LICENSE-MIT and LICENSE-APACHE for details.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.
