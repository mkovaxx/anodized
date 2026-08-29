use syn::{Pat, parse_quote};

use crate::{
    instrument::patterns::{IdentGenerator, TamePat, tame_pattern},
    test_util::assert_tame_pat_eq,
};

#[test]
fn ident_is_inv() {
    let pat: Pat = parse_quote! { ident };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { ident },
        parse_quote! { ident },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn mut_ident_is_inv() {
    let pat: Pat = parse_quote! { mut a };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { mut a },
        parse_quote! { a },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_with_mut_ident_is_inv() {
    let pat: Pat = parse_quote! { (mut a, b) };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { (mut a, b) },
        parse_quote! { (a, b) },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn ident_at_subpat_is_inv() {
    let pat: Pat = parse_quote! { x @ (a, b) };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { x @ (a, b) },
        parse_quote! { x },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn ident_at_pair_of_ident_wild_is_inv() {
    let pat: Pat = parse_quote! { x @ (_, b) };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { x @ (__anodized_ident_1, b) },
        parse_quote! { x },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn ref_ident_is_brw() {
    let pat: Pat = parse_quote! { ref ident };
    let expected = Ok(TamePat::Borrowing(parse_quote! { ref ident }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn only_wild_is_brw() {
    let pat: Pat = parse_quote! { _ };
    let expected = Ok(TamePat::Borrowing(parse_quote! { _ }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn pair_of_ref_and_move_is_err() {
    let pat: Pat = parse_quote! { (ref a, b) };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns cannot mix move and `ref` bindings",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn pair_of_ref_and_wild_is_brw() {
    let pat: Pat = parse_quote! { (ref a, _) };
    let expected = Ok(TamePat::Borrowing(parse_quote! { (ref a, _) }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn pair_of_move_and_wild_is_inv() {
    let pat: Pat = parse_quote! { (a, _) };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { (a, __anodized_ident_1) },
        parse_quote! { (a, __anodized_ident_1) },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_with_refs_on_both_sides_of_rest_is_brw() {
    let pat: Pat = parse_quote! { (ref first, .., ref last) };
    let expected = Ok(TamePat::Borrowing(
        parse_quote! { (ref first, .., ref last) },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_with_moves_on_both_sides_of_rest_is_err() {
    let pat: Pat = parse_quote! { (first, .., last) };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns containing `..` must bind only by `ref`",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_with_only_rest_is_brw() {
    let pat: Pat = parse_quote! { (..) };
    let expected = Ok(TamePat::Borrowing(parse_quote! { (..) }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn struct_with_move_and_wild_is_inv() {
    let pat: Pat = parse_quote! { Point { x, y: _ } };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { Point { x, y: __anodized_ident_1 } },
        parse_quote! { Point { x, y: __anodized_ident_1 } },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn struct_with_ref_and_wild_is_brw() {
    let pat: Pat = parse_quote! { Point { x: _, y: ref y } };
    let expected = Ok(TamePat::Borrowing(
        parse_quote! { Point { x: _, y: ref y } },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn struct_with_ref_and_rest_is_brw() {
    let pat: Pat = parse_quote! { Point { x: ref x, .. } };
    let expected = Ok(TamePat::Borrowing(parse_quote! { Point { x: ref x, .. } }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn struct_with_move_and_rest_is_err() {
    let pat: Pat = parse_quote! { Point { x, .. } };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns containing `..` must bind only by `ref`",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn struct_with_ref_and_move_is_err() {
    let pat: Pat = parse_quote! { Point { x: ref x, y } };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns cannot mix move and `ref` bindings",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_struct_with_move_and_wild_is_inv() {
    let pat: Pat = parse_quote! { Point(_, x) };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { Point(__anodized_ident_1, x) },
        parse_quote! { Point(__anodized_ident_1, x) },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_struct_with_ref_and_wild_is_brw() {
    let pat: Pat = parse_quote! { Point(ref x, _) };
    let expected = Ok(TamePat::Borrowing(parse_quote! { Point(ref x, _) }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_struct_with_ref_and_rest_is_brw() {
    let pat: Pat = parse_quote! { Point(.., ref x) };
    let expected = Ok(TamePat::Borrowing(parse_quote! { Point(.., ref x) }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_struct_with_move_and_rest_is_err() {
    let pat: Pat = parse_quote! { Point(x, ..) };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns containing `..` must bind only by `ref`",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn tuple_struct_with_ref_and_move_is_err() {
    let pat: Pat = parse_quote! { Point(x, ref y) };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns cannot mix move and `ref` bindings",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn slice_with_move_and_wild_is_inv() {
    let pat: Pat = parse_quote! { [first, _] };
    let expected = Ok(TamePat::Invertible(
        parse_quote! { [first, __anodized_ident_1] },
        parse_quote! { [first, __anodized_ident_1] },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn slice_with_ref_and_wild_is_brw() {
    let pat: Pat = parse_quote! { [_, ref last] };
    let expected = Ok(TamePat::Borrowing(parse_quote! { [_, ref last] }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn slice_with_ref_and_rest_is_brw() {
    let pat: Pat = parse_quote! { [.., ref last] };
    let expected = Ok(TamePat::Borrowing(parse_quote! { [.., ref last] }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn slice_with_move_and_rest_is_err() {
    let pat: Pat = parse_quote! { [first, ..] };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns containing `..` must bind only by `ref`",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn slice_with_refs_on_both_sides_of_rest_is_brw() {
    let pat: Pat = parse_quote! { [ref first, ref inside @ .., ref last] };
    let expected = Ok(TamePat::Borrowing(
        parse_quote! { [ref first, ref inside @ .., ref last] },
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn slice_with_only_rest_is_brw() {
    let pat: Pat = parse_quote! { [..] };
    let expected = Ok(TamePat::Borrowing(parse_quote! { [..] }));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}

#[test]
fn slice_with_ref_and_move_is_err() {
    let pat: Pat = parse_quote! { [ref first, second] };
    let expected = Err(syn::Error::new_spanned(
        &pat,
        "inside `#[spec]`, patterns cannot mix move and `ref` bindings",
    ));

    let mut id_gen = IdentGenerator::new();
    let observed = tame_pattern(&mut id_gen, pat);
    assert_tame_pat_eq(&observed, &expected);
}
