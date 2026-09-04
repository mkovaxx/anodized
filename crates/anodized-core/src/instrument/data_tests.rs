use crate::{
    instrument::Mode,
    test_util::{SpecItemEnum, SpecItemStruct, assert_tokens_eq},
};

use proc_macro2::TokenStream;
use syn::parse_quote;

#[test]
fn embed_spec_item_struct() {
    let spec_item_struct: SpecItemStruct = parse_quote! {
        #[spec(maintains: [
            COND_1,
            COND_2,
        ])]
        struct STRUCT<'LT_1, TYPE_1: BOUND_1 = DEFAULT_1, const CONST_1: TYPE_2 = DEFAULT_2>
        where
            'LT_1: 'LT_2,
        {
            FIELD_1: &'LT_1 TYPE_3,
            FIELD_2: TYPE_1,
            FIELD_3: [TYPE_4; CONST_1],
        }
    };

    let expected: TokenStream = parse_quote! {
        struct STRUCT<'LT_1, TYPE_1: BOUND_1 = DEFAULT_1, const CONST_1: TYPE_2 = DEFAULT_2>
        where
            'LT_1: 'LT_2,
        {
            FIELD_1: &'LT_1 TYPE_3,
            FIELD_2: TYPE_1,
            FIELD_3: [TYPE_4; CONST_1],
        }

        #[doc(hidden)]
        #[allow(warnings)]
        impl<'LT_1, TYPE_1: BOUND_1, const CONST_1: TYPE_2> STRUCT<'LT_1, TYPE_1, CONST_1>
        where
            'LT_1: 'LT_2,
        {
            fn __anodized_data_maintains(&self) -> bool {
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| COND_2);
                __anodized_clause_1 && __anodized_clause_2
            }
        }
    };

    let observed = Mode::EmbedSpecs
        .instrument_item_struct(spec_item_struct.spec, spec_item_struct.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn embed_spec_item_enum() {
    let spec_item_enum: SpecItemEnum = parse_quote! {
        #[spec(maintains: [
            COND_1,
            COND_2,
        ])]
        enum ENUM<'LT_1, TYPE_1: BOUND_1 = DEFAULT_1, const CONST_1: TYPE_2 = DEFAULT_2>
        where
            'LT_1: 'LT_2,
        {
            VARIANT_1(&'LT_1 TYPE_2),
            VARIANT_2 { FIELD_2: TYPE_1 },
            VARIANT_3,
            VARIANT_4([TYPE_4; CONST_1]),
        }
    };

    let expected: TokenStream = parse_quote! {
        enum ENUM<'LT_1, TYPE_1: BOUND_1 = DEFAULT_1, const CONST_1: TYPE_2 = DEFAULT_2>
        where
            'LT_1: 'LT_2,
        {
            VARIANT_1(&'LT_1 TYPE_2),
            VARIANT_2 { FIELD_2: TYPE_1 },
            VARIANT_3,
            VARIANT_4([TYPE_4; CONST_1]),
        }

        #[doc(hidden)]
        #[allow(warnings)]
        impl<'LT_1, TYPE_1: BOUND_1, const CONST_1: TYPE_2> ENUM<'LT_1, TYPE_1, CONST_1>
        where
            'LT_1: 'LT_2,
        {
            fn __anodized_data_maintains(&self) -> bool {
                use ENUM::*;
                let __anodized_clause_1 = ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_clause_2 = ::anodized::__::eval::<bool>(|| COND_2);
                __anodized_clause_1 && __anodized_clause_2
            }
        }
    };

    let observed = Mode::EmbedSpecs
        .instrument_item_enum(spec_item_enum.spec, spec_item_enum.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}
