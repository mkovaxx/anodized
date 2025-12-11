<img width="100" alt="Anodized Logo" src="https://raw.githubusercontent.com/mkovaxx/anodized/main/assets/logo.svg">

# Anodized Core

This crate is the interoperability layer for tools connected to the [Anodized](https://github.com/mkovaxx/anodized) specification system.

## Who Is This For?

- **If you want to add specifications to your code...**

  You're looking for the [`anodized`](https://crates.io/crates/anodized) crate, which provides the `#[spec]` macro.

- **If you're building a tool and want to work with Anodized specifications...**

  You're in the right place! This crate provides the necessary components to parse and interact with Anodized specification annotations.

- **If you're looking for blockchain smart contracts...**

  _"These are not the **contracts** you're looking for."_ 🤖 But don't leave yet! While Anodized is about Design by Contract (not blockchain), it can still help make your smart contracts more robust through formal specifications.

---

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
post_cond_expr = expr | closure_expr;

cfg_attr = `#[cfg(` , settings , `)]`;
```

**Notes:**

- The last `,` is optional.
- The `params` rule defines a sequence of optional parameter groups that must appear in the specified order.
- `expr` is a Rust [`expression`](https://doc.rust-lang.org/reference/expressions.html); type checking will fail if it does not evaluate to `bool`.
- `closure_expr` is a Rust [`closure`](https://doc.rust-lang.org/reference/expressions/closure-expr.html) that receives the function's return value as a reference; type checking will fail if it does not evaluate to `bool`.
- `pattern` is an irrefutable Rust [`pattern`](https://doc.rust-lang.org/reference/patterns.html); type checking will fail if its type does not match the function's return value.
- `settings` is the content of the [`cfg`](https://doc.rust-lang.org/reference/conditional-compilation.html) attribute (e.g. `test`, `debug_assertions`).

## Runtime Checks

The `#[spec]` macro transforms the function body by injecting runtime checks whose behavior is controlled by `runtime-*` feature settings. This process, known as instrumentation, follows a clear pattern.

Given an original function like this:

```rust,ignore
#[spec(
    requires: <PRECONDITION>,
    maintains: <INVARIANT>,
    captures: <CAPTURE_EXPR> as <ALIAS>,
    ensures: |<PATTERN>| <POSTCONDITION>,
)]
fn my_function(<ARGUMENTS>) -> <RETURN_TYPE> {
    <BODY>
}
```

The macro rewrites the body to be conceptually equivalent to the following:

```rust,ignore
fn my_function(<ARGUMENTS>) -> <RETURN_TYPE> {
    // 1. Preconditions and invariants are checked
    check!(<PRECONDITION>, "Precondition failed: <PRECONDITION>");
    check!(<INVARIANT>, "Pre-invariant failed: <INVARIANT>");

    // 2. Values are captured and the original function body is executed
    // Note 1: captures and body execution happen in a single tuple assignment
    //         to ensure captured values aren't accessible to the function body
    // Note 2: the body is evaluated in a closure, so returns inside the body
    //         do not bypass postcondition checks
    let (<ALIAS>, __anodized_output): (_, <RETURN_TYPE>) = (
        <CAPTURE_EXPR>,
        (|| { <BODY> })(),
    );

    // 3. Invariants and postconditions are checked
    // Note: Captured values are available to postconditions
    check!(<INVARIANT>, "Post-invariant failed: <INVARIANT>");
    // Postcondition is checked by invoking the closure with a reference to the return value
    check!(
        (|<PATTERN>: &<RETURN_TYPE>| <POSTCONDITION>)(&__anodized_output),
        "Postcondition failed: | <PATTERN> | <POSTCONDITION>",
    );

    // 4. The result is returned
    __anodized_output
}
```

Note that `__anodized_output` will be in scope for postconditions, but referring to it is strongly discouraged.

When a condition has a `#[cfg(...)]` attribute, the corresponding `check!` is wrapped in an `if cfg!(...)` block. This follows standard Rust `#[cfg]` semantics: the check only runs when the configuration predicate is true. Without a `#[cfg]` attribute, the `check!` behaves according to the `runtime-*` feature setting.
