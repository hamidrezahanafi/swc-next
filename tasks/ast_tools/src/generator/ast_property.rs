use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::ast_node::concrete_node_kinds;
use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstStructField, AstType, Schema, StorageDomain},
    util::{
        EXTRA_DATA_ID_BIT_OFFSET, InlineLayout, InlineStorageMode, calculate_inline_layout,
        generate_field_to_u32, generate_field_to_u64, generate_u32_to_field, generate_u64_to_field,
        safe_ident,
    },
};

pub fn ast_property(schema: &Schema) -> RawOutput {
    let mut impls = TokenStream::new();
    for ty in schema.types.iter() {
        match ty {
            AstType::Struct(ast_struct) => {
                impls.extend(generate_property_for_struct(ast_struct, schema))
            }
            AstType::Enum(ast_enum) => impls.extend(generate_property_for_enum(ast_enum, schema)),
            _ => continue,
        };
    }

    let output = quote! {
            #![allow(unused, clippy::useless_conversion, clippy::identity_op, clippy::erasing_op, clippy::let_and_return)]
            use crate::*;
            use oxc_index::Idx;

            #impls
    };

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_property"),
        tokens: output,
    }
    .into()
}

fn generate_property_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);

    if let StorageDomain::Dedicated { name: storage, .. } = &ast.storage {
        assert_eq!(storage, "class");
        return quote! {
            impl ClassId {
                #[inline]
                pub fn class_type(self, ast: &Ast<'_>) -> ClassType {
                    ast.class_data(self).class_type
                }
                #[inline]
                pub fn set_class_type(self, ast: &mut Ast<'_>, value: ClassType) {
                    ast.class_data_mut(self).class_type = value;
                }
                #[inline]
                pub fn decorators(self, ast: &Ast<'_>) -> TypedNodeRange<Decorator> {
                    ast.class_data(self).decorators
                }
                #[inline]
                pub fn set_decorators(self, ast: &mut Ast<'_>, value: TypedNodeRange<Decorator>) {
                    ast.class_data_mut(self).decorators = value;
                }
                #[inline]
                pub fn id(self, ast: &Ast<'_>) -> Option<BindingIdentifier> {
                    ast.class_data(self).id
                }
                #[inline]
                pub fn set_id(self, ast: &mut Ast<'_>, value: Option<BindingIdentifier>) {
                    ast.class_data_mut(self).id = value;
                }
                #[inline]
                pub fn type_parameters(self, ast: &Ast<'_>) -> Option<TsTypeParameterDeclaration> {
                    ast.class_data(self).type_parameters
                }
                #[inline]
                pub fn set_type_parameters(self, ast: &mut Ast<'_>, value: Option<TsTypeParameterDeclaration>) {
                    ast.class_data_mut(self).type_parameters = value;
                }
                #[inline]
                pub fn super_class(self, ast: &Ast<'_>) -> Option<Expr> {
                    ast.class_data(self).super_class
                }
                #[inline]
                pub fn set_super_class(self, ast: &mut Ast<'_>, value: Option<Expr>) {
                    ast.class_data_mut(self).super_class = value;
                }
                #[inline]
                pub fn super_type_arguments(self, ast: &Ast<'_>) -> Option<TsTypeParameterInstantiation> {
                    ast.class_data(self).super_type_arguments
                }
                #[inline]
                pub fn set_super_type_arguments(self, ast: &mut Ast<'_>, value: Option<TsTypeParameterInstantiation>) {
                    ast.class_data_mut(self).super_type_arguments = value;
                }
                #[inline]
                pub fn implements(self, ast: &Ast<'_>) -> TypedNodeRange<TsClassImplements> {
                    ast.class_data(self).implements
                }
                #[inline]
                pub fn set_implements(self, ast: &mut Ast<'_>, value: TypedNodeRange<TsClassImplements>) {
                    ast.class_data_mut(self).implements = value;
                }
                #[inline]
                pub fn body_span(self, ast: &Ast<'_>) -> Span {
                    ast.class_data(self).body_span
                }
                #[inline]
                pub fn set_body_span(self, ast: &mut Ast<'_>, value: Span) {
                    ast.class_data_mut(self).body_span = value;
                }
                #[inline]
                pub fn body(self, _ast: &Ast<'_>) -> ClassBody {
                    ClassBody(self)
                }
                #[inline]
                pub fn body_elements(self, ast: &Ast<'_>) -> TypedNodeRange<ClassElement> {
                    ast.class_data(self).body
                }
                #[inline]
                pub fn set_body_elements(self, ast: &mut Ast<'_>, value: TypedNodeRange<ClassElement>) {
                    ast.class_data_mut(self).body = value;
                }
                #[inline]
                pub fn r#abstract(self, ast: &Ast<'_>) -> bool {
                    ast.class_data(self).r#abstract
                }
                #[inline]
                pub fn set_abstract(self, ast: &mut Ast<'_>, value: bool) {
                    ast.class_data_mut(self).r#abstract = value;
                }
                #[inline]
                pub fn declare(self, ast: &Ast<'_>) -> bool {
                    ast.class_data(self).declare
                }
                #[inline]
                pub fn set_declare(self, ast: &mut Ast<'_>, value: bool) {
                    ast.class_data_mut(self).declare = value;
                }
            }
        };
    }

    let mut field_getters = TokenStream::new();
    let mut field_setters = TokenStream::new();

    // Check if this struct uses inline storage
    if let Some(layout) = calculate_inline_layout(ast, schema) {
        generate_inline_property_accessors(
            ast,
            schema,
            &mut field_getters,
            &mut field_setters,
            &layout,
        );
        if layout.mode == InlineStorageMode::Partial {
            generate_extra_data_property_accessors_partial(
                ast,
                schema,
                &mut field_getters,
                &mut field_setters,
                &layout,
            );
        }
    } else {
        generate_extra_data_property_accessors(ast, schema, &mut field_getters, &mut field_setters);
    }

    quote! {
        impl #name {
            #field_getters
            #field_setters
        }
    }
}

