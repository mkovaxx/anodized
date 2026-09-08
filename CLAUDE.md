# Working on Anodized

This file is for agents changing *this repository*. The skill under
`plugins/anodized/skills/anodized/` is a different thing: it teaches people who *use* the
crate how to write specs. Both exist; do not confuse them.

## Layout

| Crate | Owns |
| --- | --- |
| `anodized` | The user-facing facade: re-exports `spec`, plus `result` and `try_call!`. |
| `anodized-macros` | The proc macro, and the `cfg` dispatch that decides what it emits. |
| `anodized-core` | Parsing and instrumentation. The real logic, and the tool interop layer. |
| `anodized-logic` | `int`, `implies!`, `opaque!`, `forall`, `exists`. |
| `anodized-fmt` | Formatter library and CLI for `#[spec(...)]`. |
| `test-util` | Internal test helpers. Not published. |
| `skill-tools` | Generates and validates the agent skill. Not published. |

`README.md` is a symlink to `crates/anodized/README.md`, which is included as the crate's
rustdoc via `#![doc = include_str!]` — so **its examples are doctested**.

## Build configurations

There are **no Cargo features**. Behavior is controlled entirely by `--cfg` flags:

| Flag | Effect |
| --- | --- |
| *(none)* | Specs embedded and type-checked; no checks run. |
| `anodized_discard_specs` | Specs dropped entirely. |
| `anodized_print` | Violations report to stderr and continue. |
| `anodized_panic` | Violations panic. |
| `anodized_try` | Enables `try_call!`. Requires `anodized_panic`. |

CI passes them as `cargo --config 'build.rustflags=["--cfg","anodized_panic"]' test`. Adding a
new flag means updating `check-cfg` in **both** `crates/anodized/Cargo.toml` and
`crates/anodized-macros/Cargo.toml`; `skill-tools` fails if the documented set drifts.

## Commands

```bash
cargo fmt --all -- --check
cargo clippy -p <crate> --all-targets -- -D warnings
cargo test -p anodized --tests --no-fail-fast
cargo test -p skill-tools --tests --no-fail-fast
cargo run -p skill-tools --bin skill-gen -- --write
```

## Conventions

Clippy must be clean. **No `#[allow]`** — fix the cause, or delete the code. No `unwrap`,
`expect`, or `panic!` in production paths; propagate with `?`. Files stay under 500 lines.

Unit tests live in a sibling file, wired up with `#[cfg(test)] #[path = "x_tests.rs"] mod`.
Integration tests are one file per feature under `crates/anodized/tests/`. UI tests use
trybuild with `.stderr` goldens — regenerate with `TRYBUILD=overwrite`. `anodized-fmt` uses
golden fixtures under `tests/fixtures/`.

## Editing the agent skill

`plugins/anodized/skills/anodized/` is a shipped artifact.

- Every ```` ```rust ```` block in it is compiled by `cargo test -p skill-tools`. So is every
  block in `crates/anodized/REFERENCE.md`.
- The tables between `<!-- anodized:generated:... -->` markers are generated. Edit
  `crates/skill-tools/src/generate.rs`, then run `skill-gen --write`. Do not hand-edit inside
  the markers.
- Change a clause keyword, a qualifier, a `cfg` flag, or a macro error message and the tests
  will name the document that needs updating.

The harness has one blind spot worth knowing: loop and `enum` conditions are dropped before
they reach the compiler, so "every block compiles" proves nothing about the conditions inside
a loop or enum example. Read those by eye.

**No test covers meaning.** The coverage tests assert that a term *appears*; they cannot tell
whether the sentence around it is still true. Changing spec semantics requires a manual read
of `SKILL.md` and `reference.md`. This is the residual risk and it is not automatable.

## Which documents are verified

| Document | Verified? |
| --- | --- |
| `crates/anodized/README.md` | Yes — doctested as crate rustdoc. |
| `crates/anodized/REFERENCE.md` | Examples only, via `skill-tools`. Prose is not. |
| The agent skill | Examples, tables, and runtime claims, via `skill-tools`. Prose is not. |

When they disagree, the skill and the README win.

## Releasing

Bump, in one commit: `[workspace.package] version` and the `[workspace.dependencies]`
`anodized*` lines; the `anodized = "x.y.z"` snippet in `crates/anodized/README.md`;
`version` in `.claude-plugin/marketplace.json`, `plugins/anodized/.claude-plugin/plugin.json`,
and `plugins/anodized/.codex-plugin/plugin.json`; and `CHANGELOG.md`.

The plugin version is always the workspace version. `skill-tools` reads its own
`CARGO_PKG_VERSION` — which is the workspace version — and fails if a manifest disagrees, so a
mismatched plugin cannot be released.
