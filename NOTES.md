# Clones Feature Implementation Progress

## Current Branch
`mate-support-clones`

## Feature Overview
Implementing support for saving entry-time values of function arguments using a `clones:` parameter, similar to the `old()` operator in the `contracts` crate.

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

## Completed Work

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

### 3. Parser Edge Cases ✓
- Discovered that `value as alias` parses as `Expr::Cast`, not separate tokens
- Solved ambiguity between `[a, b, c]` (array of bindings) vs `[a, b, c] as slice` (array expression)
- Solution: If top-level expr is array, try interpreting elements as bindings; propagate first error if any fail

### 4. Test Infrastructure ✓
- Updated `assert_spec_eq` to compare `clones` field
- Refactored test utilities to use pattern matching for compile-time safety
- Added comprehensive parser tests for all cases

### 5. Tests Written ✓
- `test_parse_clones_simple_identifier` - simple identifier with auto-generated alias
- `test_parse_clones_identifier_with_alias` - identifier with explicit alias
- `test_parse_clones_array` - array of mixed bindings
- `test_parse_clones_with_all_clauses` - integration with other spec parameters
- `test_parse_clones_out_of_order` - parameter ordering validation
- `test_parse_clones_array_expression` - array literal with alias (edge case)

## Remaining Work

### 1. Code Generation
- Generate clone statements in `instrument_fn_body()` after preconditions/invariants
- Each binding becomes: `let alias = expr.clone();`
- Use hygienic identifiers to avoid collisions

### 2. Scope Management
- Make cloned identifiers available to postcondition closures
- Ensure they're NOT available to preconditions or maintains

### 3. Error Case Tests
- Test missing alias for complex expressions
- Test duplicate aliases
- Test invalid expressions

### 4. Documentation
- Update README with clones feature
- Add examples to documentation

## Key Insights

1. **Parser Ambiguity**: The main challenge was distinguishing between `[a, b, c]` as multiple bindings vs. an array expression. Solved by attempting to interpret array elements as bindings first (common case), falling back to requiring explicit alias.

2. **Cast Expression**: Rust parses `ident as alias` as a single `Expr::Cast`, not separate tokens. This simplified our implementation once discovered.

3. **Interpretation vs Parsing**: Clean separation between parsing (getting an `Expr`) and interpretation (converting to `CloneBinding`).

4. **Test Utilities**: Using pattern matching in test utilities ensures compile-time safety when adding new struct fields.

## Next Steps

The next major task is implementing the code generation in `instrument_fn_body()` to actually generate the clone statements and make them available to postconditions.

## Commit History (on branch)
- Initial data structures and field additions
- Parser implementation with cast expression handling  
- Test infrastructure updates
- Array interpretation for list of bindings
- All parser tests passing