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
            FIELD_1: Spec!(&'LT_1 TYPE_3),
            FIELD_2: Spec!(TYPE_1),
            FIELD_3: Spec!([TYPE_4; CONST_1]),
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
        impl<'LT_1, TYPE_1: BOUND_1, const CONST_1: TYPE_2> ::anodized::types::Spec
            for STRUCT<'LT_1, TYPE_1, CONST_1>
        where
            'LT_1: 'LT_2,
        {
            fn predicate(&self) -> bool {
                let __anodized_inv = match self {
                    STRUCT { FIELD_1, FIELD_2, FIELD_3 } => {
                        ::anodized::__::eval_type_spec(FIELD_1)
                            & ::anodized::__::eval_type_spec(FIELD_2)
                            & ::anodized::__::eval_type_spec(FIELD_3)
                    }
                };
                let __anodized_inv = __anodized_inv & ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_inv = __anodized_inv & ::anodized::__::eval::<bool>(|| COND_2);
                __anodized_inv
            }
        }
    };

    let observed = Mode::DEFAULT
        .instrument_item_struct(spec_item_struct.spec, spec_item_struct.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn default_instrument_item_enum() {
    let spec_item_enum: SpecItemEnum = parse_quote! {
        #[spec(maintains: [
            COND_1,
            COND_2,
        ])]
        enum ENUM<'LT_1, TYPE_1: BOUND_1 = DEFAULT_1, const CONST_1: TYPE_2 = DEFAULT_2>
        where
            'LT_1: 'LT_2,
        {
            VARIANT_1(Spec!(&'LT_1 TYPE_2)),
            VARIANT_2 { FIELD_1: Spec!(TYPE_1), FIELD_2: Spec!(TYPE_2) },
            VARIANT_3,
            VARIANT_4(Spec!([TYPE_4; CONST_1])),
        }
    };

    let expected: TokenStream = parse_quote! {
        enum ENUM<'LT_1, TYPE_1: BOUND_1 = DEFAULT_1, const CONST_1: TYPE_2 = DEFAULT_2>
        where
            'LT_1: 'LT_2,
        {
            VARIANT_1(&'LT_1 TYPE_2),
            VARIANT_2 { FIELD_1: TYPE_1, FIELD_2: TYPE_2 },
            VARIANT_3,
            VARIANT_4([TYPE_4; CONST_1]),
        }

        #[doc(hidden)]
        #[allow(warnings)]
        impl<'LT_1, TYPE_1: BOUND_1, const CONST_1: TYPE_2> ::anodized::types::Spec
            for ENUM<'LT_1, TYPE_1, CONST_1>
        where
            'LT_1: 'LT_2,
        {
            fn predicate(&self) -> bool {
                use ENUM::*;
                let __anodized_inv = match self {
                    VARIANT_1(field_0) => ::anodized::__::eval_type_spec(field_0),
                    VARIANT_2 { FIELD_1, FIELD_2 } => {
                        ::anodized::__::eval_type_spec(FIELD_1)
                            & ::anodized::__::eval_type_spec(FIELD_2)
                    }
                    VARIANT_3 => true,
                    VARIANT_4(field_0) => ::anodized::__::eval_type_spec(field_0),
                };
                let __anodized_inv = __anodized_inv & ::anodized::__::eval::<bool>(|| COND_1);
                let __anodized_inv = __anodized_inv & ::anodized::__::eval::<bool>(|| COND_2);
                __anodized_inv
            }
        }
    };

    let observed = Mode::DEFAULT
        .instrument_item_enum(spec_item_enum.spec, spec_item_enum.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}

#[test]
fn embed_spec_item_type_checks_specified_fields() {
    let spec_item_enum: SpecItemEnum = parse_quote! {
        #[spec]
        enum ENUM {
            TUPLE(Spec!(CHILD_1), CHILD_2, Spec!(CHILD_3)),
            STRUCT {
                included: Spec!(CHILD_4),
                omitted: CHILD_5,
            },
            UNIT,
        }
    };

    let expected: TokenStream = parse_quote! {
        enum ENUM {
            TUPLE(CHILD_1, CHILD_2, CHILD_3),
            STRUCT {
                included: CHILD_4,
                omitted: CHILD_5,
            },
            UNIT,
        }

        #[doc(hidden)]
        #[allow(warnings)]
        impl ::anodized::types::Spec for ENUM {
            fn predicate(&self) -> bool {
                use ENUM::*;
                let __anodized_inv = match self {
                    TUPLE(field_0, _, field_2) => {
                        ::anodized::__::eval_type_spec(field_0)
                            & ::anodized::__::eval_type_spec(field_2)
                    }
                    STRUCT { included, .. } => ::anodized::__::eval_type_spec(included),
                    UNIT => true,
                };
                __anodized_inv
            }
        }
    };

    let observed = Mode::EMBED_SPECS
        .instrument_item_enum(spec_item_enum.spec, spec_item_enum.node)
        .unwrap();
    assert_tokens_eq(&observed, &expected);
}
