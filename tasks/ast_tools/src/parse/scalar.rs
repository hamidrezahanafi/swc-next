//! Metadata for scalar enums, independent of the NodeId-based AST unions.

use std::collections::HashSet;

use syn::{Expr, ItemEnum, Lit, Meta};

use crate::schema::{EstreeValue, ScalarEnum, ScalarEnumVariant};

pub(super) fn parse(item: &ItemEnum) -> Option<ScalarEnum> {
    let attrs: Vec<_> = item
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("estree_enum"))
        .collect();
    let attr = *attrs.first()?;
    assert_eq!(attrs.len(), 1, "Duplicate estree_enum on {}", item.ident);
    assert!(
        !item.attrs.iter().any(|attr| attr.path().is_ident("ast")),
        "Scalar enums cannot use #[ast]: {}",
        item.ident
    );
    assert!(
        item.generics.params.is_empty(),
        "Scalar enums cannot be generic: {}",
        item.ident
    );
    let size = super::parse_repr_size(&item.attrs)
        .expect("estree_enum requires repr(u8), repr(u16), or repr(u32)");
    assert!(
        matches!(size, 1 | 2 | 4),
        "Unsupported scalar enum representation: {}",
        item.ident
    );
    let max = (1u64 << (size * 8)) - 1;
    let mut no_ts_def = false;
    let mut optional_none = None;
    if matches!(attr.meta, Meta::List(_)) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("no_ts_def") && !no_ts_def {
                no_ts_def = true;
            } else if meta.path.is_ident("optional_none") && optional_none.is_none() {
                optional_none = Some(
                    meta.value()?
                        .parse::<syn::LitInt>()?
                        .base10_parse::<u32>()?,
                );
            } else {
                return Err(meta.error("Unknown or duplicate scalar enum attribute"));
            }
            Ok(())
        })
        .expect("Invalid estree_enum attribute");
    } else {
        assert!(
            matches!(attr.meta, Meta::Path(_)),
            "Invalid estree_enum attribute"
        );
    }
    if let Some(none) = optional_none {
        assert!(
            u64::from(none) <= max,
            "optional_none exceeds enum representation: {}",
            item.ident
        );
    }
    let mut next = 0u64;
    let mut discriminants = HashSet::new();
    let mut values = HashSet::new();
    let variants: Vec<_> = item
        .variants
        .iter()
        .map(|variant| {
            assert!(
                matches!(variant.fields, syn::Fields::Unit),
                "Scalar enum variants must be fieldless: {}::{}",
                item.ident,
                variant.ident
            );
            if let Some((_, expr)) = &variant.discriminant {
                let Expr::Lit(syn::ExprLit {
                    lit: Lit::Int(lit), ..
                }) = expr
                else {
                    panic!(
                        "Scalar enum discriminants must be unsigned integer literals: {}::{}",
                        item.ident, variant.ident
                    );
                };
                next = lit
                    .base10_parse()
                    .expect("Invalid scalar enum discriminant");
            }
            assert!(
                next <= max,
                "Scalar enum discriminant exceeds representation: {}::{}",
                item.ident,
                variant.ident
            );
            let discriminant = next as u32;
            next += 1;
            assert!(
                discriminants.insert(discriminant),
                "Duplicate scalar enum discriminant: {}::{}",
                item.ident,
                variant.ident
            );
            assert_ne!(
                optional_none,
                Some(discriminant),
                "optional_none collides with a variant: {}::{}",
                item.ident,
                variant.ident
            );
            let mut value = None;
            let mut skip = false;
            for attr in variant
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("estree"))
            {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") && !skip {
                        skip = true;
                    } else if meta.path.is_ident("value") && value.is_none() {
                        let expr = meta.value()?.parse::<Expr>()?;
                        value = Some(match expr {
                            Expr::Lit(syn::ExprLit {
                                lit: Lit::Str(lit), ..
                            }) => EstreeValue::String(lit.value()),
                            Expr::Lit(syn::ExprLit {
                                lit: Lit::Bool(lit),
                                ..
                            }) => EstreeValue::Bool(lit.value),
                            Expr::Path(path) if path.path.is_ident("null") => EstreeValue::Null,
                            _ => {
                                return Err(meta.error(
                                    "Scalar ESTree values must be strings, booleans, or null",
                                ));
                            }
                        });
                    } else {
                        return Err(meta.error("Unknown or duplicate scalar variant attribute"));
                    }
                    Ok(())
                })
                .expect("Invalid scalar variant attribute");
            }
            let value = value.unwrap_or_else(|| {
                panic!("Missing ESTree value: {}::{}", item.ident, variant.ident)
            });
            assert!(
                values.insert(value.js()),
                "Duplicate scalar ESTree value: {}::{}",
                item.ident,
                variant.ident
            );
            assert!(
                optional_none.is_none() || !matches!(value, EstreeValue::Null),
                "Optional scalar enums reserve null for None: {}",
                item.ident
            );
            ScalarEnumVariant {
                discriminant,
                value,
                skip,
            }
        })
        .collect();
    assert!(
        variants.iter().any(|variant| !variant.skip),
        "Scalar enum has no output variants: {}",
        item.ident
    );
    Some(ScalarEnum {
        size,
        no_ts_def,
        optional_none,
        variants,
    })
}
