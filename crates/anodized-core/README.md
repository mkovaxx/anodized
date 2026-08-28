<img width="100" alt="Anodized Logo" src="https://raw.githubusercontent.com/anodized-rs/anodized/main/assets/logo.svg">

# Anodized Core

This crate is the interoperability layer for tools connected to the [Anodized](https://github.com/anodized-rs/anodized) specification system.

## Who Is This For?

- **If you want to add specifications to your code...**

  You're looking for the [`anodized`](https://crates.io/crates/anodized) crate, which provides the `#[spec]` macro.

- **If you're building a tool and want to work with Anodized specifications...**

  You're in the right place! This crate provides the necessary components to parse and interact with Anodized specification annotations.

- **If you're looking for blockchain smart contracts...**

  > _"These are not the **contracts** you're looking for."_ 🤖

  But don't leave yet! While Anodized is about Design by Contract (not blockchain), it can still help make your smart contracts more robust through formal specifications.

---

## Specification Syntax

The `#[spec]` attribute's fields follow a specific grammar, which is formally defined using EBNF as follows.

```ebnf
fields = [ qualifiers ]
         [ requires_fields ]
       , [ maintains_fields ]
       (* not a typo: at most one `captures:` *)
       , [ captures_field ]
       , [ ensures_fields ];

qualifiers = { qualifier };
qualifier = `functional` | `pure` | `total`
          | `deterministic` | `effectfree` | `infallible` | `terminating`;

requires_fields  = { requires_field };
maintains_fields = { maintains_field };
ensures_fields   = { ensures_field };

requires_field  = [ cfg_attr ] , `requires:` , conditions, `,`;
maintains_field = [ cfg_attr ] , `maintains:` , conditions, `,`;
captures_field  = `captures:` , captures, `,`;
ensures_field   = [ cfg_attr ] , `ensures:` , postconds, `,`;

conditions = expr | condition_list;
condition_list = `[` , expr , { `,` , expr } , [ `,` ] , `]`;

captures = capture_stmt | capture_list;
capture_list = `[` , capture_stmt , { `,` , capture_stmt } , [ `,` ] , `]`;
capture_stmt = pattern , `=` , expr;

postconds = postcond_expr | postcond_list | postcond_list_closure;
postcond_list = `[` , postcond_expr , { `,` , postcond_expr } , [ `,` ] , `]`;
postcond_expr = expr | postcond_closure;
postcond_closure = `|` , pattern , `|` , expr;
postcond_list_closure = `|` , pattern , `|` , condition_list;

cfg_attr = `#[cfg(` , settings , `)]`;
```

**Notes:**

- The last `,` is optional.
- The `fields` rule defines a sequence of optional field groups that must appear in the specified order.
- `expr` is a Rust [`expression`](https://doc.rust-lang.org/reference/expressions.html).
- `pattern` is an irrefutable Rust [`pattern`](https://doc.rust-lang.org/reference/patterns.html).
- `settings` is the content of the [`cfg`](https://doc.rust-lang.org/reference/conditional-compilation.html) attribute (e.g. `test`, `debug_assertions`).
- Every valid spec field can be parsed as a Rust [`struct` expression field](https://doc.rust-lang.org/reference/expressions/struct-expr.html#grammar-StructExprField).

## Instrumentation

The `#[spec]` macro transforms the function body by injecting code, determined by compiler `cfg` settings (e.g. `anodized_panic`, `anodized_print`). This process, known as instrumentation, follows a clear pattern.

Given an original function like this:

```rust,ignore
#[spec(
    requires: <PRECONDITION>,
    maintains: <INVARIANT>,
    captures: <ALIAS> = <CAPTURE_EXPR>,
    ensures: |<OUTPUT_BINDING>| <POSTCONDITION>,
)]
fn my_function(<FUNCTION_INPUTS>) -> <RETURN_TYPE> {
    <BODY>
}
```

The macro rewrites the body to be conceptually equivalent to the following:

```rust,ignore
fn my_function(<FUNCTION_INPUTS>) -> <RETURN_TYPE> {
    // 1. Preconditions and invariants are checked.
    let __anodized_pre = true;
    let __anodized_pre = __anodized_pre & check_condition!(
        <PRECONDITION>,
        "precondition failed: <PRECONDITION>",
    );
    let __anodized_pre = __anodized_pre & check_condition!(
        <INVARIANT>,
        "preinvariant failed: <INVARIANT>",
    );
    if !__anodized_pre {
        handle_failure!();
    }

    // 2. Values are captured and the original function body is executed.
    // Note 1: Captures and body execution happen in a single tuple assignment
    //         to ensure captured values aren't accessible to the function body.
    // Note 2: The body is evaluated in a closure, so returns inside the body
    //         do not bypass postcondition checks.
    let (<ALIAS>, __anodized_output): (_, <RETURN_TYPE>) = (
        <CAPTURE_EXPR>,
        (|| { <BODY> })(),
    );

    // 3. Invariants and postconditions are checked.
    // Note 1: Captured values are in scope for postconditions.
    // Note 2: `__anodized_output` is also in scope for postconditions,
    //         but referring to it is strongly discouraged.
    let __anodized_post = true;
    let __anodized_post = __anodized_post & check_condition!(
        <INVARIANT>,
        "postinvariant failed: <INVARIANT>",
    );
    // Each postcondition is evaluated with its output bindings in scope.
    // Note 1: Details of pattern handling are omitted for brevity.
    let __anodized_post = __anodized_post & check_condition!(
        { let <OUTPUT_BINDING> = __anodized_output; <POSTCONDITION> },
        "postcondition failed: | <OUTPUT_BINDING> | <POSTCONDITION>",
    );
    if !__anodized_post {
        handle_failure!();
    }

    // `check_condition!` represents the generated condition expression. Depending on
    // enabled settings, it evaluates the condition, prints a diagnostic, or short-circuits
    // to `true` when no runtime action is enabled.

    // 4. The result is returned
    __anodized_output
}
```

When a condition has a `#[cfg(...)]` attribute, its generated evaluator is individually guarded
with `!cfg!(...) || ...`. This follows standard Rust `#[cfg]` semantics: the condition is evaluated
only when the configuration predicate is true, but the condition remains type-checked in every build
configuration, with details of diagnostics and failure handling determined by the `anodized_*`
settings.
