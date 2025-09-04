<img width="100" alt="Anodized Logo" src="https://raw.githubusercontent.com/mkovaxx/anodized/main/assets/logo.svg">

# Anodized Core

This crate provides the core data structures and logic for the [Anodized](https://github.com/mkovaxx/anodized) ecosystem.

## Who Is This For?

- **If you want to add specifications to your code...**

  You're looking for the [`anodized`](https://crates.io/crates/anodized) crate, which provides the `#[spec]` macro.

- **If you're building a tool and want to work with Anodized specifications...**

  You're in the right place! This crate provides the necessary components to parse and interact with Anodized specification annotations.

- **If you're looking for blockchain smart contracts...**

  _"These are not the **contracts** you're looking for."_ 🤖 But don't leave yet! While Anodized is about Design by Contract (not blockchain), it can still help make your smart contracts more robust through formal specifications.

This crate is the foundational layer that enables interoperability between different tools in the Anodized correctness ecosystem.

***

## Specification Syntax

The `#[spec]` attribute's parameters follow a specific grammar, which is formally defined using EBNF as follows.

```ebnf
params = [ requires_params ]
       , [ maintains_params ]
       (* not a typo: at most one `captures:` *)
       , [ captures_param ]
       (* not a typo: at most one `binds:` *)
       , [ binds_param ]
       , [ ensures_params ];

requires_params  = { requires_param };
maintains_params = { maintains_param };
ensures_params   = { ensures_param };

requires_param  = [ cfg_attr ] , `requires:` , conditions, `,`;
maintains_param = [ cfg_attr ] , `maintains:` , conditions, `,`;
captures_param  = `captures:` , captures, `,`;
binds_param     = `binds:` , pattern, `,`;
ensures_param   = [ cfg_attr ] , `ensures:` , post_conds, `,`;

conditions = expr | condition_list;
condition_list = `[` , expr , { `,` , expr } , [ `,` ] , `]`;

captures = capture_expr | capture_list;
capture_list = `[` , capture_expr , { `,` , capture_expr } , [ `,` ] , `]`;
capture_expr = expr | (expr , `as` , ident);

post_conds = post_cond_expr | post_cond_list;
post_cond_list = `[` , post_cond_expr , { `,` , post_cond_expr } , [ `,` ] , `]`;
post_cond_expr = expr | closed_expr;
closed_expr = pattern , `=>` , expr;

cfg_attr = `#[cfg(` , settings , `)]`;
```

**Notes:**

- The last `,` is optional.
- The `params` rule defines a sequence of optional parameter groups that must appear in the specified order.
- `expr` refers to a Rust [`expression`](https://doc.rust-lang.org/reference/expressions.html); type checking will fail if it does not evaluate to `bool`.
- `closed_expr` has a `pattern` to explicitly bind the return value, and `expr` is evaluated with that in scope. Type checking will fail if the expression does not evaluate to `bool`.
- `pattern` refers to any valid Rust [`pattern`](https://doc.rust-lang.org/reference/patterns.html); type checking will fail if its type does not match the function's return value.
- `settings` is the content of the [`cfg`](https://doc.rust-lang.org/reference/conditional-compilation.html) attribute (e.g. `test`, `debug_assertions`).

## Runtime Checks

The `#[spec]` macro transforms the function body by injecting runtime checks as `assert!` statements, which are active in all builds by default. This process, known as instrumentation, follows a clear pattern.

Given an original function like this:

```rust,ignore
#[spec(
    requires: <PRECONDITION>,
    maintains: <INVARIANT>,
    captures: <CAPTURE_EXPR> as <ALIAS>,
    ensures: <PATTERN> => <POSTCONDITION>,
)]
fn my_function(<ARGUMENTS>) -> <RETURN_TYPE> {
    <BODY>
}
```

The macro rewrites the body to be conceptually equivalent to the following:

```rust,ignore
fn my_function(<ARGUMENTS>) -> <RETURN_TYPE> {
    // 1. Preconditions and invariants are checked
    assert!(<PRECONDITION>, "Precondition failed: <PRECONDITION>");
    assert!(<INVARIANT>, "Pre-invariant failed: <INVARIANT>");

    // 2. Values are captured and the original function body is executed
    // Note: captures and body execution happen in a single tuple assignment
    // to ensure captured values aren't accessible to the function body
    let (<ALIAS>, __anodized_output): (_, <RETURN_TYPE>) = (<CAPTURE_EXPR>, {
        <BODY>
    });

    // 3. Invariants and postconditions are checked
    // The captured value is available to postconditions
    assert!(<INVARIANT>, "Post-invariant failed: <INVARIANT>");
    // Postcondition is checked with the return value bound by a pattern
    {
        let <PATTERN> = __anodized_output;
        assert!(<POSTCONDITION>, "Postcondition failed: <PATTERN> => <POSTCONDITION>");
    }

    // 4. The result is returned
    __anodized_output
}
```

Note that the `__anodized_output` will be in scope for the postconditions, but referring to it is not recommended.

When a condition has a `#[cfg(...)]` attribute, the corresponding `assert!` is wrapped in an `if cfg!(...)` block. This follows standard Rust `#[cfg]` semantics—the check only runs when the configuration predicate is true. Without a `#[cfg]` attribute, the `assert!` behaves exactly like a plain `assert!` in your code—always active unless you compile with optimizations that specifically remove assertions.
