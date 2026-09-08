# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- **[anodized] Agent skill and plugin** - The repository now ships an agent skill for Claude
  Code and Codex under `plugins/anodized`, installable with
  `claude plugin marketplace add anodized-rs/anodized`. Its factual tables are generated from
  the crate sources and its examples are compiled, so it cannot drift from the syntax it
  documents.

### Fixed

- **[anodized] Repaired the examples in `REFERENCE.md`** - Several examples still used the
  pre-0.6 postcondition syntax and did not compile; others warned, or asserted a comparison
  the type already guaranteed.
### Fixed

- **[anodized-core] `#[cfg]` on a condition is honored in every check mode** - The guard was
  emitted only alongside the printing path, so a condition gated off by its `#[cfg]` was still
  checked under `--cfg anodized_panic` without `--cfg anodized_print`.

## 0.6.0 (2026 Aug 08)

### Breaking Changes

- **[anodized] Revised function-spec syntax** - `captures:` now uses assignments (`pattern = expression`) instead of `expression as pattern`; the automatic `old_` capture binding is removed. Postcondition output bindings now belong to each `ensures:` clause, for example `ensures: |result| [condition_a, condition_b]`; `binds:` and `inspects:` are no longer supported (#173, #178, #179, #180).
- **[anodized] Tightened condition evaluation** - Preconditions and invariants are no longer written as closures, and condition evaluation must implement `Fn`, preventing mutations of captured state (#157, #163).
- **[workspace] Raised the minimum supported Rust version to 1.95** (#158).
- **[anodized-core] Reworked the public specification AST** - `PreCondition` is now `Condition`, postconditions store an optional pattern and expression, and parser terminology/types now use fields (`SpecFields` and `syn::FieldValue`) instead of arguments (`SpecArgs` and `SpecArg`) (#165, #178, #181).

### Added

- **[anodized] `try_call!` and `anodized::result`** - With `--cfg anodized_try` and `--cfg anodized_panic`, wrap a specified call to receive a `Result` that distinguishes precondition and postcondition failures. This enables fuzzing and property-based testing to discard invalid inputs without masking postcondition failures (#156, #158, #159, #160).
- **[examples] Property-based and fuzzing examples for binary search** - Added QuickCheck, proptest, and cargo-fuzz demonstrations using `try_call!` (#166, #167).
- **[anodized] Postcondition closure-array form** - A single output pattern can bind the return value for a group of postconditions, e.g. `ensures: |output| [first_check, second_check]` (#179).
- **[anodized-fmt] Array formatting and comment preservation in arrays** (#140).

### Changed

- **[anodized-core] Improved runtime-check instrumentation and spec embedding** - Checks now enforce `bool` results and use mode-specific configuration internally (#143, #151, #153, #154).
- **[workspace] Expanded CI coverage** - Added checks for test utilities and fuzz targets, including formatter fuzzing (#141, #142).

### Documentation

- **[anodized] Refined the README introduction and positioning** (#155).

## 0.5.1 (2026 May 29)

### Fixed

- **[anodized] Removed a prototyping leftover from trait narrowing build-fail tests**.

### Changed

- **[anodized] Test suite cleanup for Clippy compliance** - Updated test code patterns and lint allowances in test files (#136).
- **[workspace] Updated repository URLs after ownership transfer** - Crate metadata, README links/badges, and diagnostics issue links now point to `anodized-rs/anodized` (#138).

### Documentation

- **[anodized] README refinements** - Clarified project positioning, ecosystem fit, and relationship to existing efforts (#137).

## 0.5.0 (2026 May 28)

### Breaking Changes

- **[anodized] Runtime mode selection moved from Cargo features to compiler `cfg` flags** - Removed `runtime-check-and-panic`, `runtime-check-and-print`, and `runtime-no-check` features; use flags such as `--cfg anodized_panic` or `--cfg anodized_print` instead (#118).
- **[anodized-core] `Backend` became `Config`** - Instrumentation configuration APIs were renamed and cleaned up (#131).

### Added

- **[anodized-logic] Quantifiers `exists` and `forall`** (#115).
- **[anodized] Data specs on `struct` and `enum`** - `#[spec]` now supports data-level invariants on these items (#121).
- **[anodized] Loop specs on `for` and `while`** - Added loop invariant (`maintains`) and loop variant (`decreases`) support (#122).
- **[anodized-core] Trait spec narrowing** - `impl` trait methods can narrow trait-level specs, with checks in place (#124).
- **[anodized-logic] `int` type for embedding integer arithmetic in specs** (#127).
- **[anodized-logic] Material implication and opaque expressions** (#129).
- **[anodized] `cfg(anodized_discard_specs)`** - Allows dropping spec embedding while still compiling normal code (#130).
- **[anodized-core] `fn` qualifiers in spec annotations** (#133).

### Changed

- **[workspace] Introduced the `anodized-macros` crate** - Macro implementation moved out of the `anodized` crate and re-exported from there (#126).
- **[anodized-core] Embedded spec elements now use internal `fn __anodized_*` items** (#120).
- **[anodized-core] `instrument_fn` was generalized to work with function items across contexts** (`ItemFn`, `TraitItemFn`, `ImplItemFn`) (#119).

### Documentation

- **[anodized] README updates for loop specs and related guidance** (#122).
- **[anodized] Split docs into `README` and `REFERENCE`** for clearer user guidance and API details (#134).

## 0.4.0 (2026 Apr 19)

### Breaking Changes

- **[anodized-core] `Condition` became `PreCondition`** - Preconditions are now represented as zero-argument closures.
- **[anodized-core] `Capture.alias` became `Capture.pat`** - Captures now bind with full Rust patterns, not only identifiers. A pattern must still be irrefutable.
- **[anodized-core] `Spec` internals changed** - `Spec` now tracks source span and exposes helper methods such as `is_empty` and `spec_err`.

### Added

- **[anodized] Trait specs support** - `#[spec]` on traits and trait `impl` blocks, with compile-fail checks for unsupported placements (#71).
- **[anodized-core] Patterns in `captures`** - Destructuring in captures is now supported, e.g. `captures: expr as (a, b)` (#100).
- **[anodized-fmt] Experimental: New formatter package** - Added a dedicated formatter (CLI and library) for `#[spec]` attributes, including `--check` mode and TOML config support (#92).
- **[anodized-core] `SpecArgs` for permissive parsing** - Added a raw spec representation that can hold partially-valid specs for tooling workflows (#94).

### Fixed

- **[anodized] Preconditions no longer permit early function exit** - Preconditions are wrapped in closures to prevent `return` from escaping the instrumented function (#69).
- **[anodized] Captures no longer permit early function exit** - Each capture expression is evaluated in a closure to prevent `return` from escaping the instrumented function (#107).
- **[anodized-core] Improved `captures` diagnostics and parsing behavior** - Simplified parsing and clearer errors for invalid capture forms (#101).
- **[anodized-fmt] Preserve comments in formatted specs** - Comments inside `#[spec]` attributes are now preserved (with documented skip cases) (#106).

### Changed

- **[workspace] CI now checks formatting and lints** (#105).
- **[workspace] Minor maintenance and Clippy cleanups** (#99).
- **[workspace] VS Code rust-analyzer settings updates** - Added setup for selecting spec runtime behavior (#68, #95).

### Documentation

- **[anodized] Documented trait specs feature in README** (#104).
- **[anodized] Quickstart and Cargo feature docs updates** (#89, #91).
- **[workspace] Added crates.io typo redirect package/docs** (#67).

## 0.3.0 (2025 Dec 11)

### Breaking Changes

- **Fixed handling `return` statements in a `fn` body** - Returns no longer bypass runtime checks.
- **Must explicitly select a `runtime-*` behavior** - See below.

### Added

- Support for capturing entry-time values via `captures:`.
- Explicit `runtime-*` behavior settings: `check-and-panic`, `check-and-print`, `no-check`.

## 0.2.1 (2025 Aug 26)

### Breaking Changes

- **Renamed `#[contract]` to `#[spec]`** - The main attribute macro has been renamed from `contract` to `spec` to avoid confusion with blockchain smart contracts and improve discoverability (#23)
- **Renamed `Contract` type to `Spec`** - The exposed data type follows the same renaming for consistency

### Added

- Support for `#[cfg(...)]` attributes on individual conditions, allowing conditional compilation of runtime checks (#14)
- Support for array syntax in conditions - multiple conditions can now be specified as arrays: `requires: [cond1, cond2]` (#9)
- Improved error messages for misplaced macro attributes (#19)

### Changed

- Enforced parameter order in the `#[spec]` macro - conditions must now appear in the order: `requires`, `maintains`, `ensures` (#12)
- Improved internal parsing architecture (#15)
- Enhanced test coverage with unit tests for instrumentation (#18)

### Documentation

- Completely revised README with clearer examples and motivation (#21)
- Added project logo (#20)
- Clarified dual MIT/Apache-2.0 licensing (#13)

## 0.1.0 (2025 Aug 20)

Initial release
