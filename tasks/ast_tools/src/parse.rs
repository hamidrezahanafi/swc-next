use std::{
    collections::{HashMap, HashSet},
    fs,
};

use indexmap::IndexSet;
use oxc_index::IndexVec;
use quote::ToTokens;
use syn::{
    Attribute, Field, GenericArgument, Ident, Item, ItemEnum, ItemStruct, Meta, PathArguments,
    Type, Variant, punctuated::Punctuated, token::Comma,
};

use crate::schema::{
    AstAttrs, AstEnum, AstEnumVariant, AstFieldAttrs, AstOption, AstPrimitive, AstStruct,
    AstStructField, AstSubRange, AstType, EstreeAddedField, EstreeAttrs, EstreeConcat,
    EstreeFieldValue, EstreeTagCondition, EstreeTagSelector, EstreeValue, Schema, TypeId,
};

struct Parser {
    /// Mapping all type names to their type ids
    type_names: IndexSet<String>,
    /// Types exclude top level structs and enums
    extra_types: IndexVec<TypeId, AstType>,
}

pub fn parse_files(file_paths: &[&str]) -> Schema {
    parse_sources(file_paths.iter().map(|source| {
        fs::read_to_string(source).unwrap_or_else(|_| panic!("Cannot find {source}"))
    }))
}

fn parse_sources(sources: impl IntoIterator<Item = String>) -> Schema {
    struct PrototypeStruct {
        type_id: TypeId,
        item: ItemStruct,
        attrs: AstAttrs,
    }

    struct PrototypeEnum {
        type_id: TypeId,
        item: ItemEnum,
        attrs: AstAttrs,
    }

    enum PrototypeStructOrEnum {
        Struct(PrototypeStruct),
        Enum(PrototypeEnum),
    }

    // Collect declared enums and structs ahead of time and create their types
    let mut type_names = IndexSet::new();
    let mut prototypes = Vec::new();
    let mut repr_sizes = HashMap::new();

    for content in sources {
        let file = syn::parse_file(&content).unwrap();
        for item in file.items {
            match item {
                Item::Struct(item) => {
                    // Filter structs with #[ast]
                    let Some(attrs) = parse_attrs(&item.attrs) else {
                        continue;
                    };

                    let name = item.ident.to_string();
                    let type_id = TypeId::from_usize(type_names.len());
                    type_names.insert(name);
                    prototypes.push(PrototypeStructOrEnum::Struct(PrototypeStruct {
                        type_id,
                        item,
                        attrs,
                    }));
                }
                Item::Enum(item) => {
                    // Collect #[repr(uN)] sizes for all enums (not just #[ast] enums)
                    if let Some(size) = parse_repr_size(&item.attrs) {
                        repr_sizes.insert(item.ident.to_string(), size);
                    }

                    // Filter enums with #[ast]
                    let Some(attrs) = parse_attrs(&item.attrs) else {
                        continue;
                    };

                    let name = item.ident.to_string();
                    let type_id = TypeId::from_usize(type_names.len());
                    type_names.insert(name);
                    prototypes.push(PrototypeStructOrEnum::Enum(PrototypeEnum {
                        type_id,
                        item,
                        attrs,
                    }));
                }
                _ => continue,
            }
        }
    }

    let mut parser = Parser {
        type_names,
        extra_types: IndexVec::default(),
    };

    // Collect inner types like struct fields and enum variants
    let mut types = IndexVec::new();
    for prototype in prototypes {
        match prototype {
            PrototypeStructOrEnum::Struct(prototype_struct) => {
                types.push(parser.parse_struct(
                    prototype_struct.type_id,
                    prototype_struct.attrs,
                    prototype_struct.item,
                ));
            }
            PrototypeStructOrEnum::Enum(prototype_enum) => {
                types.push(parser.parse_enum(
                    prototype_enum.type_id,
                    prototype_enum.attrs,
                    prototype_enum.item,
                ));
            }
        }
    }

    types.extend(parser.extra_types);
    let schema = Schema { types, repr_sizes };
    validate_invalidations(&schema);
    validate_estree_shapes(&schema);
    crate::estree::validate_tags(&schema);
    for ty in &schema.types {
        if let AstType::Struct(ast) = ty {
            crate::estree::converters::ConversionPlan::new(ast, &schema);
        }
    }
    schema
}

