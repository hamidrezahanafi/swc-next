//! Generate the public types of the decoder output, rather than the storage AST.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, Schema},
};

use crate::estree::{ENUMS, estree_field_name, estree_type, is_ts_field, ts_extra_fields};

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
        // These storage wrappers disappear when decoding a parameter list.
        if matches!(ast.name.as_str(), "FormalParameter" | "FormalParameters") {
            continue;
        }
        let mut fields = fields(ast, schema);
        reshape(&ast.name, &mut fields);
        for name in node_names(&ast.name) {
            let mut fields = fields.clone();
            if name == "TSEmptyBodyFunctionExpression" {
                set(&mut fields, "body", "null");
            }
            let tag = if matches!(
                ast.name.as_str(),
                "StringLiteral"
                    | "NumericLiteral"
                    | "BigIntLiteral"
                    | "BooleanLiteral"
                    | "NullLiteral"
                    | "RegExpLiteral"
            ) {
                "Literal".into()
            } else {
                name.clone()
            };
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
    for (name, values) in ENUMS {
        if matches!(*name, "ClassType" | "FunctionType" | "FormalParameterKind") {
            continue;
        }
        write_alias(&mut output, &public_name(name), values.iter().copied());
    }
    for ty in &schema.types {
        let AstType::Enum(ast) = ty else { continue };
        let variants: BTreeSet<_> = ast
            .variants
            .iter()
            .map(|variant| {
                let ty = &schema.types[variant.type_id.expect("AST unions contain nodes")];
                // The storage Function/Class structs represent both declarations and
                // expressions; ESTree exposes a separate discriminant for each.
                match (ast.name.as_str(), ty.name()) {
                    ("Decl" | "ExportDefaultDeclarationKind", "Function") => {
                        "FunctionDeclaration".into()
                    }
                    ("Decl" | "ExportDefaultDeclarationKind", "Class") => "ClassDeclaration".into(),
                    ("Expr", "Function") => "FunctionExpression".into(),
                    ("Expr", "Class") => "ClassExpression".into(),
                    _ => type_ref(ty, schema),
                }
            })
            .collect();
        write_alias(
            &mut output,
            &public_name(&ast.name),
            variants.iter().map(String::as_str),
        );
    }
    output.push_str("export type Function = FunctionDeclaration | FunctionExpression;\n\nexport type Class = ClassDeclaration | ClassExpression;\n\nexport type Literal = StringLiteral | NumericLiteral | BigIntLiteral | BooleanLiteral | NullLiteral | RegExpLiteral;\n\n");
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

fn public_name(name: &str) -> String {
    match name {
        "Stmt" => "Statement".into(),
        "Decl" => "Declaration".into(),
        "Expr" => "Expression".into(),
        "BindingPattern" => "Pattern".into(),
        "FunctionBody" => "BlockStatement".into(),
        "Directive" => "ExpressionStatement".into(),
        "BindingProperty" | "ObjectProperty" => "Property".into(),
        "TsFunction" => "TSDeclareFunction".into(),
        "TsGlobalDeclaration" => "TSModuleDeclaration".into(),
        "TsThisParameter" => "Identifier".into(),
        _ => estree_type(name),
    }
}

fn node_names(name: &str) -> Vec<String> {
    match name {
        "Function" => vec!["FunctionDeclaration".into(), "FunctionExpression".into()],
        "Class" => vec!["ClassDeclaration".into(), "ClassExpression".into()],
        "MethodDefinition" | "TsMethodDefinition" => vec![
            "MethodDefinition".into(),
            "TSAbstractMethodDefinition".into(),
        ],
        "PropertyDefinition" => vec![
            "PropertyDefinition".into(),
            "AccessorProperty".into(),
            "TSAbstractPropertyDefinition".into(),
            "TSAbstractAccessorProperty".into(),
        ],
        "TsFunction" => vec![
            "TSDeclareFunction".into(),
            "TSEmptyBodyFunctionExpression".into(),
        ],
        _ => vec![public_name(name)],
    }
}

fn type_ref(ty: &AstType, schema: &Schema) -> String {
    match ty {
        AstType::SubRange(range) => format!("Array<{}>", type_ref(&schema.types[range.inner_type_id], schema)),
        AstType::Option(option) => format!("{} | null", type_ref(&schema.types[option.inner_type_id], schema)),
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" | "AstWtf8" => "string".into(),
            "bool" => "boolean".into(),
            "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "f64" => "number".into(),
            "Span" => "{ start: number; end: number }".into(),
            "ScopeId" | "SymbolId" => "undefined".into(),
            name if ENUMS.iter().any(|(key, _)| *key == name) => public_name(name),
            name => panic!("unsupported ESTree primitive: {name}"),
        },
        AstType::Struct(ast) => match ast.name.as_str() {
            "FormalParameter" => "FormalParameterPattern".into(),
            "FormalParameters" => "Array<FormalParameterItem | RestElement>".into(),
            "MethodDefinition" | "TsMethodDefinition" => "MethodDefinition | TSAbstractMethodDefinition".into(),
            "PropertyDefinition" => "PropertyDefinition | AccessorProperty | TSAbstractPropertyDefinition | TSAbstractAccessorProperty".into(),
            _ => public_name(&ast.name),
        },
        AstType::Enum(ast) => public_name(&ast.name),
    }
}