/// Generate getters/setters for fields stored in NodePayload.
fn generate_inline_property_accessors(
    ast: &AstStruct,
    schema: &Schema,
    field_getters: &mut TokenStream,
    field_setters: &mut TokenStream,
    layout: &InlineLayout,
) {
    for &(field_idx, byte_offset, byte_size) in &layout.fields {
        let field = &ast.fields[field_idx];
        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id];
        let getter_name = safe_ident(&field.name.to_case(Case::Snake));
        let ret_ty = field_ty.repr_ident(schema);

        let read_expr = generate_inline_read(field_ty, schema, byte_offset, byte_size);
        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #ret_ty {
                #read_expr
            }
        });

        let setter_name = format_ident!("set_{}", field_name);
        let invalidations = generate_invalidations(field);
        let write_expr = generate_inline_write(
            field_ty,
            &field_name,
            byte_offset,
            byte_size,
            schema,
            invalidations,
        );
        field_setters.extend(quote! {
            #[inline]
            pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #ret_ty) {
                #write_expr
            }
        });
    }
}

/// Generate read expression for a field from inline storage using bit operations
fn generate_inline_read(
    field_ty: &AstType,
    schema: &Schema,
    byte_offset: usize,
    byte_size: usize,
) -> TokenStream {
    let extract_expr = generate_extract_bits(byte_offset, byte_size);
    let cast_expr = if byte_size == std::mem::size_of::<u64>() {
        generate_u64_to_field(field_ty, schema)
    } else {
        generate_u32_to_field(field_ty, schema)
    };

    quote! {
        #extract_expr
        #cast_expr
    }
}

