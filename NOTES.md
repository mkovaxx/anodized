# Clones Feature - COMPLETED ✓

## Current Status
- **Branch**: `mate-support-clones` - Ready to merge to main
- **Feature**: COMPLETE - All implementation, tests, and documentation done
- **Recent commits**: 
  - `76a83f3` - Fix example to include all specification parameters including binds
  - `908ec81` - Add clones feature documentation to README

## Feature Overview
Support for saving entry-time values of function arguments using a `clones:` parameter, similar to the `old()` operator in the `contracts` crate. This allows postconditions to compare final state with initial state.

## Design
```rust
#[spec(
    clones: [
        count,                           // Shorthand: implicitly `as old_count`
        self.items.len() as orig_len,   // Explicit alias for complex expressions
    ],
    ensures: [
        count == old_count + 1,
        self.items.len() == orig_len + 1,
    ],
)]
fn push(&mut self, item: T) { /* ... */ }
```

## Implementation Summary

### 1. Data Structures ✓
- Added `CloneBinding` struct with `expr: Expr` and `alias: Ident` fields
- Added `clones: Vec<CloneBinding>` field to `Spec` struct
- Updated parameter ordering to: `requires`, `maintains`, `clones`, `binds`, `ensures`

### 2. Parser Implementation ✓
- Created `interpret_as_clone_binding()` function to interpret single expressions as clone bindings
- Created `interpret_array_as_clone_bindings()` function to interpret arrays as lists of bindings
- Handles three cases:
  - Simple identifier: `count` → auto-generates `old_count`
  - Cast expression: `value as old_val` → explicit alias
  - Array of bindings: `[count, value as old_val]` → multiple bindings
- Added check to disallow multiple `clones:` parameters (must use array for multiple values)
- Added check to disallow `#[cfg]` attributes on `clones:`

### 3. Code Generation ✓
- Implemented in `instrument_fn_body()`
- **CRITICAL FIX**: Uses tuple assignment to ensure cloned values are NOT accessible to function body
  - Evaluates clones and body together: `let (alias1, alias2, output) = (expr1.clone(), expr2.clone(), body)`
  - This prevents scope leakage - body cannot reference clone aliases
- Uses iterator chains for clean code generation
- Works correctly even when no clones present (generates single-element tuple)
- Aliases available to postconditions through lexical scoping

### 4. Tests ✓

#### Parser Tests
- `test_parse_clones_simple_identifier` - simple identifier with auto-generated alias
- `test_parse_clones_identifier_with_alias` - identifier with explicit alias
- `test_parse_clones_array` - array of mixed bindings
- `test_parse_clones_with_all_clauses` - integration with other spec parameters
- `test_parse_clones_out_of_order` - parameter ordering validation
- `test_parse_clones_array_expression` - array literal with alias (edge case)

#### Error Case Tests
- `test_parse_clones_complex_expr_without_alias` - complex expressions require explicit alias
- `test_parse_clones_method_call_without_alias` - method calls require explicit alias
- `test_parse_clones_binary_expr_without_alias` - binary expressions require explicit alias
- `test_parse_clones_array_with_complex_expr_no_alias` - mixed array with complex expr
- `test_parse_cfg_on_clones` - cfg attribute not allowed on clones
- `test_parse_multiple_clones` - multiple clones parameters not allowed

#### Instrumentation Test
- `test_instrument_with_clones` - verifies correct code generation

#### Integration Tests
- `crates/anodized/tests/clones_feature.rs` - end-to-end feature test
- `crates/anodized/tests/execution_order.rs` - verifies exact execution order of all spec clauses
- `crates/anodized/tests/block_expressions.rs` - tests block expressions in spec conditions
- `crates/anodized/tests/compile-fail/clones_scope_isolation.rs` - compile-fail test verifying scope isolation

## Key Design Decisions

### Evaluation Order
- Preconditions run first (check true entry state)
- Clones capture after preconditions (avoiding corrupted state checks)
- Function body executes
- Postconditions can use cloned values

### Scope Management
- Clone aliases NOT available to preconditions/maintains (enforced by lexical scoping)
- Tuple destructuring prevents scope creep between clone expressions
- Span preservation for good error messages

### Restrictions
- At most one `clones:` parameter allowed (use array for multiple)
- No `#[cfg]` attributes on clones (must execute unconditionally)
- Complex expressions require explicit alias

## Implementation Insights

1. **Scope Isolation Solution**: Using tuple assignment `let (clones..., output) = (clone_exprs..., body)` ensures cloned values cannot be accessed by the function body, preventing semantic changes to user code

2. **Format String Fix**: Changed assert macros from compile-time interpolation to runtime formatting to handle block expressions with braces correctly

3. **Iterator Chains**: Refactored to use iterator chains instead of mutable vectors for cleaner code generation

4. **Test Patterns**: Use opaque placeholders (EXPR_1, ALIAS_1, CONDITION_1) in tests to verify transformation logic

## What Was Completed

### Documentation Updates ✓
- Added comprehensive clones feature documentation to README.md (see sections on "Capturing Entry-Time Values with `clones`")
- Updated parameter ordering documentation to include `clones` in the correct sequence
- Added examples showing:
  - Simple identifiers with auto-generated `old_` prefix
  - Complex expressions requiring explicit aliases  
  - Integration with all spec parameters (requires, maintains, clones, binds, ensures)
- Integration tests in `crates/anodized/tests/clones_feature.rs` serve as additional examples

### Key Files Modified
- `crates/anodized/README.md` - Added clones documentation
- `NOTES.md` - Updated to reflect completion status

## Next Steps
1. **Merge to main** - The feature is complete and ready to merge
2. **Version bump** - Consider updating version in Cargo.toml files for release
3. **Changelog** - Update CHANGELOG.md with the new clones feature

## Future Work (Not Blocking)

### Import System Compatibility Investigation
- **Critical**: When importing from the `contracts` crate (which uses separate attributes like `#[requires]`, `#[invariant]`, `#[ensures]`), the ORDER of attributes matters for instrumentation
- The `old()` function in `contracts` captures values at function entry
- The `contracts` crate uses `ret` as the name for the return value (vs our `output`)
- Need to determine: Does attribute order affect when `old()` captures occur relative to precondition/invariant checks?
- **KEY QUESTION**: If attributes are reordered, does behavior change? E.g.:
  ```rust
  #[requires(x > 0)]
  #[ensures(ret == old(x) + 1)]
  fn foo(x: i32) -> i32 { ... }
  ```
  vs.
  ```rust
  #[ensures(ret == old(x) + 1)]
  #[requires(x > 0)]
  fn foo(x: i32) -> i32 { ... }
  ```
  Does reversing the order change when `old(x)` is captured relative to checking `x > 0`?
- This affects our clones implementation - we currently capture AFTER preconditions
- May need to analyze how `contracts` crate handles this and ensure compatibility
- Consider: Should import system preserve original attribute order or normalize to our ordering?

