# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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