/// Generate code to extract a field value using u128 bit operations.
fn generate_extract_bits(byte_offset: usize, byte_size: usize) -> TokenStream {
    let bit_offset = byte_offset * 8;
    if byte_size == std::mem::size_of::<u64>() {
        return quote! {
            let raw =
                (unsafe { ast.typed_node_payload_unchecked(self).0 } >> #bit_offset) as u64;
        };
    }

    let bit_size = byte_size * 8;
    let mask = if bit_size >= 32 {
        u32::MAX
    } else {
        (1u32 << bit_size) - 1
    };
    quote! {
        let raw =
            ((unsafe { ast.typed_node_payload_unchecked(self).0 } >> #bit_offset) as u32) & #mask;
    }
}

/// Generate write expression for a field to inline storage using bit operations
fn generate_inline_write(
    field_ty: &AstType,
    field_name: &syn::Ident,
    byte_offset: usize,
    byte_size: usize,
    schema: &Schema,
    invalidations: TokenStream,
) -> TokenStream {
    let insert_expr = generate_insert_bits(byte_offset, byte_size);
    let change_guard = (!invalidations.is_empty()).then(|| {
        let extract_expr = generate_extract_bits(byte_offset, byte_size);
        quote! {
            #extract_expr
            if raw == field_val {
                return;
            }
        }
    });

    if byte_size == std::mem::size_of::<u64>() {
        let to_u64 = generate_field_to_u64(field_ty, field_name, quote!(ast), schema);
        quote! {
            let field_val: u64 = #to_u64;
            #change_guard
            #insert_expr
            #invalidations
        }
    } else {
        let to_u32 = generate_field_to_u32(field_ty, field_name, quote!(ast), schema);
        quote! {
            let field_val: u32 = #to_u32;
            #change_guard
            #insert_expr
            #invalidations
        }
    }
}

/// Generate code to insert a field value using bit operations (read-modify-write)
fn generate_insert_bits(byte_offset: usize, byte_size: usize) -> TokenStream {
    let bit_size = byte_size * 8;
    let mask = if bit_size >= 128 {
        u128::MAX
    } else {
        (1u128 << bit_size) - 1
    };
    let bit_offset = byte_offset * 8;
    let clear_mask = !(mask << bit_offset);
    quote! {
        let payload = unsafe { ast.typed_node_payload_mut_unchecked(self) };
        payload.0 =
            (payload.0 & #clear_mask)
            | (((field_val as u128) & #mask) << #bit_offset);
    }
}

/// Generate getters/setters for extra_data storage (original implementation)
fn generate_extra_data_property_accessors(
    ast: &AstStruct,
    schema: &Schema,
    field_getters: &mut TokenStream,
    field_setters: &mut TokenStream,
) {
    for (offset, field) in ast.fields.iter().enumerate() {
        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id].repr_ident(schema);

        let getter_name = safe_ident(&field.name.to_case(Case::Snake));
        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #field_ty {
                let offset = unsafe {
                    ExtraDataId::from_usize_unchecked(
                        ((ast.typed_node_payload_unchecked(self).0 >> #EXTRA_DATA_ID_BIT_OFFSET) as u32 as usize)
                            .wrapping_add(#offset)
                    )
                };

                debug_assert!(offset.index() < ast.extra_data.len());
                unsafe {
                    ExtraDataCompact::from_extra_data(
                        *ast.extra_data.as_slice().get_unchecked(offset.index()),
                        ast,
                    )
                }
            }
        });

        field_setters.extend(generate_extra_data_setter(
            field,
            &field_name,
            field_ty,
            offset,
        ));
    }
}

fn generate_extra_data_property_accessors_partial(
    ast: &AstStruct,
    schema: &Schema,
    field_getters: &mut TokenStream,
    field_setters: &mut TokenStream,
    layout: &InlineLayout,
) {
    let mut extra_offset = 0usize;

    for (field_index, field) in ast.fields.iter().enumerate() {
        if layout
            .fields
            .iter()
            .any(|(inline_index, _, _)| *inline_index == field_index)
        {
            continue;
        }

        let field_name = format_ident!("{}", field.name);
        let field_ty = &schema.types[field.type_id].repr_ident(schema);
        let getter_name = safe_ident(&field.name.to_case(Case::Snake));
        let offset = extra_offset;

        field_getters.extend(quote! {
            #[inline]
            pub fn #getter_name<'a>(self, ast: &crate::Ast<'a>) -> #field_ty {
                let offset = unsafe {
                    ExtraDataId::from_usize_unchecked(
                        ((ast.typed_node_payload_unchecked(self).0 >> #EXTRA_DATA_ID_BIT_OFFSET) as u32 as usize)
                            .wrapping_add(#offset)
                    )
                };

                debug_assert!(offset.index() < ast.extra_data.len());
                unsafe {
                    ExtraDataCompact::from_extra_data(
                        *ast.extra_data.as_slice().get_unchecked(offset.index()),
                        ast,
                    )
                }
            }
        });

        field_setters.extend(generate_extra_data_setter(
            field,
            &field_name,
            field_ty,
            offset,
        ));

        extra_offset += 1;
    }
}

fn generate_extra_data_setter(
    field: &AstStructField,
    field_name: &syn::Ident,
    field_ty: &TokenStream,
    offset: usize,
) -> TokenStream {
    let setter_name = format_ident!("set_{}", field_name);
    let invalidations = generate_invalidations(field);
    if invalidations.is_empty() {
        return quote! {
            #[inline]
            pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #field_ty) {
                let offset = unsafe {
                    ExtraDataId::from_usize_unchecked(
                        ((ast.typed_node_payload_unchecked(self).0 >> #EXTRA_DATA_ID_BIT_OFFSET) as u32 as usize)
                            .wrapping_add(#offset)
                    )
                };

                debug_assert!(offset.index() < ast.extra_data.len());
                unsafe {
                    *ast.extra_data.as_mut_slice().get_unchecked_mut(offset.index()) = #field_name.to_extra_data(ast);
                };
            }
        };
    }

    quote! {
        #[inline]
        pub fn #setter_name<'a>(self, ast: &mut crate::Ast<'a>, #field_name: #field_ty) {
            let field_value = #field_name.to_extra_data(ast);
            let offset = unsafe {
                ExtraDataId::from_usize_unchecked(
                    ((ast.typed_node_payload_unchecked(self).0 >> #EXTRA_DATA_ID_BIT_OFFSET) as u32 as usize)
                        .wrapping_add(#offset)
                )
            };

            debug_assert!(offset.index() < ast.extra_data.len());
            let previous = unsafe {
                *ast.extra_data.as_slice().get_unchecked(offset.index())
            };
            if unsafe { previous.other } == unsafe { field_value.other } {
                return;
            }
            unsafe {
                *ast.extra_data.as_mut_slice().get_unchecked_mut(offset.index()) = field_value;
            };
            #invalidations
        }
    }
}

fn generate_invalidations(field: &AstStructField) -> TokenStream {
    let invalidations = field.attrs.invalidates.iter().map(|target| {
        let setter_name = format_ident!("set_{target}");
        quote! {
            self.#setter_name(ast, None);
        }
    });
    quote!(#(#invalidations)*)
}

fn generate_property_for_enum(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    let data_name = format_ident!("{}Data", ast.name);
    let data_fn_name = format_ident!("{}_data", ast.name.to_case(Case::Snake));
    let has_contextual_variants = ast.name == "ExportDefaultDeclarationKind";

    let mut is_variant = TokenStream::new();
    let mut as_variant = TokenStream::new();

    for variant in ast.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);
        let is_fn_name = format_ident!("is_{}", variant.name.to_case(Case::Snake));
        let kind_check = if has_contextual_variants {
            quote!(matches!(ast.#data_fn_name(self), #data_name::#variant_name(_)))
        } else {
            let node_kinds: Vec<_> = concrete_node_kinds(variant.type_id.unwrap(), schema)
                .into_iter()
                .map(|name| format_ident!("{name}"))
                .collect();
            if let [node_kind] = node_kinds.as_slice() {
                quote!(ast.node_kind(self.0) == NodeKind::#node_kind)
            } else {
                quote!(matches!(ast.node_kind(self.0), #(NodeKind::#node_kinds)|*))
            }
        };
        is_variant.extend(quote! {
            #[inline]
            pub fn #is_fn_name(self, ast: &Ast<'_>) -> bool {
                #kind_check
            }
        });

        let as_fn_name = format_ident!("as_{}", variant.name.to_case(Case::Snake));
        let variant_type = schema.types[variant.type_id.unwrap()].repr_ident(schema);
        as_variant.extend(quote! {
            #[inline]
            pub fn #as_fn_name(self, ast: &Ast<'_>) -> Option<#variant_type> {
                match ast.#data_fn_name(self) {
                    #data_name::#variant_name(it) => Some(it),
                    _ => None,
                }
            }
        });
    }

    quote! {
        impl #name {
            #is_variant
            #as_variant
        }
    }
}
