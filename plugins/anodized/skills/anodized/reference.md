# Anodized reference

Semantics behind the tables in `SKILL.md`. Read this when you need to know *why* a clause
behaves the way it does, or what exactly the parser accepts.

## Grammar

<!-- anodized:generated:ebnf -->
```ebnf
fields = [ qualifiers ]
         [ requires_fields ]
       , [ maintains_fields ]
       (* not a typo: at most one `captures:` *)
       , [ captures_field ]
       , [ ensures_fields ];

qualifiers = { qualifier };
qualifier = `functional` | `pure` | `total`
          | `deterministic` | `effectfree` | `infallible` | `terminating`;

requires_fields  = { requires_field };
maintains_fields = { maintains_field };
ensures_fields   = { ensures_field };

requires_field  = [ cfg_attr ] , `requires:` , conditions, `,`;
maintains_field = [ cfg_attr ] , `maintains:` , conditions, `,`;
captures_field  = `captures:` , captures, `,`;
ensures_field   = [ cfg_attr ] , `ensures:` , postconds, `,`;

conditions = expr | condition_list;
condition_list = `[` , expr , { `,` , expr } , [ `,` ] , `]`;

captures = capture_stmt | capture_list;
capture_list = `[` , capture_stmt , { `,` , capture_stmt } , [ `,` ] , `]`;
capture_stmt = pattern , `=` , expr;

postconds = postcond_expr | postcond_list | postcond_list_closure;
postcond_list = `[` , postcond_expr , { `,` , postcond_expr } , [ `,` ] , `]`;
postcond_expr = expr | postcond_closure;
postcond_closure = `|` , pattern , `|` , expr;
postcond_list_closure = `|` , pattern , `|` , condition_list;

cfg_attr = `#[cfg(` , settings , `)]`;
```
<!-- /anodized:generated:ebnf -->

Notes on reading it: `expr` is any Rust expression, `pattern` is any irrefutable Rust pattern,
and every field is parseable as a Rust struct-expression field — which is why the attribute
uses `name: value` and tolerates a trailing comma. `decreases` is absent from the function
fields on purpose: it belongs to loops only.

## How a spec is instrumented

The macro rewrites the body into something equivalent to this:

1. Preconditions and invariants are checked.
2. Captures and the body are evaluated in a **single tuple assignment**, so the body cannot
   see the captured values, and the body runs inside a closure so an early `return` cannot
   skip the postconditions.
3. Invariants are checked again, then postconditions, with the captures and the return value
   in scope.
4. The return value is produced.

Two consequences follow, and both surface as ordinary compiler errors rather than as
Anodized-specific ones. Conditions are invoked through `anodized::__::eval`, which takes an
`impl Fn() -> T`: a condition that mutates anything cannot satisfy `Fn`, so it fails to
compile. And captures are bound in the same statement that runs the body, so they cannot be
named by the body or by a precondition.

## Clauses

**`requires`** — checked on entry, before captures. One condition or a list.

**`maintains`** — checked twice, on entry and again on exit. On a `fn` it usually describes
`&self` or a `&mut` parameter. On a `struct` or `enum` it is a refinement on the type itself.

**`captures`** — evaluated after `requires`, before the body. At most one field per spec; use
`captures: [a = x, b = y]` for several. The right-hand side is moved, not copied, so a
non-`Copy` value needs `.clone()`. The left-hand side is any irrefutable pattern, including
tuples and struct patterns. Captures are visible **only** to postconditions.

**`ensures`** — checked after the body and after the exit invariants. The return value is
bound only by a closure pattern; a bare expression cannot see it. Within a list, each element
may carry its own closure.

**`decreases`** — loops only, one expression, at most one field. States the quantity that
strictly decreases and is bounded below.

Ordering is mandatory and enforced by the parser: qualifiers, `requires`, `maintains`,
`captures`, `ensures`; and for loops, `maintains` then `decreases`. Repeating `requires`,
`maintains`, or `ensures` is allowed, and the conditions accumulate.

## Conditional compilation of a clause

Any of `requires`, `maintains`, `ensures`, and `decreases` may carry exactly one `#[cfg(...)]`
attribute. No other attribute is accepted, and `captures` accepts none at all. A gated
condition is still type-checked when its check is compiled out, which is what makes
`#[cfg(test)]` and `#[cfg(debug_assertions)]` safe places to put an expensive invariant.

## Traits

Trait functions are rewritten to a `__anodized_`-prefixed name with a generated default that
carries the checks. That is why the two structural attributes are required: a bare
`#[spec]` on the trait, and a bare `#[spec]` on each impl. Clauses on the trait's functions and
narrowing clauses on an impl's functions are both optional — a function with no attribute
simply has an empty spec. Omitting an impl's bare `#[spec]` does not silently skip
instrumentation; it fails to compile with `E0046`, naming the generated `__anodized_` item as
unimplemented.

Qualifiers on an impl may not be weaker than on the trait, and this is enforced at build time
by a generated `const { assert!(...) }`. Pre- and postcondition narrowing is a contract you
are trusted to honor; only runtime checks will catch a violation.

Trait function parameters must be patterns the macro can forward: names, tuples, tuple
structs, struct patterns, slices, literals, paths, ranges, references. It rejects `_`, `..`,
or-patterns, macro patterns, `ref` bindings, and struct patterns with `..`.

## `try_call!` and `anodized::result`

With `--cfg anodized_try` (which requires `--cfg anodized_panic`), `try_call!` wraps a call so
a violation becomes a `Result` instead of a panic, distinguishing precondition from
postcondition failure. It accepts a method call or a **qualified** function call —
`path::f(..)`, `Type::f(..)`, `<T as Tr>::f(..)`, `recv.m(..)` — but not a bare `f(x)`. This is
what lets property-based testing discard invalid inputs without also swallowing genuine
postcondition failures.

## `anodized-logic`

- `implies!(p, q)` — material implication, evaluating `q` lazily. A macro because arguments
  are otherwise eager.
- `int` — unbounded integer with operator interop across the primitive integer types, for
  conditions that would otherwise overflow.
- `opaque!(tokens)` — a term whose meaning is defined by an analysis backend.
- `forall` / `exists` — quantifiers for static backends.

`opaque!`, `forall`, and `exists` **panic** if evaluated at runtime. They belong in conditions
that a backend interprets, not in ones a test executes.

## `anodized-fmt`

A formatter for `#[spec(...)]` blocks, configured by `anodized-fmt.toml`. Install with
`cargo install anodized-fmt`; it normalizes clause layout and ordering so specs read
consistently across a codebase.

## Removed, or never there

| Form | Status |
| --- | --- |
| `old(x)` | Never existed. Use `captures`. |
| `old_x` | Removed in 0.6. Use `captures: old_x = x`. |
| `result` / bare `output` | Never existed. Bind with an `ensures` closure. |
| `captures: expr as pattern` | Removed in 0.6. Now `pattern = expression`. |
| `binds:` / `inspects:` | Removed in 0.6. Use `ensures: \|PAT\| [EXPR, ...]`. |
| `#[requires(...)]` / `#[ensures(...)]` | Never existed. One unified `#[spec(...)]`. |
| Closure-form preconditions | Removed in 0.6. Write the expression directly. |

The parser still recognizes `binds` and `inspects` purely so it can tell you what replaced
them; they are scheduled for deletion before 0.7.
