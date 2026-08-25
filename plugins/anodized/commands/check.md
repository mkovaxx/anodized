---
description: Compile and test the current crate under every Anodized build configuration.
allowed-tools: Bash(cargo:*)
---

# Check every Anodized build configuration

Anodized behavior is controlled entirely by `--cfg` flags, not Cargo features. A spec that
type-checks under one configuration can still fail under another, so verify all six — the same
set the Anodized CI matrix runs.

Run each pass in order, from the current crate's directory:

```bash
cargo test --no-fail-fast
cargo --config 'build.rustflags=["--cfg","anodized_discard_specs"]' test --no-fail-fast
cargo --config 'build.rustflags=["--cfg","anodized_print"]' test --no-fail-fast
cargo --config 'build.rustflags=["--cfg","anodized_panic"]' test --no-fail-fast
cargo --config 'build.rustflags=["--cfg","anodized_print","--cfg","anodized_panic"]' test --no-fail-fast
cargo --config 'build.rustflags=["--cfg","anodized_print","--cfg","anodized_panic","--cfg","anodized_try"]' test --no-fail-fast
```

What each pass proves:

| Pass | Proves |
| --- | --- |
| no cfg | specs are embedded and type-checked; checks do not run |
| `anodized_discard_specs` | the code still builds with specs erased entirely |
| `anodized_print` | violations report to stderr and execution continues |
| `anodized_panic` | violations panic |
| `print` + `panic` | both, the usual configuration for a test suite |
| `print` + `panic` + `try` | additionally enables `try_call!` |

Report the results as a table of pass versus outcome. For any failure:

- `precondition failed: <expr>` — the **caller** violated the contract. Fix the call site, or
  the precondition if it was wrong.
- `postcondition failed: <expr>` — the **implementation** violated its own contract. Fix the
  body, or the postcondition if it was wrong.
- A compile error only under `anodized_discard_specs` means real code depends on something a
  spec brought into scope.

Never edit a clause just to make a pass go green. Which edit is legitimate depends on which
kind of check fired.

A **postcondition** failure means the spec is false of the code. If the clause overclaimed,
weaken the postcondition, or strengthen the precondition so it no longer covers the inputs
where the guarantee does not hold. Either keeps the spec true without re-examining the body.

A **precondition** failure means a caller broke an obligation; the spec is not thereby false
of the code, so the usual repair is at the call site. Weaken the precondition only when the
body demonstrably handles the input it rejected — that is a new claim about the body, so
verify it rather than assuming it.

Only the passes with `anodized_print` name the failing expression; `anodized_panic` alone
reports the bare text, so reproduce with both before diagnosing.

Before applying any of that, check which clause failed, because the message does not say. A
`maintains` clause is checked twice and reports as `precondition failed` on entry and
`postcondition failed` on exit, so a `precondition failed` may name an invariant rather than a
`requires`. Weakening a `maintains` relaxes the exit guarantee as well as the entry condition,
so it is not the safe edit that weakening a true precondition is.

Note who each edit can hurt: weakening a genuine precondition cannot break an existing caller,
while strengthening a precondition, weakening a postcondition, or weakening an invariant can.
Say what you changed and why.