impl Parser {
    fn create_new_type(&mut self, mut item: AstType) -> TypeId {
        let type_id = TypeId::from_usize(self.type_names.len());
        match &mut item {
            AstType::Struct(ast) => ast.type_id = type_id,
            AstType::Enum(ast) => ast.type_id = type_id,
            AstType::SubRange(ast) => ast.type_id = type_id,
            AstType::Option(ast) => ast.type_id = type_id,
            AstType::Primitive(ast) => ast.type_id = type_id,
        }

        let not_contained = self.type_names.insert(item.name().to_string());
        assert!(not_contained, "{} is duplicated", item.name());

        self.extra_types.push(item);
        type_id
    }

    /// Get type id of the simple type which is not wrapped by `Option` or `SubRange`
    fn simple_type_id(&mut self, name: &str) -> TypeId {
        if let Some(type_id) = self.type_names.get_index_of(name) {
            return TypeId::from_usize(type_id);
        }

        let primitive = |name| {
            AstType::Primitive(AstPrimitive {
                type_id: TypeId::DUMMY,
                name,
            })
        };

        let type_def = match name {
            "bool" => primitive("bool"),
            "u8" => primitive("u8"),
            "u16" => primitive("u16"),
            "u32" => primitive("u32"),
            "u64" => primitive("u64"),
            "u128" => primitive("u128"),
            "usize" => primitive("usize"),
            "i8" => primitive("i8"),
            "i16" => primitive("i16"),
            "i32" => primitive("i32"),
            "i64" => primitive("i64"),
            "i128" => primitive("i128"),
            "isize" => primitive("isize"),
            "f32" => primitive("f32"),
            "f64" => primitive("f64"),

            // Custom enum
            "Span" => primitive("Span"),
            "ScopeId" => primitive("ScopeId"),
            "SymbolId" => primitive("SymbolId"),
            "AstUtf8" => primitive("AstUtf8"),
            "AstWtf8" => primitive("AstWtf8"),
            "ImportPhase" => primitive("ImportPhase"),
            "SourceType" => primitive("SourceType"),
            "MethodDefinitionKind" => primitive("MethodDefinitionKind"),
            "Accessibility" => primitive("Accessibility"),
            "ClassType" => primitive("ClassType"),
            "BinaryOperator" => primitive("BinaryOperator"),
            "LogicalOperator" => primitive("LogicalOperator"),
            "UnaryOperator" => primitive("UnaryOperator"),
            "UpdateOperator" => primitive("UpdateOperator"),
            "AssignmentOperator" => primitive("AssignmentOperator"),
            "VariableKind" => primitive("VariableKind"),
            "FunctionType" => primitive("FunctionType"),
            "FormalParameterKind" => primitive("FormalParameterKind"),
            "PropertyKind" => primitive("PropertyKind"),
            "NumericLiteralKind" => primitive("NumericLiteralKind"),
            "ImportOrExportKind" => primitive("ImportOrExportKind"),
            "TsTypeOperatorKind" => primitive("TsTypeOperatorKind"),
            "TsMethodSignatureKind" => primitive("TsMethodSignatureKind"),
            "TsModuleDeclarationKind" => primitive("TsModuleDeclarationKind"),
            "TsMappedTypeModifier" => primitive("TsMappedTypeModifier"),
            _ => panic!("Unknown primitive {name}"),
        };
        self.create_new_type(type_def)
    }
}

/// Parse #[repr(uN)] attribute and return the size in bytes
/// Handles whitespace and multiple repr arguments (e.g., `#[repr(C, u8)]`)
fn parse_repr_size(attrs: &[Attribute]) -> Option<usize> {
    for attr in attrs {
        if let Meta::List(meta_list) = &attr.meta
            && meta_list.path.is_ident("repr")
        {
            // Parse as comma-separated identifiers to handle `#[repr(C, u8)]`
            if let Ok(args) =
                meta_list.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
            {
                for arg in args {
                    match arg.to_string().as_str() {
                        "u8" => return Some(1),
                        "u16" => return Some(2),
                        "u32" => return Some(4),
                        "u64" => return Some(8),
                        _ => continue,
                    }
                }
            }
        }
    }
    None
}

