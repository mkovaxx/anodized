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

| `--cfg` Setting                                     | Effect                  |
| --------------------------------------------------- | ----------------------- |
| [`anodized_discard_specs`](#anodized_discard_specs) | disable spec embedding  |
| [`anodized_panic`](#anodized_panic)                 | runtime check: panic    |
| [`anodized_print`](#anodized_print)                 | runtime check: print    |
| [`anodized_try`](#anodized_try)                     | runtime check: `Result` |

Select the desired options via compiler `cfg` flags, for example:

```bash
RUSTFLAGS="--cfg anodized_print" cargo test
```

### `anodized_discard_specs`

Disable embedding the specs as Rust code.

**Important:** This is the only configuration in which a spec costs exactly nothing. Conditions are dead code whenever checks are disabled, but `captures` expressions are bound alongside the body and so are evaluated on every call regardless — a `.clone()` in a capture is a real clone. Discarding specs also **prevents syntax/type checking** them, so it may decrease compilation time.

## Runtime Checks

To disable runtime checks completely, run without any `anodized_*` options.

### `anodized_panic`

Checks each condition via an `assert!`, so a violation panics with a descriptive message.

### `anodized_print`

Reports each violation with `eprintln!`, so execution can continue. Useful for experiments, logging, etc.

### `anodized_try`

Enables the `try_call!` macro, which checks conditions but defers action to the caller by
returning a `Result`. See the `anodized::result` module for details.

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
    ensures: |ref output| output.is_ok(),
)]
fn perform_complex_operation(input: i32) -> Result<i32, String> { todo!() }
```

The `#[cfg]` attribute follows standard Rust semantics: when the configuration predicate is false, the runtime check for the condition is completely omitted. This holds in every build configuration that runs checks, whether or not `anodized_print` is enabled.

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

You can include any number of each flavor. Multiple conditions of the same flavor are combined with a logical **AND** (`&&`).

For convenience, `<conditions>` can be either a single condition or a group (i.e. `[<condition>, <condition>, ...]`).

A condition is a `bool`-valued Rust expression; as simple as that. This is a non-trivial design choice, so its benefits are explained in the section below: [Why Conditions Are Rust Expressions](#why-conditions-are-rust-expressions).

**Note** that each condition behaves as an [`Fn() -> bool`](https://doc.rust-lang.org/std/ops/trait.Fn.html)
closure, which means that it _cannot_ mutate the function's inputs, captured values (see below),
or the function's output (for a postcondition).

The conditions must be given in the following order: `requires`, `maintains`, and `ensures`. This order is enforced to mirror the logical flow of a function's execution: preconditions (`requires`) are checked upon entry, invariants (`maintains`) must hold true upon both entry and exit, and postconditions (`ensures`) are checked upon exit.

```rust, no_run
use anodized::spec;

#[spec(
    // Precondition: the vector must have room for at least one more element
    requires: vec.len() < vec.capacity() || vec.capacity() == 0,
    // Invariant: length never exceeds capacity
    maintains: vec.len() <= vec.capacity(),
)]
fn push_checked<T>(vec: &mut Vec<T>, value: T) { vec.push(value) }
```

### `captures`: Capture Entry-Time Values

Sometimes postconditions need to compare the function's final state with its initial state. The `captures` field lets you capture values at function entry for use in postconditions.

```rust, no_run
use anodized::spec;

#[spec(
    requires: !items.is_empty(),
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

### Bind the Return Value in `ensures` Closures

When you write a **postcondition** (`ensures`) as a plain `bool` expression, it does not have access to the function's return value.

To give it access to the function's return value, write the condition as a closure.

```rust, no_run
use anodized::spec;

#[spec(
    ensures: |output| output > 0,
)]
fn get_positive_value() -> i32 { todo!() }
```

An output pattern can bind or destructure a return value directly, including a non-`Copy` value.
For move patterns, Anodized reconstructs the return value after each postcondition, so it can still
be returned to the caller. Use a reference pattern such as `|ref output|` when the postcondition
is naturally expressed in terms of a borrow.

Output patterns must be irrefutable. A pattern cannot mix move and `ref` bindings. A pattern that
contains `..` may bind other values only by `ref`, because the omitted values cannot be
reconstructed.

**1. Per-Group Binding**: Use the following shorthand form to set a binding for an entire group of postconditions.

```rust, no_run
use anodized::spec;

#[spec(
    requires: input < i32::MAX - 1,
    // Bind the return value to `output` for the entire group of postconditions.
    ensures: |output| [
        output > input,
        output % 2 == 0,
    ],
)]
fn calculate_larger_even_result(input: i32) -> i32 { todo!() }
```

**2. Beyond Names: Destructure Return Values**

Bindings also let you destructure return values, making complex postconditions easier to read and
write. You can use irrefutable tuple, struct, slice, and nested patterns, subject to the pattern
rules above.

```rust, no_run
use anodized::spec;

#[spec(
    // Destructure the returned tuple into `(a, b)`.
    ensures: |(a, b)| [
        // Postconditions can now use the bound variables `a` and `b`.
        a <= b,
        // They can also reference the inputs.
        (a, b) == pair || (b, a) == pair,
    ],
)]
fn sort_pair(pair: (i32, i32)) -> (i32, i32) { todo!() }
```

**5. Example With All Function Spec Fields**

```rust, no_run
use anodized::spec;

#[spec(
    requires: amount >= 0 && *balance >= amount,
    maintains: *balance >= 0,
    captures: initial_balance = *balance,
    ensures: |(new_balance, receipt_amount)| [
        new_balance == initial_balance - amount,
        receipt_amount == amount,
        *balance == new_balance,
    ],
)]
fn withdraw(balance: &mut i64, amount: i64) -> (i64, i64) { todo!() }
```

### Loop Specs

Anodized supports specs on loops to ensure correctness and bounded iteration.

Loop specs support the following fields:

- `maintains`: Loop invariants that must hold both before and after each iteration.
- `decreases`: A loop variant expression that shows strict progress toward termination.

**On a `for` Loop**

```rust, no_run
use anodized::spec;

#[spec(
    requires: !seq.is_empty(),
    ensures: |output| [
        seq.iter().any(|elem| *elem == output),
        seq.iter().all(|elem| *elem <= output),
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
    ensures: |output| [
        output <= seq.len(),
        seq[0..output].iter().all(|item| item < value),
        seq[output..].iter().all(|item| item >= value),
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
        requires: self.current() < i32::MAX,
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
enum MonotonicVec<T: Ord> {
    Ascending(Vec<T>),
    Descending(Vec<T>),
}
```

Important restrictions:

- Runtime checks are **not implemented** yet.
- Only the `maintains` spec field is supported.
