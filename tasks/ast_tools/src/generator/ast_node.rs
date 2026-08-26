use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH,
    output::{RawOutput, RustOutput, output_path},
    schema::{AstEnum, AstStruct, AstType, Schema, StorageDomain},
    util::{InlineStorageMode, calculate_inline_layout},
};

pub fn ast_node(schema: &Schema) -> RawOutput {
    let mut node_impls = TokenStream::new();
    let mut data_accessors = TokenStream::new();
    let mut node_kind_variants = TokenStream::new();
    let mut node_kind_enum_checks = TokenStream::new();
    let mut node_kind_extra_data_arms = TokenStream::new();
    let mut node_kind_map_ast_id_arms = TokenStream::new();
    let mut node_kind_store_arms = TokenStream::new();
    let mut node_kind_data_variants = TokenStream::new();
    let mut node_kind_data_arms = TokenStream::new();

    for ty in &schema.types {
        match ty {
            AstType::Struct(ast) => {
                node_impls.extend(generate_node_impls_for_struct(ast, schema));
                let name = format_ident!("{}", ast.name);
                let store_tag = match &ast.storage {
                    StorageDomain::Dedicated { .. } => format_ident!("Class"),
                    StorageDomain::Pooled { .. } => format_ident!("Function"),
                    StorageDomain::Common => canonical_store_tag(&ast.name, schema),
                };
                let handle = ty.repr_ident(schema);
                node_kind_variants.extend(quote!(#name,));
                node_kind_store_arms.extend(quote!(Self::#name => StoreTag::#store_tag,));
                let uses_extra_data = ast.storage.uses_node_payload()
                    && !matches!(
                        calculate_inline_layout(ast, schema),
                        Some(layout) if layout.mode == InlineStorageMode::Full
                    );
                node_kind_extra_data_arms.extend(quote!(Self::#name => #uses_extra_data,));
                node_kind_map_ast_id_arms.extend(generate_map_ast_id_arm(ast, schema));
                node_kind_data_variants.extend(quote!(#name(#handle) = NodeKind::#name as u8,));
                if ast.storage.uses_node_payload() {
                    node_kind_data_arms.extend(quote! {
                        NodeKind::#name => NodeKindData::#name(unsafe {
                            <#name as CommonAstNode>::from_ast_id_unchecked(ast_id)
                        }),
                    });
                } else {
                    node_kind_data_arms.extend(quote! {
                        NodeKind::#name => {
                            NodeKindData::#name(unsafe {
                                <#handle as CommonAstNode>::from_ast_id_unchecked(ast_id)
                            })
                        },
                    });
                }
            }
            AstType::Enum(ast) => {
                node_impls.extend(generate_node_impls_for_enum(ast));
                data_accessors.extend(generate_data_accessor(ast, schema));
                node_kind_enum_checks.extend(generate_node_kind_enum_check(ast, schema));
            }
            _ => {}
        }
    }

    RustOutput {
        path: output_path(AST_CRATE_PATH, "ast_node"),
        tokens: quote! {
            #![allow(unused)]
            use crate::{Ast, GetSpan, SetSpan};
            use crate::{ast::*, node_id::*};

            #[repr(u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum NodeKind {
                #node_kind_variants
                /// Internal vacant slot reserved for transform insertion.
                Hole,
            }

            impl NodeKind {
                #node_kind_enum_checks

                #[inline]
                pub const fn uses_extra_data(self) -> bool {
                    match self {
                        #node_kind_extra_data_arms
                        Self::Hole => false,
                    }
                }

                #[inline]
                pub const fn store_tag(self) -> StoreTag {
                    match self {
                        #node_kind_store_arms
                        Self::Hole => panic!("hole has no canonical AST storage"),
                    }
                }

                pub(crate) fn map_ast_ids(
                    self,
                    payload: &mut crate::NodePayload,
                    extra_data: &mut [crate::ExtraData],
                    mut map: impl FnMut(AstId) -> AstId,
                ) {
                    match self {
                        #node_kind_map_ast_id_arms
                        Self::Hole => {},
                    }
                }
            }

            #[repr(C, u8)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum NodeKindData {
                #node_kind_data_variants
            }

            #node_impls

            impl<'a> Ast<'a> {
                #[inline]
                pub fn kind_data<I: Into<AstId>>(&self, node_id: I) -> NodeKindData {
                    let ast_id = node_id.into();
                    match self.node_kind(ast_id) {
                        #node_kind_data_arms
                        NodeKind::Hole => panic!("hole does not contain an AST node"),
                    }
                }

                #data_accessors
            }
        },
    }
    .into()
}

fn generate_map_ast_id_arm(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    if ast.storage.is_dedicated_object() {
        return quote!(Self::#name => {},);
    }

    let layout = calculate_inline_layout(ast, schema);
    let mut mappings = TokenStream::new();
    let mut extra_offset = 0usize;
    for (field_index, field) in ast.fields.iter().enumerate() {
        let field_ty = &schema.types[field.type_id];
        let contains_ast_id = match field_ty {
            AstType::Struct(_) | AstType::Enum(_) => true,
            AstType::Option(option) => matches!(
                schema.types[option.inner_type_id],
                AstType::Struct(_) | AstType::Enum(_)
            ),
            _ => false,
        };
        let inline_offset = layout.as_ref().and_then(|layout| {
            layout
                .fields
                .iter()
                .find(|(index, _, _)| *index == field_index)
                .map(|(_, offset, _)| *offset)
        });
        if let Some(byte_offset) = inline_offset {
            if contains_ast_id {
                let bit_offset = byte_offset * 8;
                let mask = (u32::MAX as u128) << bit_offset;
                if bit_offset == 0 {
                    mappings.extend(quote! {
                        let raw = payload.0 as u32;
                        let mapped = map(AstId::from_raw(raw)).raw();
                        payload.0 = (payload.0 & !#mask) | mapped as u128;
                    });
                } else {
                    mappings.extend(quote! {
                        let raw = (payload.0 >> #bit_offset) as u32;
                        let mapped = map(AstId::from_raw(raw)).raw();
                        payload.0 = (payload.0 & !#mask) | ((mapped as u128) << #bit_offset);
                    });
                }
            }
        } else {
            if contains_ast_id {
                let offset = extra_offset;
                let index = if offset == 0 {
                    quote!(base)
                } else {
                    quote!(base + #offset)
                };
                mappings.extend(quote! {
                    let base = (payload.0 >> 96) as u32 as usize;
                    let word = unsafe { extra_data[#index].other };
                    let mapped = map(AstId::from_raw(word as u32)).raw();
                    extra_data[#index] = crate::ExtraData {
                        other: (word & !u32::MAX as u64) | mapped as u64,
                    };
                });
            }
            extra_offset += 1;
        }
    }

    quote!(Self::#name => { #mappings },)
}

fn generate_node_kind_enum_check(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let fn_name = format_ident!("is_{}", ast.name.to_case(Case::Snake));
    let mut seen = HashSet::new();
    let node_kinds: Vec<_> = ast
        .variants
        .iter()
        .filter_map(|variant| variant.type_id)
        .flat_map(|type_id| concrete_node_kinds(type_id, schema))
        .filter(|node_kind| seen.insert(node_kind.clone()))
        .map(|node_kind| format_ident!("{node_kind}"))
        .collect();

    quote! {
        #[inline]
        pub const fn #fn_name(self) -> bool {
            matches!(self, #(Self::#node_kinds)|*)
        }
    }
}

fn generate_data_accessor(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let has_nested_enum = ast.variants.iter().any(|variant| {
        variant
            .type_id
            .is_some_and(|type_id| matches!(schema.types[type_id], AstType::Enum(_)))
    });
    let marked_nested = ast._attrs.ast_attrs.contains("nested");
    assert_eq!(
        marked_nested, has_nested_enum,
        "AST enum nesting annotation does not match its variants: {}",
        ast.name
    );

    if has_nested_enum {
        generate_nested_data_accessor(ast, schema)
    } else {
        generate_flat_data_accessor(ast, schema)
    }
}

fn generate_flat_data_accessor(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let wrapper = format_ident!("{}", ast.name);
    let data = format_ident!("{}Data", ast.name);
    let function = format_ident!("{}_data", ast.name.to_case(Case::Snake));
    let kind_check = format_ident!("is_{}", ast.name.to_case(Case::Snake));
    let mut arms = TokenStream::new();

    for variant in &ast.variants {
        let variant_name = format_ident!("{}", variant.name);
        let type_id = variant.type_id.expect("AST enum variant has a payload");
        let payload = schema.types[type_id].repr_ident(schema);
        for node_kind in concrete_node_kinds(type_id, schema) {
            let node_kind = format_ident!("{node_kind}");
            arms.extend(quote! {
                NodeKind::#node_kind => #data::#variant_name(unsafe {
                    <#payload as CommonAstNode>::from_ast_id_unchecked(node.0)
                }),
            });
        }
    }

    quote! {
        #[inline]
        pub fn #function(&self, node: #wrapper) -> #data {
            let kind = self.node_kind(node.0);
            debug_assert!(kind.#kind_check());
            match kind {
                #arms
                _ => unsafe { std::hint::unreachable_unchecked() },
            }
        }
    }
}

fn generate_nested_data_accessor(ast: &AstEnum, schema: &Schema) -> TokenStream {
    let wrapper = format_ident!("{}", ast.name);
    let data = format_ident!("{}Data", ast.name);
    let function = format_ident!("{}_data", ast.name.to_case(Case::Snake));
    let mut seen = HashSet::new();
    let mut arms = TokenStream::new();

    for variant in &ast.variants {
        let Some(type_id) = variant.type_id else {
            continue;
        };
        let variant_name = format_ident!("{}", variant.name);
        let payload = schema.types[type_id].repr_ident(schema);
        let payload_is_dedicated = matches!(
            &schema.types[type_id],
            AstType::Struct(ast) if ast.storage.is_dedicated_object()
        );
        for node_kind_name in concrete_node_kinds(type_id, schema) {
            if !seen.insert(node_kind_name.clone()) {
                continue;
            }
            let node_kind = format_ident!("{node_kind_name}");
            let arm = match (
                ast.name.as_str(),
                variant.name.as_str(),
                node_kind_name.as_str(),
            ) {
                ("ExportDefaultDeclarationKind", "Expr", "Function") => quote! {
                    NodeKind::Function => {
                        let function = unsafe {
                            <Function as CommonAstNode>::from_ast_id_unchecked(node.0)
                        };
                        if function.function_type(self) == FunctionType::FunctionDeclaration {
                            #data::Function(function)
                        } else {
                            #data::Expr(unsafe {
                                <#payload as CommonAstNode>::from_ast_id_unchecked(node.0)
                            })
                        }
                    },
                },
                ("ExportDefaultDeclarationKind", "Expr", "Class") => quote! {
                    NodeKind::Class => {
                        let class = unsafe {
                            <ClassId as CommonAstNode>::from_ast_id_unchecked(node.0)
                        };
                        if class.class_type(self) == ClassType::ClassDeclaration {
                            #data::Class(class)
                        } else {
                            #data::Expr(unsafe {
                                <#payload as CommonAstNode>::from_ast_id_unchecked(node.0)
                            })
                        }
                    },
                },
                (_, _, "Class") if payload_is_dedicated => quote! {
                    NodeKind::Class => {
                        #data::#variant_name(unsafe {
                            <ClassId as CommonAstNode>::from_ast_id_unchecked(node.0)
                        })
                    },
                },
                _ => quote! {
                    NodeKind::#node_kind => #data::#variant_name(unsafe {
                        <#payload as CommonAstNode>::from_ast_id_unchecked(node.0)
                    }),
                },
            };
            arms.extend(arm);
        }
    }

    quote! {
        #[inline]
        pub fn #function(&self, node: #wrapper) -> #data {
            match self.node_kind(node.0) {
                #arms
                _ => unsafe { std::hint::unreachable_unchecked() },
            }
        }
    }
}