fn parse_attrs(attrs: &[Attribute]) -> Option<AstAttrs> {
    let mut has_ast_attr = false;
    let mut ast_attrs = HashSet::new();
    let mut generate_derives = HashSet::new();
    let mut estree = EstreeAttrs::default();

    // Collect AST markers and node-level ESTree metadata.
    for attr in attrs {
        match &attr.meta {
            Meta::Path(path) => {
                if path.is_ident("ast") {
                    has_ast_attr = true;
                }
            }
            Meta::List(meta_list) => {
                if meta_list.path.is_ident("ast") {
                    has_ast_attr = true;
                    let args = meta_list
                        .parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
                        .expect("Unable to parse `#[ast]`");
                    ast_attrs.extend(args.into_iter().map(|arg| arg.to_string()));
                }
                if meta_list.path.is_ident("generate_derive") {
                    let args = meta_list
                        .parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
                        .expect("Unable to parse `#[generate_derive]`");
                    generate_derives.extend(args.into_iter().map(|arg| arg.to_string()));
                }
                if meta_list.path.is_ident("estree") {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("add_field") {
                            estree.added_fields.push(parse_added_field(meta)?);
                            return Ok(());
                        }
                        if meta.path.is_ident("type_from") {
                            if estree.type_from.is_some() { return Err(meta.error("Duplicate type_from")); }
                            estree.type_from = Some(meta.value()?.parse::<Ident>()?.to_string());
                            return Ok(());
                        }
                        if meta.path.is_ident("type_via") {
                            if estree.type_via.is_some() { return Err(meta.error("Duplicate type_via")); }
                            estree.type_via = Some(parse_tag_selector(meta.value()?.parse()?)?);
                            return Ok(());
                        }
                        if meta.path.is_ident("transparent") {
                            if estree.transparent.is_some() {
                                return Err(meta.error("Duplicate transparent field"));
                            }
                            estree.transparent = Some(meta.value()?.parse::<Ident>()?.to_string());
                            return Ok(());
                        }
                        let target = if meta.path.is_ident("rename") {
                            &mut estree.rename
                        } else if meta.path.is_ident("ty") {
                            &mut estree.ty
                        } else {
                            return Err(meta
                                .error("Expected `rename`, `ty`, `type_from`, `type_via`, `add_field`, or `transparent`"));
                        };
                        if target.is_some() {
                            return Err(meta.error("Duplicate ESTree attribute"));
                        }
                        let value: syn::LitStr = meta.value()?.parse()?;
                        *target = Some(value.value());
                        Ok(())
                    })
                    .expect("Unable to parse node `#[estree]`");
                }
            }
            _ => continue,
        };
    }

    if !has_ast_attr {
        return None;
    }

    Some(AstAttrs {
        ast_attrs,
        generate_derives,
        estree,
    })
}

fn parse_tag_selector(expr: syn::Expr) -> syn::Result<EstreeTagSelector> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(tag),
            ..
        }) => Ok(EstreeTagSelector::Tag(tag.value())),
        syn::Expr::If(expr) => {
            let (_, alternate) = expr.else_branch.ok_or_else(|| {
                syn::Error::new_spanned(&expr.then_branch, "Tag selector requires else")
            })?;
            Ok(EstreeTagSelector::If {
                condition: parse_tag_condition(*expr.cond)?,
                consequent: Box::new(parse_tag_selector(tag_block(expr.then_branch)?)?),
                alternate: Box::new(parse_tag_selector(*alternate)?),
            })
        }
        syn::Expr::Block(expr) => parse_tag_selector(tag_block(expr.block)?),
        expr => Err(syn::Error::new_spanned(
            expr,
            "Expected a tag string or if expression",
        )),
    }
}

