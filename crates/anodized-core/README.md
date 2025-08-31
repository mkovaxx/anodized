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
       (* not a typo: at most one `binds` *)
       , [ binds_param ]
       , [ ensures_params ];

requires_params  = { requires_param };
maintains_params = { maintains_param };
ensures_params   = { ensures_param };

requires_param  = [ cfg_attr ] , `requires:` , conditions, `,`;
maintains_param = [ cfg_attr ] , `maintains:` , conditions, `,`;
binds_param     = `binds:` , pattern, `,`;
ensures_param   = [ cfg_attr ] , `ensures:` , post_conds, `,`;

conditions = expr | condition_list;
condition_list = `[` , expr , { `,` , expr } , [ `,` ] , `]`;

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
- `closed_expr` provides an explicit binding where `pattern` binds the return value and `expr` is evaluated with that binding; type checking will fail if the expression does not evaluate to `bool`.
- `pattern` refers to any valid Rust [`pattern`](https://doc.rust-lang.org/reference/patterns.html); type checking will fail if its type does not match the function's return value.
- `settings` is the content of the [`cfg`](https://doc.rust-lang.org/reference/conditional-compilation.html) attribute (e.g. `test`, `debug_assertions`).

## Runtime Checks

When runtime checks are enabled (by default, in debug builds), the `#[spec]` macro transforms the function body to inject assertions. This process, known as instrumentation, follows a clear pattern.

Given an original function like this:

```rust,ignore
#[spec(
    requires: <PRECONDITION>,
    maintains: <INVARIANT>,
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

    // 2. The original function body is executed
    let __anodized_output = {
        <BODY>
    };

    // 3. Invariants and postconditions are checked
    assert!(<INVARIANT>, "Post-invariant failed: <INVARIANT>");
    // Postcondition is checked with the return value bound to a pattern
    {
        let <PATTERN> = __anodized_output;
        assert!(<POSTCONDITION>, "Postcondition failed: <PATTERN> => <POSTCONDITION>");
    }

    // 4. The result is returned
    __anodized_output
}
```

This transformation happens hygienically, meaning the `__anodized_output` variable will not conflict with any existing variables in your code. Any `#[cfg]` attributes on conditions are respected, and the corresponding `assert!` will be wrapped in an `if cfg!(...)` block, ensuring that expensive checks can be conditionally compiled. In release builds, this entire instrumentation is disabled, resulting in zero performance overhead.
