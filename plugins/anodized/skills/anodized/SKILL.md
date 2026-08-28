---
name: anodized
description: >
  Write, review, and repair Anodized `#[spec]` specifications in Rust: preconditions
  (`requires`), invariants (`maintains`), entry-time snapshots (`captures`), postconditions
  (`ensures`), loop invariants and variants (`decreases`), refinements on `struct` and `enum`,
  and the `fn` qualifier lattice. Use when adding or reviewing specs in a crate that depends
  on `anodized`, when a `#[spec(...)]` fails to compile, when a precondition or postcondition
  fires at runtime, when specifying a trait and its impls, or when choosing a
  `--cfg anodized_*` build. Anodized has no `old()` and no implicit `output` binding, and its
  0.6 syntax broke from 0.5; this skill supplies the current forms.
when_to_use: >
  Trigger on: `#[spec]`, anodized, requires:, ensures:, maintains:, captures:, decreases:,
  precondition, postcondition, loop invariant, design by contract, add a contract to this
  function, specify this trait, narrow a spec on an impl. Also: old(), "the old value", result
  in a postcondition, old_x, "expression as pattern", binds:, inspects:, E0596 or E0425 inside
  a spec, "qualifier is redundant", "qualifiers cannot be weaker", "unknown spec field",
  "fields are out of order". Also migrating from contracts, Creusot, Prusti, Kani, or Verus.
  Also anodized_panic, anodized_print, anodized_try, anodized_discard_specs, try_call!,
  anodized-fmt, implies!, opaque!.
allowed-tools: Bash(cargo:*), Bash(anodized-fmt:*)
---

<!-- anodized-skill -->

# Writing Anodized specifications

A spec is ordinary Rust: every condition is a `bool` expression, and there is no separate
specification language to learn. Conditions on functions and on `struct` definitions are
type-checked by the compiler like any other code. Conditions on **loops and `enum`
definitions are not** — as of 0.6 they are parsed and then dropped, so a typo in one compiles
silently. Write those with extra care. What has to
be learned is the shape of the attribute, and the handful of places where that shape differs
from what every other contract system has trained you to expect.

## Four things that are not what you expect

**1. There is no `old()`.** Entry-time values come from `captures`, which are visible only to
postconditions.

```rust,compile_fail
// EXPECT: E0425
use anodized::spec;

#[spec(ensures: |output| output == old(count) + 1)]
fn increment(count: u32) -> u32 {
    count + 1
}
```

```rust
use anodized::spec;

#[spec(
    requires: count < u32::MAX,
    captures: before = count,
    ensures: |output| output == before + 1,
)]
fn increment(count: u32) -> u32 {
    count + 1
}
```

**2. There is no implicit `output` or `result`.** A bare `ensures` expression cannot see the
return value at all; bind it with a closure.

```rust,compile_fail
// EXPECT: E0425
use anodized::spec;

#[spec(ensures: output > 0)]
fn one() -> u32 {
    1
}
```

```rust
use anodized::spec;

#[spec(ensures: |output| output > 0)]
fn one() -> u32 {
    1
}
```

**3. Conditions cannot mutate anything.** Each is coerced to `Fn() -> bool`, so touching a
`&mut` inside one is a borrow error, not a runtime surprise.

```rust,compile_fail
// EXPECT: E0596
use anodized::spec;

#[spec(requires: buffer.pop().is_some())]
fn drain(buffer: &mut Vec<u32>) {
    buffer.clear();
}
```

**4. Anodized 0.6 broke from 0.5.** If you remember this crate at all, you probably remember
the old syntax. `old_x` bindings are gone, `captures` flipped to `pattern = expression`, and
output bindings moved into the `ensures` closure.

```rust,compile_fail
// EXPECT: E0425
use anodized::spec;

#[spec(ensures: |output| output == old_count + 1)]
fn increment(count: u32) -> u32 {
    count + 1
}
```

