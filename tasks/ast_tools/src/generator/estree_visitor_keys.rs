use std::{collections::BTreeMap, fmt::Write};

use super::estree::{estree_field_name, estree_type};
use crate::{
    output::RawOutput,
    schema::{AstType, Schema, TypeId},
};

// ESTree projections which flatten, merge, or synthesize internal AST nodes.
// Keep these aligned with write_special_node in estree_decoder.
pub fn estree_visitor_keys(schema: &Schema) -> Vec<RawOutput> {
    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut lists = BTreeMap::new();
    for ty in &schema.types {
        let AstType::Struct(ast) = ty else { continue };
        let name = ast.name.as_str();
        let (names, special): (Vec<String>, Option<&[&str]>) = match name {
            "FormalParameter" | "FormalParameters" | "TsIndexSignatureParameter" => continue,
            "Program" => (vec!["Program".into()], Some(&["body"])),
            "FunctionBody" => (vec!["BlockStatement".into()], Some(&["body"])),
            "TsModuleBlock" => (vec!["TSModuleBlock".into()], Some(&["body"])),
            "Function" => (
                vec!["FunctionDeclaration".into(), "FunctionExpression".into()],
                Some(&["id", "typeParameters", "params", "returnType", "body"]),
            ),
            "TsFunction" => (
                vec![
                    "TSDeclareFunction".into(),
                    "TSEmptyBodyFunctionExpression".into(),
                ],
                Some(&["id", "typeParameters", "params", "returnType", "body"]),
            ),
            "ArrowFunctionExpression" => (
                vec![name.into()],
                Some(&["typeParameters", "params", "returnType", "body"]),
            ),
            "Class" => (
                vec!["ClassDeclaration".into(), "ClassExpression".into()],
                Some(&[
                    "decorators",
                    "id",
                    "typeParameters",
                    "superClass",
                    "superTypeArguments",
                    "implements",
                    "body",
                ]),
            ),
            "MethodDefinition" | "TsMethodDefinition" => (
                vec![
                    "MethodDefinition".into(),
                    "TSAbstractMethodDefinition".into(),
                ],
                Some(&["decorators", "key", "value"]),
            ),
            "PropertyDefinition" => (
                vec![
                    "PropertyDefinition".into(),
                    "AccessorProperty".into(),
                    "TSAbstractPropertyDefinition".into(),
                    "TSAbstractAccessorProperty".into(),
                ],
                Some(&["decorators", "key", "typeAnnotation", "value"]),
            ),
            "Directive" => (vec!["ExpressionStatement".into()], Some(&["expression"])),
            "StringLiteral" | "NumericLiteral" | "BigIntLiteral" | "BooleanLiteral"
            | "NullLiteral" | "RegExpLiteral" => (vec!["Literal".into()], Some(&[])),
            "TemplateElement" => (vec![name.into()], Some(&[])),
            "FormalParameterRest" | "BindingRestElement" => (
                vec!["RestElement".into()],
                Some(&["decorators", "argument", "typeAnnotation"]),
            ),
            "BindingProperty" | "ObjectProperty" => {
                (vec!["Property".into()], Some(&["key", "value"]))
            }
            "ArrayPattern" => (
                vec![name.into()],
                Some(&["decorators", "elements", "typeAnnotation"]),
            ),
            "ObjectPattern" => (
                vec![name.into()],
                Some(&["decorators", "properties", "typeAnnotation"]),
            ),
            "AssignmentPattern" => (
                vec![name.into()],
                Some(&["decorators", "left", "typeAnnotation", "right"]),
            ),
            "VariableDeclarator" => (vec![name.into()], Some(&["id", "init"])),
            "CatchClause" => (vec![name.into()], Some(&["param", "body"])),
            "TsThisParameter" => (
                vec!["Identifier".into()],
                Some(&["decorators", "typeAnnotation"]),
            ),
            "TsGlobalDeclaration" => (vec!["TSModuleDeclaration".into()], Some(&["id", "body"])),
            "TsFunctionType"
            | "TsConstructorType"
            | "TsCallSignatureDeclaration"
            | "TsConstructSignatureDeclaration" => (
                vec![estree_type(name)],
                Some(&["typeParameters", "params", "returnType"]),
            ),
            "TsMethodSignature" => (
                vec![estree_type(name)],
                Some(&["key", "typeParameters", "params", "returnType"]),
            ),
            _ => (vec![estree_type(name)], None),
        };
        let fields: Vec<String> = special
            .map(|fields| fields.iter().map(|s| (*s).into()).collect())
            .unwrap_or_else(|| {
                ast.fields
                    .iter()
                    .filter(|f| !f.attrs.estree_skip && is_child(f.type_id, schema))
                    .map(|f| estree_field_name(name, &f.name))
                    .collect()
            });
        for name in names {
            for field in &fields {
                let list = if special.is_some() {
                    matches!(
                        field.as_str(),
                        "decorators" | "params" | "implements" | "elements" | "properties"
                    ) || (field == "body"
                        && matches!(
                            name.as_str(),
                            "Program" | "BlockStatement" | "TSModuleBlock"
                        ))
                } else {
                    ast.fields.iter().any(|f| {
                        estree_field_name(ast.name.as_str(), &f.name) == *field
                            && is_list(f.type_id, schema)
                    })
                };
                let previous = lists.insert((name.clone(), field.clone()), list);
                assert!(previous.is_none_or(|previous| previous == list));
            }
            let entry = keys.entry(name).or_default();
            for field in &fields {
                if !entry.contains(field) {
                    entry.push(field.clone());
                }
            }
        }
    }
    let mut js = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport const visitorKeys = Object.freeze({\n",
    );
    let mut dts = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport declare const visitorKeys: {\n",
    );
    let mut children = String::from(
        "// generated by tasks/ast_tools, do not edit\nexport const childVisitors = {\n",
    );
    for (name, fields) in &keys {
        writeln!(js, "  {name}: Object.freeze({fields:?}),").unwrap();
        writeln!(dts, "  readonly {name}: readonly string[];").unwrap();
        if fields.is_empty() {
            writeln!(children, "  {name}: null,").unwrap();
        } else {
            writeln!(children, "  {name}(node, walker) {{").unwrap();
            for (index, field) in fields.iter().enumerate() {
                writeln!(children, "    const child{index} = node.{field};").unwrap();
                let method = if lists[&(name.clone(), field.clone())] {
                    "visitList"
                } else {
                    "visitScalar"
                };
                writeln!(children, "    if (child{index} != null && walker.{method}(child{index}, node, {field:?})) return true;").unwrap();
            }
            writeln!(children, "    return false;\n  }},").unwrap();
        }
    }
    js.push_str("});\n");
    children.push_str("};\n");
    children.push_str("\nexport const nodeWalkers = Object.assign(Object.create(null), {\n");
    for (name, fields) in &keys {
        if fields.is_empty() {
            writeln!(children, "  {name}: null,").unwrap();
        } else {
            writeln!(children, "  {name}: walk{name},").unwrap();
        }
    }
    children.push_str("});\n");
    for (name, fields) in &keys {
        if fields.is_empty() {
            continue;
        }
        writeln!(
            children,
            "\nfunction walk{name}(node, walker, parent, key, list, index) {{"
        )
        .unwrap();
        for (index, field) in fields.iter().enumerate() {
            let method = if lists[&(name.clone(), field.clone())] {
                "visitList"
            } else {
                "visitScalar"
            };
            writeln!(children, "  const child{index} = node.{field};\n  if (child{index} != null && walker.{method}(child{index}, node, {field:?})) return index;").unwrap();
        }
        writeln!(children, "  if (node.type !== {name:?}) return walker.leaveChanged(node, parent, key, list, index);\n  return index;\n}}").unwrap();
    }
    // Partition large same-length groups by the most distinguishing character.
    // Full-name guards preserve rejection of unsupported names; each known type
    // retains its own property/call feedback site. No corpus frequencies enter
    // the generator's choice of discriminator.
    let mut by_length: BTreeMap<usize, Vec<&String>> = BTreeMap::new();
    for name in keys.keys() {
        by_length.entry(name.len()).or_default().push(name);
    }
    children.push_str("\nexport function dispatchNode(node, walker, parent, key, list, index, walkers) {\n  const type = node.type;\n  switch (typeof type === \"string\" ? type.length : -1) {\n");
    for (length, names) in by_length {
        writeln!(children, "    case {length}:").unwrap();
        write_dispatch_group(&mut children, &names, "      ");
        children.push_str("      break;\n");
    }
    children.push_str("  }\n  throw new TypeError(`Unsupported AST node type: ${type}`);\n}\n");
    dts.push_str("};\nexport type NodeType = keyof typeof visitorKeys;\n");
    vec![
        RawOutput {
            path: "packages/walker/child-visitors.js".into(),
            content: children.into_bytes(),
        },
        RawOutput {
            path: "packages/walker/visitor-keys.js".into(),
            content: js.into_bytes(),
        },
        RawOutput {
            path: "packages/walker/visitor-keys.d.ts".into(),
            content: dts.into_bytes(),
        },
    ]
}

fn write_dispatch_group(out: &mut String, names: &[&String], indent: &str) {
    if names.len() <= 16 {
        writeln!(out, "{indent}switch (type) {{").unwrap();
        for name in names {
            writeln!(out, "{indent}  case {name:?}: {{\n{indent}    const visit = walkers.{name};\n{indent}    return visit === null ? index : visit(node, walker, parent, key, list, index);\n{indent}  }}").unwrap();
        }
        writeln!(out, "{indent}}}").unwrap();
        return;
    }
    let position = (0..names[0].len())
        .min_by_key(|&position| {
            let mut counts = BTreeMap::<u8, usize>::new();
            for name in names {
                *counts.entry(name.as_bytes()[position]).or_default() += 1;
            }
            counts.values().map(|count| count * count).sum::<usize>()
        })
        .unwrap();
    let mut groups = BTreeMap::<u8, Vec<&String>>::new();
    for name in names {
        groups
            .entry(name.as_bytes()[position])
            .or_default()
            .push(name);
    }
    writeln!(out, "{indent}switch (type.charCodeAt({position})) {{").unwrap();
    for (code, group) in groups {
        writeln!(out, "{indent}  case {code}:").unwrap();
        write_dispatch_group(out, &group, &format!("{indent}    "));
        writeln!(out, "{indent}    break;").unwrap();
    }
    writeln!(out, "{indent}}}").unwrap();
}

fn is_child(id: TypeId, schema: &Schema) -> bool {
    match &schema.types[id] {
        AstType::Struct(_) | AstType::Enum(_) => true,
        AstType::Option(opt) => is_child(opt.inner_type_id, schema),
        AstType::SubRange(range) => is_child(range.inner_type_id, schema),
        AstType::Primitive(_) => false,
    }
}

fn is_list(id: TypeId, schema: &Schema) -> bool {
    match &schema.types[id] {
        AstType::SubRange(_) => true,
        AstType::Option(opt) => is_list(opt.inner_type_id, schema),
        _ => false,
    }
}
