use proc_macro2::TokenStream;
use syn::{ExprForLoop, ExprWhile, parse_quote};

use crate::{LoopSpec, instrument::Mode, test_util::assert_tokens_eq};

#[test]
fn embed_spec_expr_while() {
    let while_spec: LoopSpec = parse_quote! {
        maintains: [
            INVAR_1,
            INVAR_2,
        ],
        decreases: DECREASES_1,
    };
    let mut expr_while: ExprWhile = parse_quote! {
        while WHILE_COND {
            LOOP_BODY
        }
    };

    let expected: TokenStream = parse_quote! {
        while WHILE_COND {
            let __anodized_loop_maintains = || -> bool {
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| { INVAR_1 });
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| { INVAR_2 });
                __anodized_clause_1 && __anodized_clause_2
            };
            let __anodized_loop_decreases = || {
                let __anodized_value_1 = (|| DECREASES_1)();
                (__anodized_value_1)
            };
            LOOP_BODY
        }
    };

    Mode::EmbedSpecs.instrument_expr_while(while_spec, &mut expr_while);
    let observed = expr_while;

    assert_tokens_eq(&observed, &expected);
}

#[test]
fn embed_spec_expr_for() {
    let for_spec: LoopSpec = parse_quote! {
        maintains: [
            INVAR_1,
            INVAR_2,
        ],
    };
    let mut expr_for_loop: ExprForLoop = parse_quote! {
        for FOR_VAR in FOR_EXPR {
            LOOP_BODY
        }
    };

    let expected: TokenStream = parse_quote! {
        for FOR_VAR in FOR_EXPR {
            let __anodized_loop_maintains = || -> bool {
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| { INVAR_1 });
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| { INVAR_2 });
                __anodized_clause_1 && __anodized_clause_2
            };
            let __anodized_loop_decreases = || {};
            LOOP_BODY
        }
    };

    Mode::EmbedSpecs.instrument_expr_for_loop(for_spec, &mut expr_for_loop);
    let observed = expr_for_loop;

    assert_tokens_eq(&observed, &expected);
}
