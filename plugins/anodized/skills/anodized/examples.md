# Anodized examples

Correct, compiling specs for every supported item kind. Copy the shape closest to your case.
Every Rust block here is compiled by the workspace's test suite.

## Free function

```rust
use anodized::spec;

#[spec(
    requires: [!slice.is_empty(), index < slice.len()],
    ensures: |output| output == slice[index],
)]
fn nth(slice: &[u32], index: usize) -> u32 {
    slice[index]
}
```

## Capturing an entry-time value

```rust
use anodized::spec;

#[spec(
    requires: counter < u32::MAX,
    captures: before = counter,
    ensures: |output| output == before + 1,
)]
fn bump(counter: u32) -> u32 {
    counter + 1
}
```

Several captures, and a clone for a non-`Copy` value:

```rust
use anodized::spec;

#[spec(
    captures: [first = items.first().copied(), size = items.len()],
    ensures: |ref output| [output.len() == size, output.last().copied() == first],
)]
fn reversed(items: Vec<u32>) -> Vec<u32> {
    items.into_iter().rev().collect()
}
```

## Invariant on a mutable argument

```rust
use anodized::spec;

#[spec(maintains: !buffer.is_empty())]
fn overwrite_first(buffer: &mut Vec<u32>, value: u32) {
    buffer[0] = value;
}
```

## Destructuring the return value

```rust
use anodized::spec;

#[spec(
    requires: divisor != 0,
    ensures: |(quotient, remainder)| remainder < divisor && quotient * divisor + remainder == dividend,
)]
fn divide(dividend: u32, divisor: u32) -> (u32, u32) {
    (dividend / divisor, dividend % divisor)
}
```

## Returning a `Result`

A non-`Copy` return value binds by reference:

```rust
use anodized::spec;

#[spec(ensures: |ref output| output.is_ok() == (divisor != 0))]
fn checked_divide(dividend: u32, divisor: u32) -> Result<u32, String> {
    if divisor == 0 {
        return Err("division by zero".to_owned());
    }
    Ok(dividend / divisor)
}
```

## Several postconditions under one binding

```rust
use anodized::spec;

#[spec(ensures: |output| [output >= left, output >= right, output == left || output == right])]
fn larger(left: u32, right: u32) -> u32 {
    if left > right { left } else { right }
}
```

## Qualifiers

```rust
use anodized::spec;

#[spec(
    pure,
    requires: exponent < 32,
    ensures: |output| output >= 1,
)]
fn power_of_two(exponent: u32) -> u32 {
    1 << exponent
}
```

## Inherent `impl`

The `impl` carries a bare attribute; the clauses live on the methods.

```rust
use anodized::spec;

struct Counter {
    value: u32,
}

#[spec]
impl Counter {
    #[spec(ensures: |ref output| output.value == 0)]
    fn new() -> Self {
        Self { value: 0 }
    }

    #[spec(
        requires: self.value < u32::MAX,
        captures: before = self.value,
        ensures: |output| output == before + 1,
    )]
    fn increment(&mut self) -> u32 {
        self.value += 1;
        self.value
    }
}
```

## Trait, with a narrowing impl

```rust
use anodized::spec;

#[spec]
trait Scale {
    #[spec(
        requires: factor > 0 && value <= u32::MAX / factor,
        ensures: |output| output >= value,
    )]
    fn scale(&self, value: u32, factor: u32) -> u32;
}

struct Doubling;

#[spec]
impl Scale for Doubling {
    fn scale(&self, value: u32, factor: u32) -> u32 {
        value * factor
    }
}

struct Exact;

#[spec]
impl Scale for Exact {
    #[spec(ensures: |output| output == value * factor)]
    fn scale(&self, value: u32, factor: u32) -> u32 {
        value * factor
    }
}
```

`Exact` narrows: it promises more than the trait did. An impl may never promise less.

The second condition is doing real work. Without `value <= u32::MAX / factor` the trait would
accept inputs whose product does not fit in a `u32`, and the natural multiplying implementation
cannot honor `output >= value` for them — it overflows first. (A saturating implementation
could, which is the point: a precondition defines what every implementation must cope with.) A
precondition that admits inputs the body cannot serve is a spec that is false of its own code.

Note that it is one `&&` condition rather than two list entries. Conditions in a list are
checked independently and do not short-circuit, so a separate `value <= u32::MAX / factor`
would divide by zero while being checked whenever `factor` is `0` — a spec that panics during
its own evaluation. When one condition guards another, join them with `&&`.

## `while` loop

```rust
use anodized::spec;

#[spec(requires: !items.is_empty())]
fn maximum(items: &[u32]) -> u32 {
    let mut best = items[0];
    let mut index = 1;
    #[spec(
        maintains: index <= items.len(),
        decreases: items.len() - index,
    )]
    while index < items.len() {
        if items[index] > best {
            best = items[index];
        }
        index += 1;
    }
    best
}
```

## `for` loop

```rust
use anodized::spec;

#[spec(ensures: |output| output <= items.len())]
fn count_zeros(items: &[u32]) -> usize {
    let mut count = 0usize;
    #[spec(maintains: count <= items.len())]
    for item in items {
        if *item == 0 {
            count += 1;
        }
    }
    count
}

fn main() {
    assert_eq!(count_zeros(&[]), 0);
    assert_eq!(count_zeros(&[0, 0]), 2);
    assert_eq!(count_zeros(&[1, 0, 3]), 1);
}
```

Note what the bound avoids. An upper bound like `u64::from(u32::MAX) * items.len() as u64`
reads naturally for a sum, but the condition itself would overflow on a long enough slice, and
a spec that can panic while being checked is worse than none. `count <= items.len()` cannot.

The `fn main` is not decoration. Compiling an example proves its conditions are well-formed;
only running one proves they are true of the code beside them. Exercise the boundaries — here
the empty slice and the all-zero slice, which are exactly where a plausible-looking lower bound
on the sum would have been false.

## `struct` refinement

```rust
use anodized::spec;

#[spec(maintains: self.low <= self.high)]
struct Bounds {
    low: u32,
    high: u32,
}
```

## `enum` refinement

Variants are in scope inside the condition.

```rust
use anodized::spec;

#[spec(maintains: match self { Temperature::Celsius(degrees) => *degrees >= -273, Temperature::Kelvin(degrees) => *degrees >= 0 })]
enum Temperature {
    Celsius(i32),
    Kelvin(i32),
}
```

## A condition compiled only for tests

```rust
use anodized::spec;

#[spec(
    requires: !items.is_empty(),
    #[cfg(debug_assertions)]
    ensures: |output| items.contains(&output),
)]
fn first(items: &[u32]) -> u32 {
    items[0]
}
```

## Using `implies!`

```rust
use anodized::spec;
use anodized::logic::implies;

#[spec(ensures: |output| implies!(divisor != 0, output <= dividend))]
fn safe_divide(dividend: u32, divisor: u32) -> u32 {
    if divisor == 0 { 0 } else { dividend / divisor }
}
```