| About to write | Write instead |
| --- | --- |
| `old(x)` or `old_x` | `captures: old_x = x` and read it in `ensures` |
| `ensures: output > 0` | `ensures: \|output\| output > 0` |
| `captures: x.len() as n` | `captures: n = x.len()` |
| `binds:` / `inspects:` | `ensures: \|PAT\| [EXPR, ...]` |
| `#[requires(...)]` / `#[ensures(...)]` | one `#[spec(requires: ..., ensures: ...)]` |

## When to add a spec

Add one when at least one is true: the function is `pub` or crosses a module boundary; its
inputs have a non-trivial valid domain; its output has a guaranteed property the type does not
state; or it holds a loop whose correctness rests on a non-obvious invariant.

Do **not** add a clause that restates the type system. `requires: [n >= 0]` on a `u32` is
noise, and so is a postcondition asserting the return type. A spec that adds no information
costs the reader attention and buys nothing. Existing `assert!`, `panic!`, and early
`return Err` are preconditions already written in another form — those are the ones worth
lifting into `requires`.

When a function genuinely has nothing to state, a bare `#[spec]` is still worth adding. It
records that the contract is deliberately tautological — equivalent to `requires: true` and
`ensures: true` — which is not the same as nobody having considered it. It is also what
enables loop specs inside the body, so it is not purely decorative.

Not every function can carry one, though. The macro evaluates the body inside a closure, so
any shape the language will not let cross a closure boundary stops compiling the moment a spec
is attached — in every configuration except `anodized_discard_specs`, which drops specs before
they expand. Known cases: a function returning `impl Trait` (`E0562`), a `const fn` (`E0015`),
and a function returning `&mut` borrowed from a parameter (`captured variable cannot escape
`FnMut` closure body`, which carries no error code). Returning a shared `&` is fine.

Treat that list as incomplete rather than exhaustive. The rule that generalizes is the closure:
if attaching a spec makes a function stop compiling, that shape cannot carry one, and the only
repair is to remove the attribute. None of these have a workaround.

On a function nested inside a spec'd `trait` or `impl`, an empty spec is written `#[spec()]`,
not `#[spec]`. The attribute on a nested function is read with `parse_args`, which needs the
parentheses even when there is nothing between them; a bare one is rejected with `expected
attribute arguments in parentheses`. This applies to inherent impls, traits, and trait impls
alike. Omitting the attribute entirely is also valid there and means the same empty spec — the
parentheses only matter once you write the attribute down.

Preconditions describe what callers **must** uphold. Recoverable failures are still `Result`;
a spec is not an error-handling mechanism.

## Clause grammar

Clauses appear in a fixed order, and the order is enforced:

```rust,ignore
// fragment
#[spec(
    pure,                                   // qualifiers, bare
    requires: [condition, condition],       // preconditions
    maintains: condition,                   // invariants, checked at entry and exit
    captures: [name = expression],          // entry-time values, at most one field
    ensures: |output| [condition],          // postconditions
)]
```

<!-- anodized:generated:clauses -->
| Clause | Takes | Allowed on |
| --- | --- | --- |
| `requires` | one condition, or a list | `fn` |
| `maintains` | one condition, or a list | `fn`, loop, `struct`, `enum` |
| `captures` | `pattern = expression`, or a list; at most one field | `fn` |
| `ensures` | a condition, `\|pattern\| condition`, or a list of either | `fn` |
| `decreases` | one expression; at most one field | loop only |
<!-- /anodized:generated:clauses -->

`requires`, `maintains`, and `ensures` may be repeated; the conditions accumulate. Prefer
several small clauses over one `&&` chain — a runtime failure names the clause it broke, so
smaller clauses give sharper messages. A trailing comma is optional everywhere.

## `captures`: entry-time values

At most **one** `captures` field per spec; use the list form for several. The right-hand side
is **moved**, so a non-`Copy` value needs an explicit `.clone()`. Captures are evaluated after
`requires` and before the body, and they are in scope **only** for postconditions — reading
one from `requires` or from the body is `E0425`.