pub(crate) fn concrete_node_kinds(type_id: crate::schema::TypeId, schema: &Schema) -> Vec<String> {
    match &schema.types[type_id] {
        AstType::Struct(ast) => vec![ast.name.clone()],
        AstType::Enum(ast) => ast
            .variants
            .iter()
            .filter_map(|variant| variant.type_id)
            .flat_map(|type_id| concrete_node_kinds(type_id, schema))
            .collect(),
        _ => unreachable!("AST enum variants must contain AST nodes"),
    }
}

fn generate_node_impls_for_struct(ast: &AstStruct, schema: &Schema) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    match &ast.storage {
        StorageDomain::Common => {
            let tag = canonical_store_tag(&ast.name, schema);
            let node_impls = generate_common_node_impls(
                &name,
                quote!(NodeId<#name>),
                quote!(ast.span(self.ast_id())),
                quote!(ast.set_span(self.ast_id(), span)),
            );
            quote! {
                impl IdDomain for #name {
                    const TAG: StoreTag = StoreTag::#tag;
                    const ERASED_IS_PACKED: bool = true;
                }
                #node_impls
            }
        }
        StorageDomain::Pooled { id, .. } => {
            let id = format_ident!("{id}");
            generate_common_node_impls(
                &name,
                quote!(#id),
                quote!(ast.span(self.ast_id())),
                quote!(ast.set_span(self.ast_id(), span)),
            )
        }
        StorageDomain::Dedicated { id, .. } => {
            let id = format_ident!("{id}");
            quote! {
                impl AstNode for #id {
                    type Id = #id;

                    #[inline]
                    fn id(self) -> #id {
                        self
                    }
                }

                impl CommonAstNode for #id {
                    #[inline]
                    fn ast_id(self) -> AstId {
                        self.ast_id()
                    }

                    #[inline]
                    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
                        unsafe { <#id as TypedAstId>::from_ast_id_unchecked(ast_id) }
                    }
                }

                impl GetSpan for #id {
                    #[inline]
                    fn span(self, ast: &Ast<'_>) -> crate::Span {
                        ast.span(self.ast_id())
                    }
                }

                impl SetSpan for #id {
                    #[inline]
                    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
                        ast.set_span(self.ast_id(), span)
                    }
                }
            }
        }
    }
}

