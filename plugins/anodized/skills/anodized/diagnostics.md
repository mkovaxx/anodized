# Anodized diagnostics

What the compiler tells you, what provoked it, and what to do. Three sources of error matter:
the macro's own messages, ordinary rustc errors reached *through* a spec, and runtime check
failures.

## Macro errors

The macro accumulates errors, so several clause problems surface at once.

<!-- anodized:generated:diagnostics -->
| Message | Cause | Fix |
| --- | --- | --- |
| ``unknown spec field`` | An unrecognized clause name, including the removed `binds:` and `inspects:` forms. | Use a current clause or qualifier. Replace `binds:` and `inspects:` with `ensures: \|PAT\| [EXPR, ...]`. |
| ``fields are out of order: the expected order is`` | Clauses are present but in the wrong sequence. | Reorder to qualifiers, `requires`, `maintains`, `captures`, `ensures`. |
| ``at most one `captures` field is allowed`` | Two `captures:` fields in one spec. | Merge them: `captures: [first = a, second = b]`. |
| ``postcondition closure must have exactly one input`` | An `ensures` closure taking zero or several inputs. | Take exactly the return value: `ensures: |output| ...`. |
| ``qualifier does not take a value`` | A qualifier written as `pure: true`. | Write it bare: `pure,`. |
| ``this qualifier is redundant; remove it`` | A composite qualifier alongside one of its components. | Keep the composite alone; `pure` already implies `deterministic` and `effectfree`. |
| ``attributes are not supported here`` | An attribute on a qualifier. | Remove it; only conditions accept `#[cfg]`. |
| ``unsupported attribute; only `cfg` is allowed`` | An attribute other than `#[cfg]` on a clause. | Remove it. |
| ``multiple `cfg` attributes are not supported`` | Two `#[cfg]` attributes on one clause. | Combine them: `#[cfg(all(test, debug_assertions))]`. |
| ```cfg` attribute is not supported here`` | A `#[cfg]` on `captures`. | Gate the postconditions that read the capture instead. |
| ``multiple `decreases` fields are not allowed`` | Two loop variants. | Keep one expression. |
| ``multiple `#[{name}]` attributes are not allowed`` | Repeated attributes on one item; `{name}` is `spec` for specification attributes. | Merge the clauses into one attribute. |
| ``expected an expression`` | A clause whose value is not an expression. | Supply a `bool` expression, or a list of them. |
| ``expected an assignment`` | A `captures` entry that is not `pattern = expression`. | Write `captures: name = expr`. |
| ``expected an assignment or block`` | A `captures` value that is neither an assignment nor a block. | Write `captures: name = expr`, or a list of assignments. |
| ``expected a single expression`` | `decreases` given a list. | Supply one expression. |
| ``not allowed here due to `#[spec]``` | A trait function parameter pattern the macro cannot forward. | Name the parameter; avoid `_`, `..`, `ref`, and macro patterns. |
| ``or-pattern not allowed here due to `#[spec]``` | An or-pattern in a trait function parameter. | Name the parameter and match inside the body. |
| ``The enclosing trait must have a `#[spec]` annotation.`` | Clauses on a trait function whose trait carries no `#[spec]`. | Add a bare `#[spec]` to the trait. |
| ``this `#[spec]` attribute must have no fields`` | Clauses on the attribute attached to a `trait` or `impl`. | Keep the enclosing attribute bare; put clauses on its functions or methods. |
| ``cannot be weaker than the qualifiers on the trait`` | An impl claiming fewer guarantees than the trait promised. | An impl may only narrow; restore the trait's qualifiers. |
| ``The #[spec] attribute doesn't yet support this item`` | `#[spec]` on an item kind with no support, such as `mod` or `type`. | Move it to a supported item. |
| ```try_call` needs the `anodized_try` build `cfg` to be enabled`` | `try_call!` without the build configuration that generates its entry points. | Build with `--cfg anodized_try` and `--cfg anodized_panic`. |
| ``precondition failed`` | At runtime: a `requires` was false on entry. The panic raised without `anodized_print` also uses this for the whole entry group, invariants included. | Fix the call site, or the precondition if it was wrong. |
| ``postcondition failed`` | At runtime: an `ensures` was false on exit. The panic raised without `anodized_print` also uses this for the whole exit group, invariants included. | Fix the body, or the postcondition if it was wrong. |
| ``preinvariant failed`` | At runtime under `anodized_print`: a `maintains` was already false on entry. | The state was wrong before the call; look at whoever produced it. |
| ``postinvariant failed`` | At runtime under `anodized_print`: a `maintains` was false on exit. | The body broke an invariant it was required to preserve. |
<!-- /anodized:generated:diagnostics -->

For function fields, use the order qualifiers, `requires`, `maintains`, `captures`, `ensures`.
The removed `binds` and `inspects` clauses produce `unknown spec field`.

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

Returning a mutable reference borrowed from a parameter is supported. This example compiles
and runs with runtime checks enabled:

```rust
use anodized::spec;

#[spec]
fn first_mut(values: &mut Vec<u8>) -> &mut u8 {
    &mut values[0]
}

fn main() {
    let mut values = vec![1];
    *first_mut(&mut values) = 2;
    assert_eq!(values, vec![2]);
}
```

