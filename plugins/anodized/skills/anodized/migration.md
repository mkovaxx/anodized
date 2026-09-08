# Migrating to Anodized

Every contract system spells the same ideas differently. This file translates. If you are
about to write something remembered from elsewhere, find it here first.

## From Anodized 0.5 or earlier

**Start here.** 0.6 changed the syntax, so a model's or a codebase's memory of this crate is
more likely to be stale than to be wrong about some other crate.

| Before 0.6 | Now |
| --- | --- |
| `old_x` (automatic) | `captures: old_x = x`, read in `ensures` |
| `captures: x.len() as n` | `captures: n = x.len()` |
| `ensures: output > 0` | `ensures: \|output\| output > 0` |
| `binds: \|pat\| ...` | `ensures: \|pat\| [...]` |
| `inspects: ...` | `ensures: \|pat\| [...]` |
| `requires: \|\| condition` | `requires: condition` |

The automatic `old_` binding is the one that bites hardest, because the replacement is not a
rename: captures must be declared, and they are visible only to postconditions.

## From `contracts`

| `contracts` | Anodized |
| --- | --- |
| `#[requires(x > 0)]` | `#[spec(requires: x > 0)]` |
| `#[ensures(ret > 0)]` | `#[spec(ensures: \|output\| output > 0)]` |
| `#[invariant(...)]` | `#[spec(maintains: ...)]` |
| `old(x)` | `captures: old_x = x` |
| separate attributes | one `#[spec(...)]` carrying every clause |

## From Creusot

| Creusot | Anodized |
| --- | --- |
| `#[requires(...)]` / `#[ensures(...)]` | clauses inside `#[spec(...)]` |
| `result` | the `ensures` closure's binding |
| `^x` (final value) | `maintains`, or a capture plus a postcondition |
| `pearlite!` terms | plain Rust expressions; `opaque!` for what cannot be written |
| `#[predicate]` / logic functions | ordinary pure Rust functions |

## From Prusti

| Prusti | Anodized |
| --- | --- |
| `#[requires(...)]` / `#[ensures(...)]` | clauses inside `#[spec(...)]` |
| `old(x)` | `captures: old_x = x` |
| `result` | the `ensures` closure's binding |
| `#[pure]` | the `pure` qualifier |
| `#[trusted]` | no equivalent; Anodized does not verify statically |
| `body_invariant!(...)` | `#[spec(maintains: ...)]` on the loop |

## From Kani

Closest cousin, and the closure form is genuinely similar — which makes the differences easy
to miss.

| Kani | Anodized |
| --- | --- |
| `#[kani::requires(...)]` | `#[spec(requires: ...)]` |
| `#[kani::ensures(\|result\| ...)]` | `#[spec(ensures: \|output\| ...)]` |
| `old(...)` | `captures: name = ...` |
| `#[kani::modifies(...)]` | no equivalent |

## From Verus

| Verus | Anodized |
| --- | --- |
| `requires` / `ensures` blocks in the signature | clauses inside `#[spec(...)]` |
| `old(x)` | `captures: old_x = x` |
| `ret` | the `ensures` closure's binding |
| `invariant` on a loop | `#[spec(maintains: ...)]` |
| `decreases` on a loop | `#[spec(decreases: ...)]`, same idea |
| ghost/spec functions | ordinary Rust functions |

## If you were about to write X

| X | Write instead |
| --- | --- |
| `old(x)`, `old_x` | `captures: old_x = x` |
| `result`, bare `output` | `ensures: \|output\| ...` |
| `#[requires(...)]` above the fn | `#[spec(requires: ...)]` |
| `ensures: ret == ...` | `ensures: \|output\| output == ...` |
| `captures: expr as name` | `captures: name = expr` |
| `#[invariant(...)]` on a loop | `#[spec(maintains: ...)]` on the loop |
| a spec expression that mutates | rewrite it to observe only; conditions are `Fn` |
| `#[pure]` attribute | the bare `pure` qualifier inside `#[spec(...)]` |

## What Anodized does not do

It does not verify statically. `#[spec]` embeds a specification that the compiler type-checks
and that runtime checks can enforce; proving a postcondition for all inputs is the job of a
backend built on `anodized-core`, not of the macro. There is no `#[trusted]`, no proof
obligation, and no solver.
