use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr, ExprLit, Fields, GenericArgument, Item, ItemEnum, ItemStruct, Lit, LitInt, PathArguments,
    Type, parse_quote, token::Eq,
};

pub fn expand_ast(repr: bool, nested: bool, id: Option<syn::Ident>, item: Item) -> TokenStream {
    assert!(!nested || repr, "`nested` requires `repr`");
    match item {
        Item::Enum(item_enum) => expand_enum(repr, nested, item_enum),
        Item::Struct(item_struct) if !repr && !nested => expand_struct(item_struct, id),
        Item::Struct(_) => panic!("`#[ast(repr)]` is only valid on AST enums"),
        _ => panic!("AST type must be enum or struct"),
    }
}

fn expand_struct(mut item: ItemStruct, id: Option<syn::Ident>) -> TokenStream {
    let name = item.ident.clone();
    let Some(_) = id else {
        return quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
            pub struct #name(pub(crate) crate::NodeId<#name>);
        };
    };

    if name != "Class" {
        let id = id.expect("pooled AST nodes require a typed ID");
        return quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
            pub struct #name(pub(crate) crate::#id);
        };
    }
    let Fields::Named(fields) = &mut item.fields else {
        panic!("dedicated AST objects must use named fields");
    };
    for field in &mut fields.named {
        field.vis = parse_quote!(pub(crate));
        let Type::Path(path) = &field.ty else {
            continue;
        };
        let Some(segment) = path.path.segments.last() else {
            continue;
        };
        if segment.ident != "Vec" {
            continue;
        }
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            continue;
        };
        let Some(GenericArgument::Type(inner)) = arguments.args.first() else {
            continue;
        };
        field.ty = parse_quote!(crate::TypedNodeRange<#inner>);
    }
    quote! {
        #[derive(Debug, Copy, Clone)]
        #item
    }
}

fn expand_enum(repr: bool, nested: bool, mut item: ItemEnum) -> TokenStream {
    let name = item.ident.clone();
    let data_name = format_ident!("{name}Data");
    item.ident = data_name;

    // Dedicated Class objects live in Vec<Class>. AST unions carry the typed
    // common owner ID, whose payload stores the dense class-vector index.
    for variant in &mut item.variants {
        let Some(field) = variant.fields.iter_mut().next() else {
            continue;
        };
        let Type::Path(field_ty) = &field.ty else {
            continue;
        };
        if field_ty
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Class")
        {
            field.ty = parse_quote!(crate::ClassId);
        }
    }

    let repr_attr = repr.then(|| {
        if nested {
            quote!(#[repr(C, u32)])
        } else {
            quote!(#[repr(C, u8)])
        }
    });
    if repr {
        for (index, variant) in item.variants.iter_mut().enumerate() {
            assert!(
                variant.discriminant.is_none(),
                "`#[ast(repr)]` does not support explicit discriminants"
            );
            variant.discriminant = Some(if nested {
                let value = LitInt::new(&(index + 1).to_string(), variant.ident.span());
                (
                    Eq::default(),
                    Expr::Lit(ExprLit {
                        attrs: Vec::new(),
                        lit: Lit::Int(value),
                    }),
                )
            } else {
                let field = variant
                    .fields
                    .iter()
                    .next()
                    .expect("AST enum variants must contain one field");
                let Type::Path(field_ty) = &field.ty else {
                    panic!("AST enum variant fields must be path types");
                };
                let node_kind = &field_ty
                    .path
                    .segments
                    .last()
                    .expect("AST enum variant field paths must not be empty")
                    .ident;
                let node_kind = if node_kind == "ClassId" {
                    format_ident!("Class")
                } else {
                    node_kind.clone()
                };
                (
                    Eq::default(),
                    parse_quote!(crate::NodeKind::#node_kind as u8),
                )
            });
        }
    }

    let constructors = item.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let field = variant
            .fields
            .iter()
            .next()
            .expect("AST enum variants must contain one field");
        let field_ty = &field.ty;
        quote! {
            #[allow(non_snake_case)]
            #[inline]
            pub fn #variant_name(node: #field_ty) -> Self {
                Self(node.ast_id())
            }
        }
    });

    quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct #name(pub(crate) crate::AstId);

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #repr_attr
        #item

        impl #name {
            #(#constructors)*
        }
    }
}