**`E0046: not all trait items implemented, missing: `__anodized_...``** — an
`impl Trait for T` is missing its bare `#[spec]`. The trait's specs generate a mangled item the
impl is expected to supply, so the fix is to add `#[spec]` to the impl, never to implement the
named item yourself.

**`E0005: refutable pattern`** — an output pattern that does not match every value, such as
`|Ok(v)|`. Bind the whole value and inspect it in the condition: `|output| output.is_ok()`.

```rust,compile_fail
// EXPECT: E0005
use anodized::spec;

#[spec(ensures: |Ok(value)| value > 0)]
fn parse() -> Result<u32, String> {
    Ok(1)
}
```

## Runtime failures

Checks only run when the build enables them.

`precondition failed: <expr>` means the **caller** broke the contract. Fix the call site,
unless the precondition was wrong.

`postcondition failed: <expr>` means the **implementation** broke its own contract. Fix the
body, unless the postcondition was wrong.

Under `anodized_print`, `maintains` reports separately at each end: `preinvariant failed` when
it is already false on entry, `postinvariant failed` when the body broke it. The distinction
matters, because an invariant false on entry is not this call's fault. The panic raised without
`anodized_print` makes no such distinction — it covers the whole entry or exit group, so an
invariant reads there as `precondition failed` or `postcondition failed`, and the location
points at the attribute, naming the function but not the clause.

To tell two clauses of the same kind apart you still have only the expression, and only under
`anodized_print`. Match it by reading rather than searching: the message reprints from the
parsed tokens, so spacing is normalized and rarely matches your source — `!data.is_empty()` is
reported as `! data.is_empty()`. Two textually identical clauses cannot be told apart at all.

`anodized_print` continues after a violation, which makes the reports after the first
unreliable. Trust the first and presume the rest is fallout until it is fixed. A violated
`requires` lets the body run on input it was meant to exclude, so whatever fails next is
usually downstream of that rather than an independent defect; and an invariant already false on
entry is simply reported a second time on exit. Either way a `postcondition failed` does not
always mean the body misbehaved — it may never have had a valid starting state.

## Repairing what failed

Never edit a clause just to make a check pass. Three things can be wrong — the call site, the
spec, or the body — and a repair names exactly one of them as wrong, taking the other two as
correct. Diagnosing two at once is how a spec stops describing the code and starts restating
it. Edits may still cascade: strengthening a `requires` obliges you to fix the callers that
relied on the looser contract, but each of those takes the corrected spec as ground truth.

The failure narrows three candidates to two:

| Failure | Either | Rarely |
| --- | --- | --- |
| `precondition failed` | the call site broke a correct obligation, or `requires` is too strong | the body |
| `postcondition failed` | the body is wrong, or the spec overclaims | the call site |

The last column is rare rather than impossible. An entry-side `maintains` failure reports as
`precondition failed`, and the state it objects to may have been left there by an earlier call
into the same code; under `anodized_print` a later report may be fallout from any of that.

Reach for the call site or the body first. Edit the spec only when the spec is the thing that
is wrong, and say which one you decided it was. Weakening a `requires` asserts that the body
handles inputs it used to reject — a new claim about the body, to be verified rather than
assumed. Where a postcondition overclaims, the alternative to weakening it is strengthening
`requires` so the guarantee no longer covers the failing inputs.

Of every edit named here, weakening a genuine precondition is the only one that cannot break an
existing caller.

Deciding which of the three is wrong is the hard part, and the message alone rarely settles it.
What usually does is context around the failure:

- Several call sites break the same clause, and each looked reasonable — suspect the spec. A
  contract nobody can satisfy is usually the contract's fault.
- One call site breaks it while others honor it — suspect that call site.
- The clause contradicts what the function's name or documentation promises — suspect the
  clause; one of the two was written later and they have drifted apart.
- The spec is older than the last change to the body — suspect the body, and check whether the
  change was meant to alter the contract.
- The failure appeared with no change to any of the three — suspect state reached through a
  reference, which makes the culprit some earlier call rather than this one.

Under `--cfg anodized_print` the message goes to stderr and execution continues. Under
`--cfg anodized_panic` it panics. Enabling both — the usual choice for a test suite — gives
the panic with the failing expression named in the printed message. To reproduce a report:

```bash
RUSTFLAGS="--cfg anodized_print --cfg anodized_panic" cargo test
```

Never relax a clause to silence a check. Fix whichever side is actually wrong, and if the
clause really was wrong, say so explicitly in the commit message.

## Loop and data specs need different checks

Loop invariants and variants are parsed and dropped: they neither run nor reach the type
checker. An unknown identifier in a loop condition can compile silently. Review these by eye.

Both `struct` and `enum` invariants generate a type-checked `anodized::types::Spec::predicate`
unless `anodized_discard_specs` is enabled. An unknown name in either produces `E0425`.
Construction alone does not evaluate that predicate. Call it explicitly, or use `Spec!(T)`
at a spec'd function boundary with runtime checks enabled. A passing test that never evaluates
the predicate is not evidence that the invariant holds.