```rust
use anodized::spec;

#[spec(
    captures: [smallest = items.iter().copied().min(), count = items.len()],
    ensures: |ref output| [output.len() == count, output.first().copied() == smallest],
)]
fn sorted(mut items: Vec<u32>) -> Vec<u32> {
    items.sort_unstable();
    items
}
```

## `ensures`: binding the return value

Four forms are accepted:

| Form | Return value |
| --- | --- |
| `ensures: expr` | not bound |
| `ensures: \|pat\| expr` | bound by `pat` |
| `ensures: \|pat\| [a, b]` | bound by `pat` for the whole group |
| `ensures: [a, \|pat\| b]` | bound only inside that element |

A pattern may bind or destructure the return value directly, non-`Copy` included: a move
pattern is reconstructed after each postcondition, so the value still reaches the caller.
`|ref output|` is for when the condition reads more naturally as a borrow, not a workaround.

Three rules the pattern must satisfy. It must be irrefutable — `|Ok(v)|` is rejected, so
inspect a `Result` with `|output| output.is_ok()`. It may not mix move and `ref` bindings. And
a pattern containing `..` may bind the rest only by `ref`, since what it omits cannot be
reconstructed.

## Qualifiers

<!-- anodized:generated:qualifiers -->
| Qualifier | Guarantees |
| --- | --- |
| `functional` | `deterministic`, `effectfree`, `infallible`, `terminating` |
| `pure` | `deterministic`, `effectfree` |
| `total` | `infallible`, `terminating` |
| `deterministic` | the return value depends only on the arguments |
| `effectfree` | no side effects |
| `infallible` | does not panic or abort |
| `terminating` | does not run forever |
<!-- /anodized:generated:qualifiers -->

Write them bare, before every value clause. Naming a composite together with one of its own
components is a hard error, not a warning: `pure, deterministic` is redundant because `pure`
already means `deterministic` and `effectfree`. Claim only what you can defend.

## Item kinds

<!-- anodized:generated:item-kinds -->
| Item | Clauses | How |
| --- | --- | --- |
| free `fn` | all clauses | `#[spec(...)]` on the `fn` |
| inherent `impl` | all clauses, per method | bare `#[spec]` on the `impl`, clauses on each `fn` |
| `trait` | all clauses, per method | bare `#[spec]` on the `trait`, clauses on each `fn` |
| `impl Trait for T` | all clauses, narrowing only | bare `#[spec]` on the `impl`, optional clauses on each `fn` |
| `while` / `for` | `maintains`, `decreases` | `#[spec(...)]` on the loop, inside a spec'd `fn` |
| `struct` | `maintains` | `#[spec(...)]` on the `struct` |
| `enum` | `maintains` | `#[spec(...)]` on the `enum` |
<!-- /anodized:generated:item-kinds -->

## Traits

Specifying a trait takes up to four steps. The two structural ones, 1 and 3, are required.
Omitting the trait's gives a macro error naming the missing annotation; omitting an impl's
gives `E0046`, complaining that a generated `__anodized_` item is unimplemented. The other two
are optional:

1. A **bare** `#[spec]` on the trait, with no clauses.
2. `#[spec(...)]` with clauses on the trait's functions, where a function has any. One
   without an attribute simply has an empty spec.
3. A **bare** `#[spec]` on every `impl Trait for T`.
4. Optionally, `#[spec(...)]` on an impl's functions — which may only **narrow**: weaken a
   precondition or strengthen a postcondition, never the reverse.

Note that narrowing runs opposite to repairing a spec, and for a different reason: an impl is
a special case, so it may accept more and promise more than the trait did. Repairing a spec
must never claim more than the code delivers.

Qualifier narrowing is checked at build time by a generated `const { assert!(...) }`, so an
impl claiming less than its trait fails to compile. Pre- and postcondition narrowing is
checked only at runtime. Items prefixed `__anodized_` are internal; never implement them.

## Loops

