# The Anodized Reference

## `spec` Support

| Program Element                                  | `spec` Features                      |
| ------------------------------------------------ | ------------------------------------ |
| [`fn`, free or inherent `impl`](#function-specs) | Pre- and postconditions, invariants. |
| [`trait`](#trait-specs)                          | Enforces each `impl` to conform.     |
| [`for` and `while`](#loop-specs)                 | Loop invariants and variant (bound). |
| [`struct` and `enum`](#data-specs)               | Refinements to constrain instances.  |

## Build Configurations

Anodized uses `cfg` options to control how each `#[spec]` changes the Rust code.

| `--cfg` Setting                                     | Effect                 |
| --------------------------------------------------- | ---------------------- |
| [`anodized_discard_specs`](#anodized_discard_specs) | disable spec embedding |
| [`anodized_panic`](#anodized_panic)                 | runtime check: panic   |
| [`anodized_print`](#anodized_print)                 | runtime check: print   |

Select the desired options via compiler `cfg` flags, for example:

```bash
RUSTFLAGS="--cfg anodized_print" cargo test
```

### `anodized_discard_specs`

Disable embedding the specs as Rust code.

**Important:** This has **no effect on runtime performance** because the embedded specs are always dead code. On the other hand, it **prevents syntax/type checking** specs, so it may decrease compilation time.

## Runtime Checks

To disable runtime checks completely, run without any `anodized_*` options.

### `anodized_panic`

Checks each condition via an `assert!`, so a violation panics with a descriptive message.

### `anodized_print`

Reports each violation with `eprintln!`, so execution can continue. Useful for experiments, logging, etc.

Use `#[cfg]` attributes on individual conditions to control when checks run (see the [#[cfg] section](#cfg-configure-runtime-checks) below).

**Important:** Even when a condition's runtime check is disabled via a `#[cfg]` build setting, the compiler still validates that condition at compile time for syntax errors, unknown identifiers, type mismatches, etc.

### Fine-Grained Control

When runtime checks are enabled, use the standard `#[cfg]` attribute to select build configurations under which a condition is checked.

```rust, no_run
use anodized::spec;

#[spec(
    // Runtime checks only during `cargo test`.
    #[cfg(test)]
    requires: input > 0,

    // Runtime checks only in debug builds (like `debug_assert!`)
    #[cfg(debug_assertions)]
    ensures: output.is_ok(),
)]
fn perform_complex_operation(input: i32) -> Result<i32, String> { todo!() }
```

The `#[cfg]` attribute follows standard Rust semantics: when the configuration predicate is false, the runtime check for the condition is completely omitted.

**Important:** Anodized guarantees that each condition remains syntactically valid and type-correct regardless of its `#[cfg]` settings. This prevents conditions from becoming invalid between different build configurations, and keeps the entire spec always visible to analysis tools.

**Common Patterns:**

- `#[cfg(debug_assertions)]`: Check only in debug builds (like `debug_assert!`).
- `#[cfg(test)]`: Check only during testing.
- No `#[cfg]`: Always check (like `assert!`).

## Function Specs

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

### `captures`: Capture Entry-Time Values

Sometimes postconditions need to compare the function's final state with its initial state. The `captures` field lets you capture values at function entry for use in postconditions.

```rust, no_run
use anodized::spec;

#[spec(
    captures: [
        // Copy types: captured directly
        orig_len = items.len(),
        // Non-Copy types: use .clone() explicitly
        orig_items = items.clone(),
    ],
    ensures: [
        items.len() == orig_len + 1,
        items[0] == orig_items[0],
    ],
)]
fn add_item<T: Clone + Eq>(items: &mut Vec<T>, item: T) { todo!() }

// A capture assignment may use a pattern to destructure tuples, structs, arrays, and other composite types:
#[spec(
    captures: (first, second, third) = triple,
    ensures: [
        first == triple.0,
        second == triple.1,
        third == triple.2,
    ],
)]
fn match_tuple(triple: (bool, char, i32)) { todo!() }
```

- **Capture assignments** bind a name or pattern on the left to an entry-time expression on the right, e.g. `orig_len = self.items.len()`.
- **Patterns** may be used to destructure the captured value, e.g. `Person { name, age } = person.clone()`.
- **No automatic cloning**: Each captured expression is **moved**. For a `Copy` type, a copy is made implicitly. For a non-`Copy` type, you must explicitly use `.clone()`, `.to_owned()`, or another appropriate method.
- Capturing happens **after** preconditions are checked but **before** the function body executes.
- The captured values are **only** available to postconditions, not to preconditions or the function body itself.

### `inspects`: Bind the Return Value

In **postconditions** (`ensures`), you can refer to the function's return value by the default name `output`.

```rust, no_run
use anodized::spec;

#[spec(
    ensures: *output > 0,
)]
fn get_positive_value() -> i32 { todo!() }
```

**Note** that a postcondition is a closure that takes the function's return value by reference. When you write a postcondition as a "naked" expression `<EXPR>`, that is shorthand for `|<PATTERN>| <EXPR>`, where `<PATTERN>` is the spec-wide binding. In error messages, a postcondition is always displayed as a closure to make it clear (e.g. `| output | *output > 0`).

The default spec-wide binding is `output`. If that collides with an existing identifier, you can choose a different name for it in two ways:

**1. Spec-Wide Binding**: Use the `inspects` field to set a new name for the return value across all postconditions in the specification. It must be placed immediately before any `ensures` conditions.

```rust, no_run
use anodized::spec;

#[spec(
    inspects: new_value,
    ensures: *new_value > old_value,
)]
fn increment(old_value: i32) -> i32 { todo!() }
```

**2. Explicit Binding**: Write the postcondition with an explicit binding, i.e. as a closure `|<PATTERN>| <EXPR>`. This has the highest precedence and affects only that single condition.

```rust, no_run
use anodized::spec;

#[spec(
    ensures: [
        // This postcondition uses the default binding.
        output.is_ascii(),
        // This postcondition binds the output as `c`.
        |c| c.is_digit(16),
    ],
)]
fn create_data() -> char { todo!() }
```

**3. Binding Precedence**: The explicit binding takes precedence; same as in Rust. Plain postconditions still use the spec-wide binding.

```rust, no_run
use anodized::spec;

// A function where 'output' is an argument name, requiring a different name.
#[spec(
    // Set a spec-wide binding for the return value: `result`.
    inspects: result,
    ensures: [
        // This postcondition uses the spec-wide binding: `result`.
        *result > output,
        // This postcondition uses an explicit binding: `val`.
        |val| *val % 2 == 0,
    ],
)]
fn calculate_even_result(output: i32) -> i32 { todo!() }
```

**4. Beyond Names: Destructuring Return Values**

Bindings also lets you destructure return values, making complex postconditions easier to read and write. You can use any valid Rust pattern, including tuple patterns, struct patterns, or even more complex nested patterns.

```rust, no_run
use anodized::spec;

#[spec(
    // Destructure the returned tuple into `(a, b)`.
    inspects: (a, b),
    // Postconditions can now use the bound variables `a` and `b`.
    ensures: [
        a <= b,
        // They can also reference the arguments.
        (*a, *b) == pair || (*b, *a) == pair,
    ],
)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) { todo!() }
```

**5. Example With All Function Spec Fields**

```rust, no_run
use anodized::spec;

#[spec(
    requires: *balance >= amount,
    maintains: *balance >= 0,
    captures: initial_balance = *balance,
    inspects: (new_balance, receipt_amount),
    ensures: [
        *new_balance == initial_balance - amount,
        *receipt_amount == amount,
        *balance == *new_balance,
    ],
)]
fn withdraw(balance: &mut u64, amount: u64) -> (u64, u64) { todo!() }
```

### Loop Specs

Anodized supports specs on loops to ensure correctness and bounded iteration.

Loop specs support the following elements:

- `maintains`: Loop invariants that must hold both before and after each iteration.
- `decreases`: A loop variant expression that shows strict progress toward termination.

**On a `for` Loop**

```rust, no_run
use anodized::spec;

#[spec(
    ensures: [
        seq.iter().any(|elem| elem == output),
        seq.iter().all(|elem| elem <= output),
    ],
)]
fn find_maximum(seq: &[u8]) -> u8 {
    let mut max = 0;

    #[spec(
        maintains: seq[0..i].iter().all(|elem| elem <= &max),
    )]
    for i in 0..seq.len() {
        if seq[i] > max {
            max = seq[i]
        }
    }

    max
}
```

**On a `while` Loop**

```rust, no_run
use anodized::spec;

#[spec(
    requires: seq.is_sorted(),
    ensures: [
        *output <= seq.len(),
        seq[0..*output].iter().all(|item| item < value),
        seq[*output..].iter().all(|item| item >= value),
    ],
)]
fn find_insert_position<T: Ord>(seq: &[T], value: &T) -> usize {
    let mut i = 0;

    #[spec(
        maintains: seq[0..i].iter().all(|item| item < value),
        decreases: seq.len() - i,
    )]
    while i < seq.len() && seq[i] < *value {
        i += 1;
    }

    i
}
```

Important restrictions:

- The **containing function** must have a `#[spec]` attribute.
- Runtime checking loop specs is **planned but not yet implemented**.

### Trait Specs

Anodized supports specs on trait methods, which automatically constrain all implementations.

Use the following structure:

1. Put `#[spec]` on the trait.
2. Put method-level `#[spec(...)]` on trait methods that define requirements.
3. Put `#[spec]` on each corresponding trait `impl`.
4. (Optional) Put `#[spec(...)]` on impl `fn`s to narrow the trait's spec.

```rust, no_run
use anodized::spec;

#[spec]
trait MonotonicGenerator {
    fn current(&self) -> i32;

    #[spec(
        captures: old_val = self.current(),
        ensures: self.current() > old_val,
    )]
    fn update(&mut self);
}

struct Counter(i32);

#[spec]
impl MonotonicGenerator for Counter {
    fn current(&self) -> i32 {
        self.0
    }

    fn update(&mut self) {
        self.0 += 1;
    }
}
```

Important restrictions:

- The trait-level (or impl-level) `#[spec]` is an enabler; specification clauses belong on `fn`s, not on the trait (or impl) itself.
- Only a `fn` item may have a spec, other trait items (`const`, `type`, etc.) are not supported.
- A spec on an impl `fn` must **narrow** the spec of the trait `fn`. This is a consequence of the [Liskov substitution principle](https://en.wikipedia.org/wiki/Liskov_substitution_principle).
  - Runtime checks enforce narrowing.
  - Static analyzers **must validate** narrowing as part of verification.
- Names prefixed with `__anodized_` are internal and must not be implemented directly.

### Data Specs

Anodized supports specs on data types, meant to constrain all instances. This capability is equivalent to refinement types.

**On a Struct**

```rust, no_run
use anodized::spec;

#[spec(maintains: self.a.pow(2) + self.b.pow(2) == self.c.pow(2))]
struct PythagoreanTriple {
    a: u32,
    b: u32,
    c: u32,
}

#[spec(maintains: !self.0.is_empty())]
struct NonEmptyVec<T>(Vec<T>);

#[spec(maintains: self.0.iter().rev().eq(&self.0))]
struct PalindromeVec<T: Eq>(Vec<T>);
```

**On an Enum**

```rust, no_run
use anodized::spec;

#[spec(
    maintains: match self {
        Ascending(vec) => vec.is_sorted(),
        Descending(vec) => vec.iter().rev().is_sorted(),
    }
)]
#[allow(unused)]
enum MonotonicVec<T: Ord> {
    Ascending(Vec<T>),
    Descending(Vec<T>),
}
```

Important restrictions:

- Runtime checks are **not implemented** yet.
- Only the `maintains` spec field is supported.
