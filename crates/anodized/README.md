[![crates.io](https://img.shields.io/crates/v/anodized.svg)](https://crates.io/crates/anodized)
[![docs.rs](https://docs.rs/anodized/badge.svg)](https://docs.rs/anodized)
[![CI tests](https://github.com/mkovaxx/anodized/actions/workflows/ci.yml/badge.svg)](https://github.com/mkovaxx/anodized/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/mkovaxx/anodized/blob/main/LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/mkovaxx/anodized/blob/main/LICENSE-APACHE)

<img width="100" alt="Anodized Logo" src="https://raw.githubusercontent.com/mkovaxx/anodized/main/assets/logo.svg">

# Anodized

> Harden your Rust with **specifications**.

Anodized is a pragmatic suite of tools that helps improve the correctness of your Rust code. Its central idea is **specification** annotations that are deeply integrated with your code, instead of isolated in comments or funky string literals.

Specifications serve as the foundation for a larger **ecosystem of correctness tools**. Anodized aims to connect many disparate approaches, including fuzz testing, formal verification, and more, into a unified user experience.

***

## Quick Start

**1. Add Anodized to your project.**

```toml
[dependencies]
anodized = "0.2.1"
```

**2. Add specifications to your functions.**

Use the `#[spec]` attribute to define preconditions (`requires`), postconditions (`ensures`), and invariants (`maintains`). Each _condition_ is a standard Rust expression that evaluates to `bool`. In postconditions, the function's return value is available as `output`.

```rust,no_run
use anodized::spec;

#[spec(
    requires: [
        part >= 0.0,
        part <= whole,
        whole > 0.0,
    ],
    ensures: [
        output >= 0.0,
        output <= 100.0,
    ],
)]
fn calculate_percentage(part: f64, whole: f64) -> f64 {
    (part / whole) * 100.0
}

fn main() {
    // This call satisfies the spec and runs fine.
    println!("25 out of 100 = {}%", calculate_percentage(25.0, 100.0));

    // This call violates the precondition and will panic in debug builds.
    println!("10 out of 0 = {}%", calculate_percentage(10.0, 0.0));
}
```

**3. Run or test your code as usual.**

In a **debug build** (`cargo run` or `cargo test`), your code is automatically instrumented to check the specifications. A spec violation will cause a panic with a descriptive error message:

```text
thread 'main' panicked at 'Precondition failed: whole > 0', src/main.rs:17:5
```

In a **release build** (`cargo run --release`), _runtime_ spec-checking is disabled, resulting in **zero performance cost** to your production code. Note that the compiler still checks specifications for errors such as bad syntax, unknown identifiers, type mismatches, etc.

## The Vision: An Ecosystem for Correctness

Anodized is more than just a macro for runtime assertions; it's the foundation for a future suite of interoperable correctness tools. The `#[spec]` annotations provide a **single, unified language** that other tools can use to understand your code's intent.

The long-term vision includes developing a suite of `anodized-*` tools, such as:

- `anodized-docs`: Render specifications as part of the generated documentation, making intended behavior clear to users.

- `anodized-fuzz`: Generate fuzz tests that choose valid inputs based on preconditions, making fuzzing effortless and efficient.

- `anodized-verify`: Prove formally that specifications are upheld both by implementations and at call sites, providing mathematical guarantees of correctness.

Anodized aims to support a wide spectrum of correctness tools, enabling you to choose the best combination for each project. From simple runtime checks to full formal proofs, leveraging the exact same specification annotations.

## `#[spec]`: Specifications

The `#[spec]` attribute provides a powerful and ergonomic way to define specifications.

### Preconditions, Postconditions, and Invariants

Specifications are built from conditions, which come in three flavors:

- **`requires: <conditions>`: Preconditions** must be true when the function is called.

- **`ensures: <conditions>`: Postconditions** must be true when the function returns.

- **`maintains: <conditions>`: Invariants** must hold true both before and after the function runs. It's most useful for expressing properties of `self` that a method must preserve.

For convenience, `<conditions>` can be either a single condition or a list (i.e. `[<condition>, <condition>, ...]`).

The conditions must be given in the following order: `requires`, `maintains`, and `ensures`. This order is enforced to mirror the logical flow of a function's execution: preconditions (`requires`) are checked upon entry, invariants (`maintains`) must hold true upon both entry and exit, and postconditions (`ensures`) are checked upon exit.

A condition is a `bool`-valued Rust expression; as simple as that. This is a non-trivial design choice, so its benefits are explained in the section below: [Why Conditions Are Rust Expressions](#why-conditions-are-rust-expressions).

You can include any number of each flavor. Multiple conditions of the same flavor are combined with a logical **AND** (`&&`).

```rust, no_run
use anodized::spec;

#[spec(
    // Precondition: the vector must have room for at least one more element
    requires: vec.len() < vec.capacity() || vec.capacity() == 0,
    // Invariant: length never exceeds capacity
    maintains: vec.len() <= vec.capacity(),
)]
fn push_checked<T>(vec: &mut Vec<T>, value: T) { todo!() }
```

### `#[cfg]`: Configure Runtime Checks

You can use the standard `#[cfg]` attribute to conditionally enable or disable the _runtime checks_ for any condition. This is ideal for expensive checks that you only want to run during testing or in debug builds.

```rust, no_run
use anodized::spec;

#[spec(
    // Runtime checks only during `cargo test`.
    #[cfg(test)]
    requires: input > 0,

    // Runtime checks only when debug assertions are enabled.
    #[cfg(debug_assertions)]
    ensures: |output: Result<i32, String>| output.is_ok(),
)]
fn perform_complex_operation(input: i32) -> Result<i32, String> { Ok(42) }
```

**Important:** Anodized guarantees that all your specifications are syntactically valid and type-correct, regardless of the `#[cfg]` attribute. The attribute only controls whether a check is performed at _runtime_. This ensures that e.g. a specification valid in `test` builds can't become invalid in `release` builds, and it allows other tools in the ecosystem (like static analyzers) to always see the full specification.

This gives you fine-grained control over the performance impact of your specifications, allowing you to write the conditions thoroughly without affecting release build performance.

### `clones`: Capture Entry-Time Values

Sometimes postconditions need to compare the function's final state with its initial state. The `clones` parameter lets you capture values at function entry for use in postconditions.

```rust, no_run
use anodized::spec;

#[spec(
    clones: [
        // Simple identifier: shorthand for `count as old_count`
        *count as old_count,
        // Complex expression: requires explicit alias
        vec.len() as orig_len,
    ],
    ensures: [
        *count == old_count + 1,
        vec.len() == orig_len + 1,
    ],
)]
fn add_item<T>(vec: &mut Vec<T>, count: &mut usize, item: T) { todo!() }
```

- **Simple identifiers** get an automatic `old_` prefix, i.e. `count` becomes `old_count`.
- **Complex expressions** require an explicit alias using `as`, i.e. `self.items.len() as orig_len`.
- Clone bindings are captured **after** preconditions are checked but **before** the function body executes.
- The cloned values are **only** available to postconditions, not to preconditions or the function body itself.
- Values are cloned using Rust's `Clone` trait, so the types must implement `Clone`.

### `binds`: Bind the Return Value

In **postconditions** (`ensures`), you can refer to the function's return value by the default name `output`.

```rust, no_run
use anodized::spec;

#[spec(
    ensures: output > 0,
)]
fn get_positive_value() -> i32 { todo!() }
```

**Note** that a postcondition is _always_ a closure, because it needs to bind the return value. When you write a postcondition as a "naked" expression, that is shorthand for using the default binding, i.e. `|output| <expression>`. In error messages, a postcondition is always displayed as a closure to make the binding explicit (e.g. `|output| output > 0`).

If the name `output` collides with an existing identifier, you can choose a different name for it in two ways:

**1. Spec-Wide Binding**: Use the `binds` parameter to set a new name for the return value across all postconditions in the specification. It must be placed immediately before any `ensures` conditions.

```rust, no_run
use anodized::spec;

#[spec(
    binds: new_value,
    ensures: new_value > old_value,
)]
fn increment(old_value: i32) -> i32 { todo!() }
```

 **2. Closure Binding**: Write the postcondition as a closure. This has the highest precedence and affects only that single condition.

```rust, no_run
use anodized::spec;

#[spec(
    ensures: [
        // This postcondition uses the default binding `output` - first element indicates validity
        |output: (bool, i32)| output.0,
        // This postcondition binds the return value as `val` - second element is the ID
        |val| val.1 != 0,
    ],
)]
fn create_data() -> (bool, i32) { (true, 42) }
```

**3. Binding Precedence**: The closure's binding takes precedence; same as in Rust. Plain postconditions still use the spec-wide binding.

```rust, no_run
use anodized::spec;

// A function where 'output' is an argument name, requiring a different name.
#[spec(
    // Set a spec-wide name for the return value: `result`.
    binds: result,
    ensures: [
        // This postcondition uses the spec-wide name `result`.
        result > output,
        // This postcondition is written as a closure and binds the return value as `val`.
        |val| val % 2 == 0,
    ],
)]
fn calculate_even_result(output: i32) -> i32 { output + 2 }
```

**4. Beyond Names: Destructuring Return Values**

The `binds` parameter also lets you destructure return values, making complex postconditions easier to read and write. You can use any valid Rust pattern, including tuple patterns, struct patterns, or even more complex nested patterns.

```rust, no_run
use anodized::spec;

#[spec(
    // Destructure the returned tuple into `(a, b)`.
    binds: (a, b),
    // Postconditions can now use the bound variables `a` and `b`.
    ensures: [
        a <= b,
        // They can also reference the arguments.
        (a, b) == pair || (b, a) == pair,
    ],
)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) { 
    if pair.0 <= pair.1 { pair } else { (pair.1, pair.0) }
}
```

### Example With All Specification Parameters

```rust, no_run
use anodized::spec;

#[spec(
    requires: *balance >= amount,
    clones: [
        *balance as initial_balance,
    ],
    binds: (new_balance, receipt_amount),
    ensures: [
        new_balance == initial_balance - amount,
        receipt_amount == amount,
        *balance == new_balance,
    ],
)]
fn withdraw(balance: &mut u64, amount: u64) -> (u64, u64) { 
    *balance -= amount;
    (*balance, amount)
}
```

### Why Conditions Are Rust Expressions

A core design principle of Anodized is that a condition is written as a **standard Rust expression** that evaluates to `bool`. This is a deliberate choice that provides key benefits over using a custom language.

- **The Language You Already Know**: No need to learn yet another language to write the conditions. Write them in the one you already know: standard Rust. Call functions, use macros (like `matches!`), or write `if` and `match` expressions, and so on. As long as it all evaluates to a `bool`, it's good to go.

- **An Integral Part of Your Code**: Conditions aren't special comments or strings; they are real Rust expressions, fully integrated with your code. The Rust compiler checks every condition for syntax and type errors, just like any other part of your code. If you misspell a variable, compare incompatible types, or make any other mistake, you'll get a familiar compiler error pointing directly to the condition that needs fixing.

## Why "Spec" Instead of "Contract"

The choice of "specification" (or "spec") over "contract" is deliberate. While Design by Contract has a rich history, the term "contract" is now strongly associated with blockchain. This is particularly true in Rust, which has become a leading language for smart contract development.

This naming collision hurts discoverability. Searching for "Rust contract" yields blockchain results, not correctness tools.

Using "specification" instead:

- **Improves discoverability**: Developers find correctness tools when searching for them.
- **Reduces confusion**: The distinction from blockchain is immediately clear.
- **Maintains clarity**: "Specification" accurately describes these formal behavior annotations.

The term "spec" is already familiar from test specs, API specs, and formal specifications. It conveys the same meaning as Design by Contract while avoiding modern ambiguity.

## Prior Art and Motivation

The idea of adding contracts to Rust isn't new, and Anodized builds upon the great work and ideas from several other projects and discussions in the community. It is a fresh take with a strong focus on ergonomics and a forward-looking vision for an integrated ecosystem.

**The `contracts` Crate**

The most direct and popular predecessor is the [`contracts`](https://crates.io/crates/contracts) crate. It is a mature and feature-rich library that also provides `#[requires]`, `#[ensures]`, and `#[invariant]` attributes. It has been a major inspiration for Anodized.

Anodized differentiates itself with a few key design choices:

- **Unified Attribute**: Anodized uses a single, comprehensive `#[spec]` attribute to group all conditions for a function, presenting the entire specification as one cohesive block.

- **Ergonomic Focus**: The design process has been heavily focused on refining the user-facing syntax (e.g. keyword choices, return value binding) to be as intuitive, approachable, and powerful as possible.

- **Ecosystem Vision**: While `contracts` is an excellent tool for runtime checking, Anodized is designed from the ground up to be a foundational layer for a wider ecosystem of diverse correctness tools, from fuzzing to formal verification.

**Other Crates**

Older crates like `libhoare` (a compiler plugin from before procedural macros were stabilized) and `dbc` explored similar ideas, proving the long-standing interest in Design by Contract within the Rust community. Anodized benefits from the modern procedural macro system, which allows for much better integration with the compiler and toolchain.

**Inspiration from Other Languages**

Anodized is also inspired by languages where contracts are a first-class feature, not just a library. Languages like [Whiley](https://whiley.org), [Eiffel](https://eiffel.org), and [Ada/SPARK](https://adacore.com/about-spark) demonstrate the power of deeply integrating formal specifications into the syntax, type system, and toolchain. The Anodized ecosystem begins with one library, but shares the great ambition of those languages: to bring a similar level of integration and ergonomic feel to Rust.

**Towards First-Class Contracts for Rust**

There have been official discussions and RFCs within the Rust project itself about adding native support for contracts to the language. Anodized is designed to be a practical, library-based solution that can be used **today**, while also serving as a testbed for ideas that could inform future language-level features.

## License

Anodized is distributed under the terms of the MIT License and the Apache License (Version 2.0). Users can choose either license, and contributors must license their changes under both.

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## Technical Documentation

For detailed technical documentation including the formal specification grammar and runtime check implementation details, see the [`anodized-core`](https://docs.rs/anodized-core) documentation.