A loop spec takes `maintains` and `decreases`, and the enclosing function must itself carry
`#[spec]`. `decreases` is one expression that strictly decreases each iteration and is bounded
below — it is the termination argument.

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

Loop specs are **not** checked at runtime, and their conditions are **not type-checked**
either: a loop invariant naming a variable that does not exist still compiles. They document
intent and feed analysis backends. Do not rely on them catching anything today.

## Data

`struct` and `enum` take `maintains` only, describing what is always true of a value. `self`
is in scope, and enum variants are brought into scope automatically. Neither is checked at
runtime yet. A `struct` condition **is** type-checked; an `enum` condition is not.

```rust
use anodized::spec;

#[spec(maintains: self.low <= self.high)]
struct Range {
    low: u32,
    high: u32,
}
```

## Build configurations

<!-- anodized:generated:cfg-flags -->
| Build configuration | Effect |
| --- | --- |
| *(none)* | Specs embedded and type-checked; no check runs. `captures` still evaluate. |
| `--cfg anodized_discard_specs` | Specs are dropped entirely: not embedded, not type-checked. Fastest builds. |
| `--cfg anodized_print` | A violated condition reports to stderr and execution continues. |
| `--cfg anodized_panic` | A violated condition panics, without naming the condition. Pair with `anodized_print` to be told which expression failed. |
| `--cfg anodized_try` | Enables `try_call!`, which turns a violation into a `Result`. Requires `anodized_panic`. |
<!-- /anodized:generated:cfg-flags -->

Turn checks on for tests:

```bash
RUSTFLAGS="--cfg anodized_print --cfg anodized_panic" cargo test
```

`anodized_print` names the failing condition in the message, so pairing it with
`anodized_panic` is the usual choice; `anodized_panic` alone panics without saying which
clause broke.

With no flag set, checks do not run — but the spec is not entirely free: `captures`
expressions are evaluated on every call regardless, because the capture and the body are bound
in one statement. A `.clone()` in a capture is a real clone in a release build. Use
`--cfg anodized_discard_specs` where you need the spec to cost exactly nothing.

A condition may carry one `#[cfg(...)]` attribute — and only `cfg`, never anything else, and
never on `captures`.

```rust,ignore
// fragment
#[spec(
    requires: !items.is_empty(),
    #[cfg(debug_assertions)]
    maintains: items.is_sorted(),
)]
```

## Helpers

`anodized-logic` supplies vocabulary for conditions: `implies!(p, q)` for lazy material
implication, `int` for unbounded arithmetic, and `opaque!` plus `forall`/`exists` for terms
meant only for static backends — the last three panic if actually evaluated.

## Adding a spec to existing code

Read the function and list its real obligations. Write the smallest spec that states them.
Check it compiles with `cargo check`, then run the suite with checks live
(`RUSTFLAGS="--cfg anodized_panic" cargo test`). Format with `anodized-fmt` if available.
Finally re-read: delete any clause that only repeats the type.

When a check fires, never edit a clause just to make it pass. Three things can be wrong — the
call site, the spec, or the body — and a repair names exactly one of them as wrong, taking the
other two as correct. A `precondition failed` usually points at the call site or the `requires`
and a `postcondition failed` at the body or the spec, but neither rules its third candidate out
entirely. Edit the spec only when the spec is what is wrong, and say which you decided it was.

Read `diagnostics.md` before acting on a failure: it carries the full model, and the message
alone is not enough to act on — it does not name the kind of clause that broke, only
`anodized_print` names the expression at all, and under that flag execution continues, so later
failures may be fallout from the first.

## Reference files

Open a sibling file when its situation applies, not by default:

| Situation | File |
| --- | --- |
| A `#[spec]` fails to compile, or a check fires | `diagnostics.md` |
| Someone writes `old()`, `result`, or names another contract crate; code predates 0.6 | `migration.md` |
| You need the exact grammar, or the semantics behind a clause | `reference.md` |
| You need a correct spec for a given item kind to copy | `examples.md` |