fn tag_block(block: syn::Block) -> syn::Result<syn::Expr> {
    let mut statements = block.stmts.into_iter();
    match (statements.next(), statements.next()) {
        (Some(syn::Stmt::Expr(expr, None)), None) => Ok(expr),
        _ => Err(syn::Error::new(
            block.brace_token.span.join(),
            "Tag block must contain one expression",
        )),
    }
}

fn parse_tag_condition(expr: syn::Expr) -> syn::Result<EstreeTagCondition> {
    match expr {
        syn::Expr::Path(expr) if expr.qself.is_none() && expr.path.get_ident().is_some() => {
            let name = expr.path.get_ident().unwrap().to_string();
            Ok(if name == "is_ts" {
                EstreeTagCondition::IsTs
            } else {
                EstreeTagCondition::Field(name)
            })
        }
        syn::Expr::Paren(expr) => parse_tag_condition(*expr.expr),
        syn::Expr::Unary(expr) if matches!(expr.op, syn::UnOp::Not(_)) => Ok(
            EstreeTagCondition::Not(Box::new(parse_tag_condition(*expr.expr)?)),
        ),
        syn::Expr::Binary(expr) if matches!(expr.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) => {
            let left = Box::new(parse_tag_condition(*expr.left)?);
            let right = Box::new(parse_tag_condition(*expr.right)?);
            Ok(if matches!(expr.op, syn::BinOp::And(_)) {
                EstreeTagCondition::And(left, right)
            } else {
                EstreeTagCondition::Or(left, right)
            })
        }
        expr => Err(syn::Error::new_spanned(
            expr,
            "Expected boolean fields, is_ts, !, &&, or ||",
        )),
    }
}

