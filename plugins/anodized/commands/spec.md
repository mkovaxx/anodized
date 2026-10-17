---
description: Add or tighten Anodized specs on the selected functions.
allowed-tools: Bash(cargo:*), Bash(anodized-fmt:*)
---

# Add Anodized specs

Target: $ARGUMENTS (a path, a function name, or the current selection; if empty, ask what to
specify).

Follow the `anodized` skill's decision rules. In short:

1. Read the target and list what it actually requires of its caller and guarantees in return.
   Look for existing `assert!`, `panic!`, and early `return Err` — those are preconditions
   already written in another form.
2. Write the **smallest** spec that captures them. Skip anything the type system already
   states: `requires: [x >= 0]` on a `u32` is noise, and so is a postcondition restating the
   return type.
3. Keep clauses in the mandatory order — qualifiers, `requires`, `maintains`, `captures`,
   `ensures` — and prefer several small clauses over one `&&` chain, so a failure names the
   part that broke.
4. Bind the return value with an `ensures` closure (`ensures: |output| ...`). There is no
   implicit `output` binding and no `old()`; entry-time values come from `captures`.
5. Verify with `cargo check`, then `cargo --config 'build.rustflags=["--cfg","anodized_print","--cfg","anodized_panic"]' test`.
6. Format with `anodized-fmt` if it is installed.

Show the diff and explain each clause in one line. If a function needs no spec, say so rather
than inventing one — but prefer a bare `#[spec]` to no attribute at all. It records that the
contract is deliberately tautological, equivalent to `requires: true` and `ensures: true`,
which is not the same as nobody having considered it. On a function nested in a `trait` or an
`impl`, spell it `#[spec()]` — a nested attribute is read with `parse_args`, so the parentheses
are required even when empty.

Some functions cannot carry a spec at all, because the macro evaluates the body in a closure:
returning `impl Trait` (`E0562`), `const fn` (`E0015`), and returning `&mut` borrowed from a
parameter all fail to compile. That list is not exhaustive — if adding a spec breaks the
build, the shape cannot carry one, and the fix is to drop the attribute rather than to work
around it.
