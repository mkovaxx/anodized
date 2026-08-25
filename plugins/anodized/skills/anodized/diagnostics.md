# Anodized diagnostics

What the compiler tells you, what provoked it, and what to do. Three sources of error matter:
the macro's own messages, ordinary rustc errors reached *through* a spec, and runtime check
failures.

## Macro errors

The macro accumulates errors, so several clause problems surface at once.

<!-- anodized:generated:diagnostics -->
| Message | Cause | Fix |
| --- | --- | --- |
| ``unknown spec field`` | A clause name the parser does not recognize, often a guess from another crate. | Use one of `requires`, `maintains`, `captures`, `ensures`, `decreases`, or a qualifier. |
| ``fields are out of order: the expected order is`` | Clauses are present but in the wrong sequence. | Reorder to qualifiers, `requires`, `maintains`, `captures`, `ensures`. |
| ``at most one `captures` field is allowed`` | Two `captures:` fields in one spec. | Merge them: `captures: [first = a, second = b]`. |
| ``no longer supported, use the following form instead`` | `binds:` or `inspects:`, both removed. | Write `ensures: |PAT| [EXPR, ...]` instead. |
| ``postcondition closure must have exactly one input`` | An `ensures` closure taking zero or several inputs. | Take exactly the return value: `ensures: |output| ...`. |
| ``qualifier does not take a value`` | A qualifier written as `pure: true`. | Write it bare: `pure,`. |
| ``this qualifier is redundant; remove it`` | A composite qualifier alongside one of its components. | Keep the composite alone; `pure` already implies `deterministic` and `effectfree`. |
| ``attributes are not supported here`` | An attribute on a qualifier. | Remove it; only conditions accept `#[cfg]`. |
| ``unsupported attribute; only `cfg` is allowed`` | An attribute other than `#[cfg]` on a clause. | Remove it. |
| ``multiple `cfg` attributes are not supported`` | Two `#[cfg]` attributes on one clause. | Combine them: `#[cfg(all(test, debug_assertions))]`. |
| ```cfg` attribute is not supported here`` | A `#[cfg]` on `captures`. | Gate the postconditions that read the capture instead. |
| ``multiple `decreases` fields are not allowed`` | Two loop variants. | Keep one expression. |
| ``multiple `#[spec]` attributes on a single item are not supported`` | Two `#[spec]` attributes stacked on one item. | Merge the clauses into one attribute. |
| ``expected an expression`` | A clause whose value is not an expression. | Supply a `bool` expression, or a list of them. |
| ``expected an assignment`` | A `captures` entry that is not `pattern = expression`. | Write `captures: name = expr`. |
| ``expected an assignment or block`` | A `captures` value that is neither an assignment nor a block. | Write `captures: name = expr`, or a list of assignments. |
| ``expected a single expression`` | `decreases` given a list. | Supply one expression. |
| ``not allowed here due to `#[spec]``` | A trait function parameter pattern the macro cannot forward. | Name the parameter; avoid `_`, `..`, `ref`, and macro patterns. |
| ``or-pattern not allowed here due to `#[spec]``` | An or-pattern in a trait function parameter. | Name the parameter and match inside the body. |
| ``The enclosing trait must have a `#[spec]` annotation.`` | Clauses on a trait function whose trait carries no `#[spec]`. | Add a bare `#[spec]` to the trait. |
| ``Unsupported spec element on trait.`` | Clauses on the `#[spec]` attached to a trait. | Keep the trait's attribute bare; put clauses on its functions. |
| ``Unsupported spec element on trait impl.`` | Clauses on the `#[spec]` attached to an `impl Trait for T`. | Keep it bare; put clauses on the functions. |
| ``Unsupported spec element on inherent impl.`` | Clauses on the `#[spec]` attached to an inherent `impl`. | Keep it bare; put clauses on the methods. |
| ``cannot be weaker than the qualifiers on the trait`` | An impl claiming fewer guarantees than the trait promised. | An impl may only narrow; restore the trait's qualifiers. |
| ``The #[spec] attribute doesn't yet support this item`` | `#[spec]` on an item kind with no support, such as `mod` or `type`. | Move it to a supported item. |
| ```try_call` needs the `anodized_try` build `cfg` to be enabled`` | `try_call!` without the build configuration that generates its entry points. | Build with `--cfg anodized_try` and `--cfg anodized_panic`. |
| ``precondition failed`` | At runtime: the caller broke the contract. | Fix the call site, or the precondition if it was wrong. |
| ``postcondition failed`` | At runtime: the implementation broke its own contract. | Fix the body, or the postcondition if it was wrong. |
<!-- /anodized:generated:diagnostics -->

One wording to be aware of: the out-of-order message for function fields still lists
`inspects`, a clause that no longer exists. Ignore that entry and use the order
qualifiers, `requires`, `maintains`, `captures`, `ensures`.

## Rustc errors reached through a spec

These are the confusing ones, because nothing in the message mentions Anodized.