fn parse_added_field(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<EstreeAddedField> {
    let mut name = None;
    let mut value = None;
    let mut source = None;
    let mut element_type = None;
    let mut ts = false;
    meta.parse_nested_meta(|part| {
        if part.path.is_ident("ts") {
            if ts {
                return Err(part.error("Duplicate ts"));
            }
            ts = true;
        } else if part.path.is_ident("value") {
            if value.is_some() {
                return Err(part.error("Duplicate value"));
            }
            value = Some(part.value()?.parse::<syn::Expr>()?);
        } else if part.path.is_ident("from") || part.path.is_ident("via") {
            if source.is_some() {
                return Err(part.error("Added field requires exactly one source"));
            }
            let name = part.value()?.parse::<Ident>()?.to_string();
            source = Some(if part.path.is_ident("from") {
                EstreeFieldValue::From(name)
            } else {
                EstreeFieldValue::Via(name)
            });
        } else {
            let target = if part.path.is_ident("name") {
                &mut name
            } else if part.path.is_ident("element_type") {
                &mut element_type
            } else {
                return Err(part.error("Expected name, value, from, via, element_type, or ts"));
            };
            if target.is_some() {
                return Err(part.error("Duplicate added field attribute"));
            }
            *target = Some(part.value()?.parse::<syn::LitStr>()?.value());
        }
        Ok(())
    })?;
    let name = name.ok_or_else(|| meta.error("Added field requires a name"))?;
    if let Some(source) = source {
        if value.is_some() {
            return Err(meta.error("Added field requires exactly one of value, from, or via"));
        }
        if element_type.is_some() {
            return Err(meta.error("element_type is only valid for an empty array"));
        }
        // A computed/copied value derives its type from its source.
        return Ok(EstreeAddedField {
            name,
            value: source,
            ts,
        });
    }
    let value = value.ok_or_else(|| meta.error("Added field requires value, from, or via"))?;
    let value = match value {
        syn::Expr::Path(path) if path.path.is_ident("null") => EstreeValue::Null,
        syn::Expr::Lit(lit) => match lit.lit {
            syn::Lit::Bool(value) => EstreeValue::Bool(value.value),
            syn::Lit::Str(value) => EstreeValue::String(value.value()),
            _ => return Err(meta.error("Expected null, a boolean, a string, or []")),
        },
        syn::Expr::Array(array) if array.elems.is_empty() => EstreeValue::EmptyArray {
            element_type: element_type
                .take()
                .ok_or_else(|| meta.error("An empty array requires element_type"))?,
        },
        _ => return Err(meta.error("Expected null, a boolean, a string, or []")),
    };
    if element_type.is_some() {
        return Err(meta.error("element_type is only valid for an empty array"));
    }
    Ok(EstreeAddedField {
        name,
        value: EstreeFieldValue::Constant(value),
        ts,
    })
}

impl Parser {
    fn parse_struct(&mut self, type_id: TypeId, attrs: AstAttrs, item: ItemStruct) -> AstType {
        let name = item.ident.to_string();
        let fields: Vec<_> = item
            .fields
            .into_iter()
            .map(|field| self.parse_struct_field(field))
            .collect();
        let mut output_fields = HashSet::from(["type".into(), "start".into(), "end".into()]);
        for field in fields.iter().filter(|field| {
            !field.attrs.estree_skip
                && field.attrs.estree_concat.is_none()
                && attrs.estree.type_from.as_ref() != Some(&field.name)
        }) {
            output_fields.insert(crate::estree::estree_field_name(field));
        }
        for field in &attrs.estree.added_fields {
            assert!(
                output_fields.insert(field.name.clone()),
                "Duplicate ESTree field `{}.{}`",
                name,
                field.name
            );
        }
        AstType::Struct(AstStruct {
            type_id,
            name,
            fields,
            attrs,
        })
    }

    fn parse_struct_field(&mut self, field: Field) -> AstStructField {
        let name = field.ident.unwrap().to_string();
        let type_id = self
            .parse_type_name(&field.ty)
            .unwrap_or_else(|| panic!("Cannot parse type {}", field.ty.to_token_stream()));
        let attrs = parse_field_attrs(&field.attrs);

        AstStructField {
            type_id,
            name,
            attrs,
        }
    }

    fn parse_enum(&mut self, type_id: TypeId, attrs: AstAttrs, item: ItemEnum) -> AstType {
        let name = item.ident.to_string();
        assert!(
            attrs.estree.transparent.is_none()
                && attrs.estree.type_from.is_none()
                && attrs.estree.type_via.is_none(),
            "ESTree output selectors require a struct: {name}"
        );
        assert!(
            attrs.estree.added_fields.is_empty(),
            "ESTree added fields require a struct: {name}"
        );
        assert!(
            item.variants
                .iter()
                .all(|variant| variant.discriminant.is_none()),
            "`#[ast(repr)]` enum `{name}` does not support explicit discriminants"
        );
        let variants = item
            .variants
            .into_iter()
            .map(|variant| self.parse_enum_variant(variant))
            .collect();
        AstType::Enum(AstEnum {
            type_id,
            name,
            variants,
            attrs,
        })
    }

    fn parse_enum_variant(&mut self, variant: Variant) -> AstEnumVariant {
        let name = variant.ident.to_string();

        let type_id = if variant.fields.is_empty() {
            None
        } else {
            assert!(
                variant.fields.len() == 1,
                "Only variants with single field are supported"
            );
            let field = variant.fields.iter().next().unwrap();
            let type_id = self
                .parse_type_name(&field.ty)
                .unwrap_or_else(|| panic!("Cannot parse type {}", field.ty.to_token_stream()));
            Some(type_id)
        };

        let mut estree_ts_type = None;
        for attr in &variant.attrs {
            if !attr.path().is_ident("estree") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if !meta.path.is_ident("ts_type") {
                    return Err(meta.error("Expected variant ts_type"));
                }
                if estree_ts_type.is_some() {
                    return Err(meta.error("Duplicate ts_type"));
                }
                estree_ts_type = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                Ok(())
            })
            .expect("Unable to parse variant ESTree attributes");
        }
        AstEnumVariant {
            type_id,
            name,
            estree_ts_type,
        }
    }

    fn parse_type_name(&mut self, ty: &Type) -> Option<TypeId> {
        let Type::Path(ty) = ty else {
            return None;
        };

        if ty.qself.is_some() || ty.path.leading_colon.is_some() || ty.path.segments.len() != 1 {
            return None;
        }

        let segment = ty.path.segments.first().unwrap();
        let name = segment.ident.to_string();
        match &segment.arguments {
            PathArguments::None => Some(self.simple_type_id(&name)),
            PathArguments::Parenthesized(_) => None,
            PathArguments::AngleBracketed(angle) => {
                let mut args = angle.args.iter();
                let arg = match args.next().unwrap() {
                    GenericArgument::Lifetime(_) => args.next(),
                    arg => Some(arg),
                };

                if let Some(arg) = arg {
                    self.parse_wrapped_type(&name, arg)
                } else {
                    Some(self.simple_type_id(&name))
                }
            }
        }
    }

    fn parse_wrapped_type(&mut self, wrapper_name: &str, arg: &GenericArgument) -> Option<TypeId> {
        let GenericArgument::Type(ty_arg) = arg else {
            return None;
        };

        let inner_type_id = self.parse_type_name(ty_arg)?;
        let type_id = match wrapper_name {
            "SubRange" => {
                let name = format!(
                    "{}<{}>",
                    wrapper_name,
                    self.type_names[inner_type_id.index()]
                );
                if let Some(type_id) = self.type_names.get_index_of(&name) {
                    return Some(TypeId::from(type_id));
                }

                let ast_type = AstType::SubRange(AstSubRange {
                    type_id: TypeId::DUMMY,
                    name,
                    inner_type_id,
                });
                self.create_new_type(ast_type)
            }
            "Option" => {
                let name = format!(
                    "{}<{}>",
                    wrapper_name,
                    self.type_names[inner_type_id.index()]
                );
                if let Some(type_id) = self.type_names.get_index_of(&name) {
                    return Some(TypeId::from(type_id));
                }

                let ast_type = AstType::Option(AstOption {
                    type_id: TypeId::DUMMY,
                    name,
                    inner_type_id,
                });
                self.create_new_type(ast_type)
            }
            _ => return None,
        };
        Some(type_id)
    }
}