fn canonical_store_tag(name: &str, schema: &Schema) -> proc_macro2::Ident {
    if matches!(
        name,
        "IdentifierReference"
            | "PrivateIdentifier"
            | "BindingIdentifier"
            | "IdentifierName"
            | "LabelIdentifier"
            | "AssignmentPattern"
            | "BindingRestElement"
            | "ArrayPattern"
            | "ObjectPattern"
            | "BindingProperty"
    ) {
        return format_ident!("NamePattern");
    }

    if enum_contains_node("ClassElement", name, schema) {
        return format_ident!("ClassElement");
    }

    for (enum_name, tag) in [("Expr", "Expression"), ("Stmt", "Statement")] {
        if enum_contains_node(enum_name, name, schema) {
            return format_ident!("{tag}");
        }
    }

    if matches!(name, "VariableDeclarator" | "SwitchCase" | "CatchClause") {
        return format_ident!("Statement");
    }
    if name.starts_with("Ts") || name.starts_with("Jsx") {
        return format_ident!("TypeJsx");
    }
    format_ident!("ProgramAux")
}

fn enum_contains_node(enum_name: &str, name: &str, schema: &Schema) -> bool {
    schema
        .types
        .iter()
        .find_map(|ty| match ty {
            AstType::Enum(ast) if ast.name == enum_name => Some(ast),
            _ => None,
        })
        .is_some_and(|ast| {
            ast.variants
                .iter()
                .filter_map(|variant| variant.type_id)
                .flat_map(|type_id| concrete_node_kinds(type_id, schema))
                .any(|candidate| candidate == name)
        })
}

