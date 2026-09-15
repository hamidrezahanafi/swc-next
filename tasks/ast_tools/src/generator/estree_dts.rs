//! Generate the public types of the decoder output, rather than the storage AST.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, Schema},
};

use crate::estree::types::{field_type, type_ref, union, value_type};
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
                    ty: value_type(ast, &field.value, schema),
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

fn set(fields: &mut Fields, name: &str, ty: &str) {
    fields.insert(
        name.into(),
        Field {
            ty: ty.into(),
            optional: false,
        },
    );
}

fn optional(fields: &mut Fields, names: &[&str]) {
    for name in names {
        fields.get_mut(*name).unwrap().optional = true;
    }
}

/// Mirror the decoder's special nodes. Ordinary fields, unions, renames,
/// skipped fields, TS-only fields, and scalar enums are generated from schema.
fn reshape(name: &str, fields: &mut Fields) {
    if name == "TsModuleDeclaration" {
        set(fields, "body", "TSModuleBody");
        optional(fields, &["body"]);
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
