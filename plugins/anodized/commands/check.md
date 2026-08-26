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

Report the results as a table of pass versus outcome. A compile error only under
`anodized_discard_specs` means real code depends on something a spec brought into scope.

Never edit a clause just to make a pass go green. Which of three things is wrong — the call
site, the spec, or the body — decides the repair, and a repair names exactly one of them,
taking the other two as correct. A `precondition failed` points at the call site or a
too-strong `requires`; a `postcondition failed` points at the body or an overclaiming spec.

Two things the message does not tell you. A `maintains` clause is checked on entry and exit
and reports as `precondition failed` one way and `postcondition failed` the other, so a
`precondition failed` may name an invariant. And only the `anodized_print` passes name the
failing expression, so reproduce with both flags before diagnosing — under `anodized_print`
execution also continues, so trust the first failure and presume the rest is fallout.

The skill's `diagnostics.md` carries the full model, including which candidate each failure
rarely but genuinely admits, and which edits can break an existing caller. Read it before
changing a clause, and say what you changed and why.