fn parse_field_attrs(attrs: &[Attribute]) -> AstFieldAttrs {
    let mut field_attrs = AstFieldAttrs::default();

    for attr in attrs {
        let Meta::List(meta_list) = &attr.meta else {
            continue;
        };
        if meta_list.path.is_ident("estree") {
            let args = meta_list
                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .expect("Unable to parse `#[estree]`");
            for arg in args {
                match arg {
                    Meta::Path(path) if path.is_ident("ts") => field_attrs.estree_ts = true,
                    Meta::NameValue(value) if value.path.is_ident("via") => {
                        assert!(
                            field_attrs.estree_via.is_none(),
                            "Duplicate ESTree converter"
                        );
                        field_attrs.estree_via = Some(
                            syn::parse2::<Ident>(value.value.to_token_stream())
                                .expect("ESTree converter must be an identifier")
                                .to_string(),
                        );
                    }
                    arg if arg.path().is_ident("skip") => field_attrs.estree_skip = true,
                    Meta::NameValue(value)
                        if value.path.is_ident("prepend_to")
                            || value.path.is_ident("append_to") =>
                    {
                        assert!(
                            field_attrs.estree_concat.is_none(),
                            "Duplicate ESTree list operation"
                        );
                        let target: Ident = syn::parse2(value.value.to_token_stream())
                            .expect("ESTree list target must be a field identifier");
                        field_attrs.estree_concat = Some(EstreeConcat {
                            target: target.to_string(),
                            prepend: value.path.is_ident("prepend_to"),
                        });
                    }
                    Meta::NameValue(value)
                        if value.path.is_ident("type_override")
                            || value.path.is_ident("ts_type") =>
                    {
                        let target = if value.path.is_ident("type_override") {
                            &mut field_attrs.estree_type_override
                        } else {
                            &mut field_attrs.estree_ts_type
                        };
                        assert!(target.is_none(), "Duplicate ESTree field type attribute");
                        *target = Some(
                            syn::parse2::<syn::LitStr>(value.value.to_token_stream())
                                .expect("ESTree type must be a string literal")
                                .value(),
                        );
                    }
                    Meta::NameValue(value) if value.path.is_ident("rename") => {
                        assert!(
                            field_attrs.estree_rename.is_none(),
                            "Duplicate ESTree field rename"
                        );
                        let rename: syn::LitStr = syn::parse2(value.value.to_token_stream())
                            .expect("ESTree field rename must be a string literal");
                        field_attrs.estree_rename = Some(rename.value());
                    }
                    _ => panic!(
                        "Expected ESTree field `skip`, `ts`, `rename`, `prepend_to`, `append_to`, `type_override`, `ts_type`, or `via`"
                    ),
                }
            }
            continue;
        }
        if meta_list.path.is_ident("ast") {
            let args = meta_list
                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .expect("Unable to parse AST field attributes");
            for arg in args {
                let Meta::List(list) = arg else {
                    panic!("AST field attributes must use list syntax");
                };
                if !list.path.is_ident("invalidates") {
                    panic!(
                        "Unknown AST field attribute `{}`",
                        list.path.to_token_stream()
                    );
                }
                let targets = list
                    .parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
                    .expect("Unable to parse `invalidates(...)`");
                field_attrs
                    .invalidates
                    .extend(targets.into_iter().map(|target| target.to_string()));
            }
        }
    }

    field_attrs
}

