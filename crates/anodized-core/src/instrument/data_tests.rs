use crate::{DataSpec, instrument::Mode, test_util::assert_tokens_eq};

use proc_macro2::TokenStream;
use syn::{ItemEnum, ItemStruct, parse_quote};

#[test]
fn embed_spec_item_struct() {
    let struct_spec: DataSpec = parse_quote! {
        maintains: [
            COND_1,
            COND_2,
        ],
    };
    let item_struct: ItemStruct = parse_quote! {
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
        .instrument_item_struct(struct_spec, item_struct)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn embed_spec_item_enum() {
    let struct_spec: DataSpec = parse_quote! {
        maintains: [
            COND_1,
            COND_2,
        ],
    };
    let item_enum: ItemEnum = parse_quote! {
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
        .instrument_item_enum(struct_spec, item_enum)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}
