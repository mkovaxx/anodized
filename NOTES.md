# Postcondition Syntax Migration Notes

## Summary
Successfully migrated postcondition syntax from closure-style (`|pattern| expression`) to explicit binding style (`pattern => expression`) throughout the Anodized codebase.

## Key Changes Made

### 1. Parser Implementation (`crates/anodized-core/src/lib.rs`)
- Removed support for closure syntax entirely
- Implemented `PostConditionExpr` internal type for parsing
- Uses fork-based parsing approach:
  ```rust
  impl Parse for PostConditionExpr {
      fn parse(input: ParseStream) -> Result<Self> {
          let fork = input.fork();
          // Try to parse as pattern => expr
          if Pat::parse_single(&fork).is_ok() && fork.peek(Token![=>]) {
              let pattern = Pat::parse_single(input)?;
              input.parse::<Token![=>]>()?;
              Ok(PostConditionExpr {
                  pattern: Some(pattern),
                  expr: input.parse()?,
              })
          } else {
              // Parse as naked expression
              Ok(PostConditionExpr {
                  pattern: None,
                  expr: input.parse()?,
              })
          }
      }
  }
  ```
- Handles both naked expressions and pattern bindings in arrays using `parse_terminated`

### 2. Syntax Examples
- **Old syntax**: `|output| output > 0`, `|(a, b)| a <= b`
- **New syntax**: `output => output > 0`, `(a, b) => a <= b`
- **Naked expressions**: Still work as before (e.g., `output > 0` implies `output => output > 0`)

### 3. Error Message Format
- Changed from: `"Postcondition failed: | pattern | expression"`
- Changed to: `"Postcondition failed: pattern => expression"`

### 4. Documentation Updates
- Removed all references to "closure" terminology
- Now uses "pattern binding" or "explicit binding" terminology
- Updated all code examples in READMEs and doctests

### 5. Test Updates
- Updated all test expectations to use new error message format
- Fixed integration tests in `crates/anodized/tests/`:
  - `pattern_in_closure.rs` (ironically named, but still uses patterns)
  - `rename_return_value.rs`
  - `default_output_pattern.rs`
- Updated instrumentation tests in `crates/anodized-core/src/test_instrument_fn.rs`

## Technical Decisions

### Why Fork-Based Parsing?
- Patterns like `ref output` or `(a, b)` cannot be parsed as expressions first
- Need to speculatively try pattern parsing without consuming tokens
- Fork allows us to check if pattern parsing succeeds before committing

### Why Not Use `Arm` Type Directly?
Initially tried using `syn::Arm` for parsing, but:
- `Arm` consumes trailing commas which interferes with array parsing
- We don't support guards anyway
- Simpler to just parse `pattern => expression` directly

### Pattern Type Annotations
- Rust's `match` doesn't support type annotations in patterns (e.g., `output: (bool, i32)`)
- Documentation was updated to remove incorrect examples with type annotations
- Users should use simple patterns without types

## Current Branch State
- Branch: `mate-tighten-readme-examples`
- All tests passing
- Documentation fully updated
- Ready for merge

## Commit History (Key Commits)
1. "Change postcondition syntax from closure to explicit binding style"
2. "Fix postcondition array parsing to handle both naked and pattern expressions"
3. "Simplify PostConditionExpr parser to avoid fork" (later reverted)
4. "Use fork-based parsing for PostConditionExpr to handle all pattern types"
5. "Update documentation to use pattern binding terminology instead of closure"

## Files Modified
- `/Users/k/prog/anodized/crates/anodized-core/src/lib.rs` - Core parser
- `/Users/k/prog/anodized/crates/anodized-core/src/test_parse_spec.rs` - Parser tests
- `/Users/k/prog/anodized/crates/anodized-core/src/test_instrument_fn.rs` - Instrumentation tests
- `/Users/k/prog/anodized/crates/anodized-core/src/test_util.rs` - Test utilities
- `/Users/k/prog/anodized/crates/anodized/README.md` - Main documentation
- `/Users/k/prog/anodized/crates/anodized-core/README.md` - Core documentation
- `/Users/k/prog/anodized/crates/anodized/tests/*.rs` - Integration tests

## Next Steps (If Any)
- Could consider supporting `if` guards in patterns (currently rejected with error)
- Could improve error messages when pattern parsing fails
- Documentation could be expanded with more complex pattern examples