fn fields(ast: &AstStruct, schema: &Schema) -> Fields {
    let mut fields: Fields = ast
        .fields
        .iter()
        .filter(|field| !field.attrs.estree_skip)
        .map(|field| {
            let name = estree_field_name(&ast.name, &field.name);
            let optional = is_ts_field(&ast.name, &name);
            (
                name,
                Field {
                    ty: type_ref(&schema.types[field.type_id], schema),
                    optional,
                },
            )
        })
        .collect();
    for (name, value) in ts_extra_fields(&ast.name) {
        fields.insert(
            (*name).into(),
            Field {
                ty: if *value == "[]" {
                    "Array<Decorator>"
                } else {
                    value
                }
                .into(),
                optional: !ast.name.starts_with("Ts"),
            },
        );
    }
    fields
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
        "Program" | "FunctionBody" | "TsModuleBlock" => {
            remove(fields, &["directives"]);
            if name == "Program" {
                set(fields, "sourceType", "SourceType");
            }
        }
        "Function" | "TsFunction" | "ArrowFunctionExpression" => {
            remove(fields, &["functionType"]);
            if name != "ArrowFunctionExpression" {
                set(fields, "expression", "false");
            }
            if name == "TsFunction" {
                set(fields, "body", "null");
            } else {
                optional(fields, &["typeParameters", "returnType"]);
                if name == "Function" {
                    set(fields, "declare", "false");
                    optional(fields, &["declare"]);
                } else {
                    set(fields, "id", "null");
                    set(fields, "generator", "false");
                }
            }
        }
        "Directive" => {
            remove(fields, &["value"]);
            set(fields, "directive", "string");
        }
        "NumericLiteral" => set(fields, "raw", "string"),
        "BigIntLiteral" => {
            set(fields, "value", "bigint");
            set(fields, "bigint", "string");
        }
        "BooleanLiteral" => set(fields, "raw", "\"true\" | \"false\""),
        "NullLiteral" => {
            set(fields, "value", "null");
            set(fields, "raw", "\"null\"");
        }
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
        "Class" => {
            remove(fields, &["classType"]);
            optional(
                fields,
                &[
                    "typeParameters",
                    "superTypeArguments",
                    "implements",
                    "abstract",
                    "declare",
                ],
            );
        }
        "MethodDefinition" | "TsMethodDefinition" => {
            remove(fields, &["abstract"]);
            set(
                fields,
                "value",
                if name == "MethodDefinition" {
                    "FunctionExpression"
                } else {
                    "TSEmptyBodyFunctionExpression"
                },
            );
            optional(fields, &["override", "optional", "accessibility"]);
        }
        "PropertyDefinition" => {
            remove(fields, &["abstract", "accessor"]);
            optional(
                fields,
                &[
                    "typeAnnotation",
                    "declare",
                    "override",
                    "optional",
                    "definite",
                    "readonly",
                    "accessibility",
                ],
            );
        }
        "UnaryExpression" => set(fields, "prefix", "true"),
        "BindingProperty" => {
            set(fields, "kind", "\"init\"");
            set(fields, "method", "false");
            set(fields, "optional", "false");
            optional(fields, &["optional"]);
        }
        "ArrayPattern" => remove(fields, &["rest"]),
        "ObjectPattern" => {
            remove(fields, &["rest"]);
            set(fields, "properties", "Array<Property | RestElement>");
        }
        "JsxText" => set(fields, "raw", "string"),
        "TsMethodSignature" => {
            set(fields, "accessibility", "null");
            set(fields, "readonly", "false");
            set(fields, "static", "false");
        }
        "TsMappedType" => set(fields, "optional", "boolean | \"+\" | \"-\""),
        "TsModuleDeclaration" => {
            set(fields, "body", "TSModuleBody");
            optional(fields, &["body"]);
            set(fields, "global", "false");
        }
        "TsGlobalDeclaration" => {
            set(fields, "kind", "\"global\"");
            set(fields, "global", "true");
        }
        "TsThisParameter" => {
            set(fields, "name", "\"this\"");
            set(fields, "decorators", "Array<Decorator>");
            set(fields, "optional", "false");
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
