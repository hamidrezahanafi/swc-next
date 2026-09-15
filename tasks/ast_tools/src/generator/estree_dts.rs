//! Generate the public types of the decoder output, rather than the storage AST.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstStructField, AstType, Schema, TypeId},
};

use crate::estree::{ENUMS, default_name, estree_field_name, overridden_tags};

#[derive(Clone)]
struct Field {
    ty: String,
    optional: bool,
}

type Fields = BTreeMap<String, Field>;

struct Node {
    tag: String,
    fields: Fields,
}

pub fn estree_dts(schema: &Schema) -> RawOutput {
    let mut nodes = BTreeMap::<String, Node>::new();
    for ty in &schema.types {
        let AstType::Struct(ast) = ty else { continue };
        if ast.attrs.estree.transparent.is_some() {
            continue;
        }
        let mut fields = fields(ast, schema);
        reshape(&ast.name, &mut fields);
        for field in &ast.attrs.estree.added_fields {
            fields.insert(
                field.name.clone(),
                Field {
                    ty: field.value.ts_type(),
                    optional: field.ts,
                },
            );
        }
        for (name, tag) in node_shapes(ast, schema) {
            let fields = fields.clone();
            if let Some(existing) = nodes.get_mut(&name) {
                // Several storage nodes decode to the same public node. Fields
                // absent from one shape must stay optional in the merged type.
                for (key, field) in &mut existing.fields {
                    if !fields.contains_key(key) {
                        field.optional = true;
                    }
                }
                for (key, mut field) in fields {
                    if let Some(old) = existing.fields.get_mut(&key) {
                        if old.ty != field.ty {
                            old.ty = union(&old.ty, &field.ty);
                        }
                        old.optional |= field.optional;
                    } else {
                        field.optional = true;
                        existing.fields.insert(key, field);
                    }
                }
            } else {
                nodes.insert(name, Node { tag, fields });
            }
        }
    }

    let mut output = String::from(PREAMBLE);
    let tag_enums: BTreeSet<_> = schema
        .types
        .iter()
        .filter_map(|ty| {
            let AstType::Struct(ast) = ty else {
                return None;
            };
            let name = ast.attrs.estree.type_from.as_ref()?;
            let field = ast.fields.iter().find(|field| &field.name == name).unwrap();
            Some(schema.types[field.type_id].name())
        })
        .collect();
    for (name, values) in ENUMS {
        if tag_enums.contains(name) || *name == "FormalParameterKind" {
            continue;
        }
        write_alias(&mut output, &default_name(name), values.iter().copied());
    }
    for ty in &schema.types {
        let AstType::Enum(ast) = ty else { continue };
        let variants: BTreeSet<_> = ast
            .variants
            .iter()
            .map(|variant| {
                let ty = &schema.types[variant.type_id.expect("AST unions contain nodes")];
                variant
                    .estree_ts_type
                    .clone()
                    .unwrap_or_else(|| type_ref(ty, schema))
            })
            .collect();
        write_alias(
            &mut output,
            &ast.attrs.estree.name(&ast.name),
            variants.iter().map(String::as_str),
        );
    }
    for ty in &schema.types {
        if let AstType::Struct(ast) = ty
            && ast.attrs.estree.type_from.is_some()
        {
            let tags = ast.estree_tags(schema);
            write_alias(
                &mut output,
                &ast.attrs.estree.name(&ast.name),
                tags.iter().map(String::as_str),
            );
        }
    }
    output.push_str("export type Literal = StringLiteral | NumericLiteral | BigIntLiteral | BooleanLiteral | NullLiteral | RegExpLiteral;\n\n");
    for (name, node) in &nodes {
        writeln!(
            output,
            "export interface {name} extends BaseNode {{\n  type: \"{}\";",
            node.tag
        )
        .unwrap();
        for (name, field) in &node.fields {
            writeln!(
                output,
                "  {name}{}: {};",
                if field.optional { "?" } else { "" },
                field.ty
            )
            .unwrap();
        }
        output.push_str("}\n\n");
    }
    let mut tags = BTreeMap::<&str, Vec<&str>>::new();
    for (name, node) in &nodes {
        tags.entry(&node.tag).or_default().push(name);
    }
    output.push_str("/** All decoded node kinds, indexed by their ESTree discriminant. */\nexport interface NodeMap {\n");
    for (tag, names) in tags {
        writeln!(output, "  {tag}: {};", names.join(" | ")).unwrap();
    }
    output.push_str("}\n\nexport type Node = NodeMap[keyof NodeMap];\n");
    RawOutput {
        path: "packages/ast/estree.d.ts".into(),
        content: output.into_bytes(),
    }
}