fn validate_estree_shapes(schema: &Schema) {
    for ty in &schema.types {
        let AstType::Struct(ast) = ty else { continue };
        for field in &ast.fields {
            let Some(concat) = &field.attrs.estree_concat else {
                continue;
            };
            let target = ast
                .fields
                .iter()
                .find(|field| field.name == concat.target)
                .unwrap_or_else(|| {
                    panic!(
                        "Unknown ESTree list target `{}.{}`",
                        ast.name, concat.target
                    )
                });
            assert!(
                !field.attrs.estree_skip && field.attrs.estree_rename.is_none(),
                "ESTree list source `{}.{}` cannot be skipped or renamed",
                ast.name,
                field.name
            );
            assert!(
                !target.attrs.estree_skip && target.attrs.estree_concat.is_none(),
                "ESTree list target `{}.{}` cannot be skipped or itself a source",
                ast.name,
                target.name
            );
            assert_eq!(
                field.attrs.estree_ts, target.attrs.estree_ts,
                "ESTree list source and target must have the same TS visibility: {}.{}",
                ast.name, field.name
            );
            assert!(
                matches!(schema.types[target.type_id], AstType::SubRange(_)),
                "ESTree list target `{}.{}` must be a SubRange",
                ast.name,
                target.name
            );
            let inner = match &schema.types[field.type_id] {
                AstType::SubRange(range) => range.inner_type_id,
                AstType::Option(option) => {
                    assert!(
                        matches!(
                            schema.types[option.inner_type_id],
                            AstType::Struct(_) | AstType::Enum(_)
                        ),
                        "ESTree optional list source must contain a node: {}.{}",
                        ast.name,
                        field.name
                    );
                    option.inner_type_id
                }
                _ => panic!(
                    "ESTree list source `{}.{}` must be a SubRange or optional node",
                    ast.name, field.name
                ),
            };
            assert!(
                matches!(
                    schema.types[inner],
                    AstType::Struct(_) | AstType::Enum(_) | AstType::Option(_)
                ),
                "ESTree list source must contain nodes: {}.{}",
                ast.name,
                field.name
            );
        }
        if let Some(name) = &ast.attrs.estree.transparent {
            let target = ast
                .fields
                .iter()
                .find(|field| &field.name == name)
                .unwrap_or_else(|| {
                    panic!("Unknown transparent ESTree field `{}.{name}`", ast.name)
                });
            assert!(
                ast.attrs.estree.ty.is_none()
                    && ast.attrs.estree.rename.is_none()
                    && ast.attrs.estree.added_fields.is_empty(),
                "Transparent ESTree output cannot have a name, tag, or added fields: {}",
                ast.name
            );
            assert!(
                !target.attrs.estree_skip
                    && !target.attrs.estree_ts
                    && target.attrs.estree_concat.is_none(),
                "Transparent ESTree field must always be emitted: {}.{name}",
                ast.name
            );
            assert!(
                matches!(
                    schema.types[target.type_id],
                    AstType::Struct(_) | AstType::Enum(_) | AstType::SubRange(_)
                ),
                "Transparent ESTree field must contain a node or list: {}.{name}",
                ast.name
            );
            assert!(
                ast.fields.iter().all(|field| field.name == *name
                    || field.attrs.estree_skip
                    || field.attrs.estree_concat.is_some()),
                "Other fields of transparent ESTree output must be skipped or combined: {}",
                ast.name
            );
        }
    }

    // Follow only output aliases, not ordinary node fields (which may recurse).
    fn visit(
        type_id: TypeId,
        schema: &Schema,
        active: &mut HashSet<TypeId>,
        done: &mut HashSet<TypeId>,
    ) {
        if done.contains(&type_id) {
            return;
        }
        assert!(
            active.insert(type_id),
            "Cyclic ESTree output alias: {}",
            schema.types[type_id].name()
        );
        match &schema.types[type_id] {
            AstType::Struct(ast) => {
                if let Some(field) = ast.estree_transparent_field() {
                    visit(field.type_id, schema, active, done);
                    for source in ast.estree_concat_sources(field) {
                        visit(source.type_id, schema, active, done);
                    }
                }
            }
            AstType::Enum(ast) => {
                for variant in &ast.variants {
                    if let Some(type_id) = variant.type_id {
                        visit(type_id, schema, active, done);
                    }
                }
            }
            AstType::SubRange(range) => visit(range.inner_type_id, schema, active, done),
            AstType::Option(option) => visit(option.inner_type_id, schema, active, done),
            AstType::Primitive(_) => {}
        }
        active.remove(&type_id);
        done.insert(type_id);
    }
    let mut done = HashSet::new();
    for (type_id, _) in schema.types.iter_enumerated() {
        visit(type_id, schema, &mut HashSet::new(), &mut done);
    }
}

