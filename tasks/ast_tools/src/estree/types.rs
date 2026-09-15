//! Public output types shared by declarations and converter validation.

use super::{ENUMS, converters::converter, default_name, estree_field_name};
use crate::schema::{AstStruct, AstStructField, AstType, EstreeFieldValue, Schema, TypeId};
use std::collections::BTreeSet;

// Split only top-level unions: nested arrays and object types retain their
// own unions when multiple decoder shapes contribute the same field.
pub fn union(left: &str, right: &str) -> String {
    let mut members = BTreeSet::new();
    for ty in [left, right] {
        let mut depth = 0;
        let mut quoted = false;
        let mut start = 0;
        for (index, ch) in ty.char_indices() {
            match ch {
                '"' => quoted = !quoted,
                '<' | '{' | '(' | '[' if !quoted => depth += 1,
                '>' | '}' | ')' | ']' if !quoted => depth -= 1,
                '|' if !quoted && depth == 0 => {
                    members.insert(ty[start..index].trim());
                    start = index + 1;
                }
                _ => {}
            }
        }
        members.insert(ty[start..].trim());
    }
    if members.contains("boolean") || members.contains("true") && members.contains("false") {
        members.remove("true");
        members.remove("false");
        members.insert("boolean");
    }
    if members.contains("string") {
        members.retain(|member| !member.starts_with('"'));
    }
    members.into_iter().collect::<Vec<_>>().join(" | ")
}

pub fn type_ref(ty: &AstType, schema: &Schema) -> String {
    match ty {
        AstType::SubRange(range) => format!(
            "Array<{}>",
            type_ref(&schema.types[range.inner_type_id], schema)
        ),
        AstType::Option(option) => format!(
            "{} | null",
            type_ref(&schema.types[option.inner_type_id], schema)
        ),
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" | "AstWtf8" => "string".into(),
            "bool" => "boolean".into(),
            "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "f64" => "number".into(),
            "Span" => "{ start: number; end: number }".into(),
            "ScopeId" | "SymbolId" => "undefined".into(),
            name if ENUMS.iter().any(|(key, _)| *key == name) => default_name(name),
            name => panic!("unsupported ESTree primitive: {name}"),
        },
        AstType::Struct(ast) if ast.estree_transparent_field().is_some() => {
            field_type(ast, ast.estree_transparent_field().unwrap(), schema)
        }
        AstType::Struct(ast) if ast.attrs.estree.type_via.is_some() => {
            ast.estree_tags(schema).join(" | ")
        }
        AstType::Struct(ast) => ast.attrs.estree.name(&ast.name),
        AstType::Enum(ast) => ast.attrs.estree.name(&ast.name),
    }
}

/// Expand aliases only for containment checks; keep their names in the public type.
fn type_members(type_id: TypeId, schema: &Schema, members: &mut BTreeSet<String>) {
    match &schema.types[type_id] {
        AstType::Enum(ast) => {
            for variant in &ast.variants {
                type_members(
                    variant.type_id.expect("AST unions contain nodes"),
                    schema,
                    members,
                );
            }
        }
        AstType::Option(option) => {
            members.insert("null".into());
            type_members(option.inner_type_id, schema, members);
        }
        AstType::Struct(ast) if ast.estree_transparent_field().is_some() => {
            type_members(
                ast.estree_transparent_field().unwrap().type_id,
                schema,
                members,
            );
        }
        ty => {
            members.insert(type_ref(ty, schema));
        }
    }
}

pub fn field_type(ast: &AstStruct, field: &AstStructField, schema: &Schema) -> String {
    if let Some(name) = &field.attrs.estree_via {
        return converter(name).output_type.into();
    }
    if let Some(name) = field
        .attrs
        .estree_type_override
        .as_ref()
        .or(field.attrs.estree_ts_type.as_ref())
    {
        return name.clone();
    }
    let sources: Vec<_> = ast.estree_concat_sources(field).collect();
    if sources.is_empty() {
        return type_ref(&schema.types[field.type_id], schema);
    }
    let AstType::SubRange(range) = &schema.types[field.type_id] else {
        unreachable!("validated list target");
    };
    let mut element_type = type_ref(&schema.types[range.inner_type_id], schema);
    let mut members = BTreeSet::new();
    type_members(range.inner_type_id, schema, &mut members);
    for source in sources {
        let inner = match &schema.types[source.type_id] {
            AstType::SubRange(range) => range.inner_type_id,
            AstType::Option(option) => option.inner_type_id,
            _ => unreachable!("validated list source"),
        };
        let mut additional = BTreeSet::new();
        type_members(inner, schema, &mut additional);
        if !additional.is_subset(&members) {
            element_type = union(&element_type, &type_ref(&schema.types[inner], schema));
            members.extend(additional);
        }
    }
    format!("Array<{element_type}>")
}

pub fn value_type(ast: &AstStruct, value: &EstreeFieldValue, schema: &Schema) -> String {
    match value {
        EstreeFieldValue::Constant(value) => value.ts_type(),
        EstreeFieldValue::From(name) => {
            let field = ast
                .fields
                .iter()
                .find(|field| &field.name == name)
                .expect("validated storage source");
            type_ref(&schema.types[field.type_id], schema)
        }
        EstreeFieldValue::Via(name) => converter(name).output_type.into(),
    }
}

pub fn output_type(ast: &AstStruct, name: &str, schema: &Schema) -> String {
    if let Some(field) = ast
        .attrs
        .estree
        .overridden_fields
        .iter()
        .chain(&ast.attrs.estree.added_fields)
        .find(|field| field.name == name)
    {
        assert!(
            !field.ts,
            "Converters require unconditional output inputs: {}.{name}",
            ast.name
        );
        return value_type(ast, &field.value, schema);
    }
    if matches!(name, "start" | "end") {
        return "number".into();
    }
    let field = ast
        .fields
        .iter()
        .find(|field| ast.estree_emits_field(field) && estree_field_name(field) == name)
        .unwrap_or_else(|| panic!("Unknown ESTree output {}.{name}", ast.name));
    assert!(
        !field.attrs.estree_ts,
        "Converters require unconditional output inputs: {}.{name}",
        ast.name
    );
    field_type(ast, field, schema)
}