**`E0596: cannot borrow as mutable`** — a condition tried to mutate something. Every condition
is coerced to `Fn() -> bool` through `anodized::__::eval`, and an `FnMut` body cannot satisfy
`Fn`. Conditions must observe, never change. This includes the right-hand side of a
`captures` entry.

**`E0425: cannot find value`** — one of three things. A capture was read from `requires` or
from the function body, where it is not in scope. Or a postcondition named `output` or
`result` without binding it, expecting an implicit binding that does not exist. Or the code
predates 0.6 and expects an automatic `old_` binding, which was removed.

**`E0308: mismatched types`** — a condition is not `bool` (every one is checked as
`eval::<bool>`), or an `ensures` closure's pattern does not match the return type. Also
appears as *expected `bool`, found closure* when a closure is nested inside a list that is
already bound by a closure (`ensures: |pat| [a, |x| b]`), or when a pre-0.6 closure-form
precondition survives. Note that `ensures: [a, |pat| b]` is legal: a top-level element of an
unbound list may carry its own binding.

**`E0562: `impl Trait` is not allowed in closure return types`** — the function returns
`impl Trait`. The macro evaluates the body inside a closure so that an early `return` cannot
skip the postconditions, and a closure cannot return `impl Trait`. There is no spelling of the
spec that avoids this: the function cannot carry `#[spec]` at all until it returns a named
type. Only `anodized_discard_specs`, which drops specs before they are expanded, still builds.

```rust,compile_fail
// EXPECT: E0562
use anodized::spec;

#[spec]
fn counter() -> impl Iterator<Item = u32> {
    0..3
}
```

**`E0015: cannot call non-const closure in constant functions`** — the same closure wrapping,
reached from a `const fn`. The wrapper is not a const closure, so a `const fn` cannot carry a
spec either. As with `E0562`, the only build that survives is `anodized_discard_specs`.

```rust,compile_fail
// EXPECT: E0015
use anodized::spec;

#[spec]
const fn double(value: u32) -> u32 {
    value * 2
}
```

**`captured variable cannot escape `FnMut` closure body`** — the same closure wrapping again,
reached from a function that returns a `&mut` borrowed from one of its parameters. The
reference cannot outlive the closure that produced it. Note this one carries **no error code**,
so it will not match a search by `E`-number. Returning a shared `&` is unaffected.

```rust,compile_fail
// EXPECT: cannot escape `FnMut` closure body
use anodized::spec;

#[spec]
fn first_mut(values: &mut Vec<u8>) -> &mut u8 {
    &mut values[0]
}
```

**`E0046: not all trait items implemented, missing: `__anodized_...``** — an
`impl Trait for T` is missing its bare `#[spec]`. The trait's specs generate a mangled item the
impl is expected to supply, so the fix is to add `#[spec]` to the impl, never to implement the
named item yourself.

**`E0507: cannot move out`** — an `ensures` closure took a non-`Copy` return value by move and
then used it in a way that needs ownership elsewhere. Bind by reference: `|ref output|`.

## Runtime failures

Checks only run when the build enables them.

`precondition failed: <expr>` means the **caller** broke the contract. Fix the call site,
unless the precondition was wrong.

`postcondition failed: <expr>` means the **implementation** broke its own contract. Fix the
body, unless the postcondition was wrong.

Neither message names the clause kind, and a `maintains` clause borrows both: it is checked on
entry, where a violation reports as `precondition failed`, and again on exit, where it reports
as `postcondition failed`. Match the quoted expression against the spec before concluding
which kind of clause you are looking at — bearing in mind that the expression appears only
under `anodized_print`; `anodized_panic` alone reports the bare text with a location pointing
at the attribute, which identifies the function but not the clause.

One consequence worth knowing under `anodized_print`, which continues after a violation: an
invariant already false on entry is reported again on exit, so a `postcondition failed` does
not always mean the body misbehaved — the body may never have had a valid starting state.

Under `--cfg anodized_print` the message goes to stderr and execution continues. Under
`--cfg anodized_panic` it panics. Enabling both — the usual choice for a test suite — gives
the panic with the failing expression named in the printed message. To reproduce a report:

```bash
RUSTFLAGS="--cfg anodized_print --cfg anodized_panic" cargo test
```

Never relax a clause to silence a check. Fix whichever side is actually wrong, and if the
clause really was wrong, say so explicitly in the commit message.

## Loop and data specs do not fire

Runtime checking is implemented for functions only. Loop invariants, loop variants, and
`struct` or `enum` invariants never execute, so a plainly false one will not fail a test —
never read a passing suite as evidence that one holds.

Type-checking differs between them, which matters more than it sounds. A `struct` condition is
embedded and type-checked, so a typo in one is `E0425`. Loop and `enum` conditions are dropped
before they reach the compiler: a loop invariant naming an identifier that does not exist
anywhere compiles without complaint. Review those by eye; nothing else will.