fn validate_invalidations(schema: &Schema) {
    for ty in &schema.types {
        let AstType::Struct(ast) = ty else {
            continue;
        };
        let fields = ast
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.name.as_str(), index))
            .collect::<HashMap<_, _>>();

        for field in &ast.fields {
            let mut targets = HashSet::new();
            for target_name in &field.attrs.invalidates {
                assert!(
                    targets.insert(target_name),
                    "`{}.{}` invalidates `{target_name}` more than once",
                    ast.name,
                    field.name
                );
                let Some(&target_index) = fields.get(target_name.as_str()) else {
                    panic!(
                        "`{}.{}` invalidates unknown field `{target_name}`",
                        ast.name, field.name
                    );
                };
                assert_ne!(
                    field.name, *target_name,
                    "`{}.{}` cannot invalidate itself",
                    ast.name, field.name
                );
                let target = &ast.fields[target_index];
                assert!(
                    matches!(schema.types[target.type_id], AstType::Option(_)),
                    "`{}.{}` invalidates `{target_name}`, which must be an Option field",
                    ast.name,
                    field.name
                );
            }
        }

        let mut states = vec![0_u8; ast.fields.len()];
        for field_index in 0..ast.fields.len() {
            validate_invalidation_cycles(ast, &fields, field_index, &mut states);
        }
    }
}

fn validate_invalidation_cycles(
    ast: &AstStruct,
    fields: &HashMap<&str, usize>,
    field_index: usize,
    states: &mut [u8],
) {
    match states[field_index] {
        1 => panic!(
            "invalidation dependency cycle involving `{}.{}`",
            ast.name, ast.fields[field_index].name
        ),
        2 => return,
        _ => {}
    }

    states[field_index] = 1;
    for target_name in &ast.fields[field_index].attrs.invalidates {
        let target_index = fields[target_name.as_str()];
        validate_invalidation_cycles(ast, fields, target_index, states);
    }
    states[field_index] = 2;
}