fn write_alias<'a>(output: &mut String, name: &str, members: impl Iterator<Item = &'a str>) {
    let members: Vec<_> = members.collect();
    let ty = members.join(" | ");
    if name.len() + ty.len() < 90 {
        writeln!(output, "export type {name} = {ty};\n").unwrap();
    } else {
        writeln!(
            output,
            "export type {name} =\n  | {};\n",
            members.join("\n  | ")
        )
        .unwrap();
    }
}

// Split only top-level unions: nested arrays and object types retain their
// own unions when multiple decoder shapes contribute the same field.
fn union(left: &str, right: &str) -> String {
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

fn node_shapes(ast: &AstStruct, schema: &Schema) -> BTreeMap<String, String> {
    let mut shapes = if ast.attrs.estree.type_from.is_some() || ast.attrs.estree.type_via.is_some()
    {
        ast.estree_tags(schema)
            .into_iter()
            .map(|tag| (tag.clone(), tag))
            .collect()
    } else {
        BTreeMap::from([(
            ast.attrs.estree.name(&ast.name),
            ast.attrs.estree.tag(&ast.name),
        )])
    };
    shapes.extend(
        overridden_tags(ast.type_id, schema)
            .into_iter()
            .map(|tag| (tag.clone(), tag)),
    );
    shapes
}

fn type_ref(ty: &AstType, schema: &Schema) -> String {
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

fn fields(ast: &AstStruct, schema: &Schema) -> Fields {
    ast.fields
        .iter()
        .filter(|field| ast.estree_emits_field(field))
        .map(|field| {
            let name = estree_field_name(field);
            let optional = field.attrs.estree_ts;
            (
                name,
                Field {
                    ty: field_type(ast, field, schema),
                    optional,
                },
            )
        })
        .collect()
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

fn field_type(ast: &AstStruct, field: &AstStructField, schema: &Schema) -> String {
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

fn set(fields: &mut Fields, name: &str, ty: &str) {
    fields.insert(
        name.into(),
        Field {
            ty: ty.into(),
            optional: false,
        },
    );
}

fn remove(fields: &mut Fields, names: &[&str]) {
    for name in names {
        fields.remove(*name);
    }
}

fn optional(fields: &mut Fields, names: &[&str]) {
    for name in names {
        fields.get_mut(*name).unwrap().optional = true;
    }
}

/// Mirror the decoder's special nodes. Ordinary fields, unions, renames,
/// skipped fields, TS-only fields, and scalar enums are generated from schema.
fn reshape(name: &str, fields: &mut Fields) {
    match name {
        "Program" => set(fields, "sourceType", "SourceType"),
        "NumericLiteral" => set(fields, "raw", "string"),
        "BigIntLiteral" => {
            set(fields, "value", "bigint");
            set(fields, "bigint", "string");
        }
        "BooleanLiteral" => set(fields, "raw", "\"true\" | \"false\""),
        "RegExpLiteral" => {
            fields.clear();
            set(fields, "value", "RegExp | null");
            set(fields, "raw", "string");
            set(fields, "regex", "{ pattern: string; flags: string }");
        }
        "TemplateElement" => {
            remove(fields, &["raw", "cooked", "isCookedUndefined"]);
            set(fields, "value", "{ raw: string; cooked: string | null }");
        }
        "JsxText" => set(fields, "raw", "string"),
        "TsMappedType" => set(fields, "optional", "boolean | \"+\" | \"-\""),
        "TsModuleDeclaration" => {
            set(fields, "body", "TSModuleBody");
            optional(fields, &["body"]);
        }
        _ => {}
    }
}

const PREAMBLE: &str = r#"// Generated by tasks/ast_tools. Do not edit.
// ESTree: https://github.com/estree/estree
// Includes the TypeScript and JSX extensions emitted by SWC Next.

/** Source offsets are UTF-16 code units. TS-only fields are absent in JS mode. */
export interface BaseNode {
  start: number;
  end: number;
}

"#;