fn generate_node_impls_for_enum(ast: &AstEnum) -> TokenStream {
    let name = format_ident!("{}", ast.name);
    generate_common_node_impls(
        &name,
        quote!(AstId),
        quote!(ast.span(self.ast_id())),
        quote!(ast.set_span(self.ast_id(), span)),
    )
}

fn generate_common_node_impls(
    name: &proc_macro2::Ident,
    id: TokenStream,
    get_span: TokenStream,
    set_span: TokenStream,
) -> TokenStream {
    quote! {
        impl #name {
            #[inline]
            pub fn node_id(self) -> #id {
                self.0
            }

            #[inline]
            pub fn ast_id(self) -> AstId {
                self.0.ast_id()
            }

            /// # Safety
            ///
            /// ast_id must identify this concrete common node type.
            #[inline]
            pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
                unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
            }
        }

        impl AstNode for #name {
            type Id = #id;

            #[inline]
            fn id(self) -> #id {
                self.0
            }
        }

        impl CommonAstNode for #name {
            #[inline]
            fn ast_id(self) -> AstId {
                self.0.ast_id()
            }

            #[inline]
            unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
                unsafe {
                    <Self as AstNode>::from_id_unchecked(
                        <#id as TypedAstId>::from_ast_id_unchecked(ast_id)
                    )
                }
            }
        }

        impl GetSpan for #name {
            #[inline]
            fn span(self, ast: &Ast<'_>) -> crate::Span {
                #get_span
            }
        }

        impl SetSpan for #name {
            #[inline]
            fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
                #set_span
            }
        }
    }
}
