use std::fmt::Write;

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, Schema},
};

use super::{
    estree_decoder::{self as eager, DecodedField},
    js_minifier::minify_with_class_names,
};

pub fn estree_lazy_decoder(schema: &Schema) -> Vec<RawOutput> {
    let mut outputs = Vec::new();
    for (is_ts, suffix, name) in [(false, "js", "decodeJs"), (true, "ts", "decodeTs")] {
        let pretty_path = format!("packages/decoder/lazy/decode-{suffix}.pretty.js");
        let source = decoder_source(schema, is_ts, name);
        let minified = minify_with_class_names(&source, &pretty_path);
        outputs.push(RawOutput {
            path: pretty_path,
            content: source.into_bytes(),
        });
        outputs.push(RawOutput {
            path: format!("packages/decoder/lazy/decode-{suffix}.js"),
            content: minified.into_bytes(),
        });
    }
    outputs
}

fn decoder_source(schema: &Schema, is_ts: bool, name: &str) -> String {
    let mut output = String::from(eager::PREAMBLE);
    if is_ts {
        output.push_str(eager::TS_ENUMS);
    }
    output.push_str(eager::PREAMBLE_END);
    output.push_str(eager::DECODE_HELPERS);
    output.push_str("import { constructorError, TOKEN } from './lazy-common.js';\nimport { NodeArray } from './node-array.js';\nconst inspectSymbol = Symbol.for(\"nodejs.util.inspect.custom\");\n\n");
    write_assignment_parameter(&mut output, is_ts);
    let types: Vec<_> = schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Struct(ast) => Some(ast),
            _ => None,
        })
        .enumerate()
        .filter(|(_, ast)| is_ts || !ast.name.starts_with("Ts"))
        .collect();
    for (_, ast) in &types {
        write_class(&mut output, ast, schema, is_ts);
    }
    writeln!(
        output,
        "export function {name}(buffer, source, sourceBuffer) {{"
    )
    .unwrap();
    output.push_str(eager::DECODE_SETUP);
    output.push_str(LAZY_ARRAYS);
    output.push_str(eager::DECODE_DIAGNOSTICS);
    output.push_str(
        r#"  const ast = {
    token: TOKEN, nodes: new Map(), source, sourceType, spans, payloads, extra,
    _i32, _dv, str, enumValue, optionalEnum, node, optionalNode, nodeArray, nodeArrayPair, appendNode, prependNode,
  };
  function node(index, typeOverride) {
    const key = typeOverride === undefined ? index : `${index}:${typeOverride}`;
    if (ast.nodes.has(key)) return ast.nodes.get(key);
    let result;
    switch (bytes[kinds + index]) {
"#,
    );
    for (kind, ast) in types {
        let project = matches!(
            ast.name.as_str(),
            "FormalParameter" | "TsIndexSignatureParameter"
        );
        writeln!(
            output,
            "      case {kind}: result = new {}(index, ast{}){}; break;",
            ast.name,
            if ast.name == "TsFunction" {
                ", typeOverride"
            } else {
                ""
            },
            if project { ".project" } else { "" }
        )
        .unwrap();
    }
    output.push_str("      default: throw new RangeError(`Unknown AST node kind ${bytes[kinds + index]}`);\n    }\n    ast.nodes.set(key, result);\n    return result;\n  }\n");
    output.push_str(eager::DECODE_RESULT);
    output
}

fn write_class(output: &mut String, ast: &AstStruct, schema: &Schema, is_ts: bool) {
    let raw_fields = eager::decoded_fields(ast, schema);
    let mut fields = vec![
        (
            "type".into(),
            format!("\"{}\"", eager::estree_type(&ast.name)),
        ),
        ("start".into(), "start".into()),
        ("end".into(), "end".into()),
    ];
    fields.extend(
        raw_fields
            .iter()
            .filter(|f| is_ts || !eager::is_ts_field(&ast.name, &f.name))
            .map(|f| (f.name.clone(), format!("f_{}", f.name))),
    );
    if is_ts {
        fields.extend(
            eager::ts_extra_fields(&ast.name)
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into())),
        );
    }
    special_fields(&mut fields, ast, &raw_fields, is_ts);
    for (_, expression) in &mut fields {
        if let Some(field) = raw_fields
            .iter()
            .find(|f| *expression == format!("f_{}", f.name))
            && !(ast.name == "TsMethodDefinition" && field.name == "value")
        {
            *expression = field.expression.clone();
        }
    }
    let dependencies: Vec<_> = raw_fields
        .iter()
        .filter(|f| {
            fields
                .iter()
                .any(|(_, expr)| eager::references_field(expr, &f.name))
        })
        .collect();
    writeln!(output, "export class {} {{", ast.name).unwrap();
    let fixed_type = fields
        .iter()
        .find(|(name, _)| name == "type")
        .and_then(|(_, expr)| expr.starts_with('"').then_some(expr));
    if let Some(ty) = fixed_type {
        writeln!(output, "  type = {ty};").unwrap();
    }
    let overlay = supports_overlay(&ast.name);
    let type_override = ast.name == "TsFunction";
    writeln!(output, "  #internal;\n\n  constructor(index, ast{}) {{\n    if (ast?.token !== TOKEN) constructorError();", if overlay { ", overrides" } else if type_override { ", typeOverride" } else { "" }).unwrap();
    output.push_str(if type_override {
        "    const key = typeOverride === undefined ? index : `${index}:${typeOverride}`;\n"
    } else {
        "    const key = index;\n"
    });
    if overlay {
        output.push_str("    if (overrides === undefined) {\n      const cached = ast.nodes.get(key);\n      if (cached !== undefined) return cached;\n    }\n");
    } else {
        output.push_str("    const cached = ast.nodes.get(key);\n    if (cached !== undefined) return cached;\n");
    }
    output.push_str("    this.#internal = { index, ast");
    if overlay {
        output.push_str(", overrides");
    }
    if type_override {
        output.push_str(", typeOverride");
    }
    for (name, expression) in &fields {
        if should_cache(expression) {
            write!(output, ", ${name}: void 0").unwrap();
        }
    }
    for field in &dependencies {
        if should_cache(&field.expression) {
            write!(output, ", $f_{}: void 0", field.name).unwrap();
        }
    }
    output.push_str(" };\n");
    output.push_str(if overlay {
        "    if (overrides === undefined) ast.nodes.set(key, this);\n"
    } else {
        "    ast.nodes.set(key, this);\n"
    });
    output.push_str("  }\n\n");
    if overlay {
        writeln!(output, "  withFields(overrides) {{\n    const internal = this.#internal;\n    return new {}(internal.index, internal.ast, {{ ...internal.overrides, ...overrides }});\n  }}\n", ast.name).unwrap();
    }
    for field in dependencies {
        let expression = if ast.name == "TsMethodDefinition" && field.name == "value" {
            format!(
                "node({}, \"TSEmptyBodyFunctionExpression\")",
                field
                    .expression
                    .strip_prefix("node(")
                    .unwrap()
                    .strip_suffix(')')
                    .unwrap()
            )
        } else {
            field.expression.clone()
        };
        write_getter(
            output,
            ast,
            &format!("#f_{}", field.name),
            &format!("f_{}", field.name),
            &expression,
        );
    }
    for (name, expression) in &fields {
        if name == "type" && fixed_type.is_some() {
            continue;
        }
        let mut expression = expression.clone();
        for field in &raw_fields {
            let identifier = format!("f_{}", field.name);
            let replacement = format!("this.#f_{}", field.name);
            let positions: Vec<_> = expression
                .match_indices(&identifier)
                .filter(|(start, _)| {
                    expression
                        .as_bytes()
                        .get(start + identifier.len())
                        .is_none_or(|next| {
                            !next.is_ascii_alphanumeric() && *next != b'_' && *next != b'$'
                        })
                })
                .map(|(start, _)| start)
                .collect();
            for pos in positions.into_iter().rev() {
                expression.replace_range(pos..pos + identifier.len(), &replacement);
            }
        }
        write_getter(output, ast, name, name, &expression);
    }
    output.push_str("  toJSON() {\n");
    if fields.iter().any(|(name, _)| name == "project") {
        output.push_str("    return this.project.toJSON();\n");
    } else {
        output.push_str("    return {\n");
        for (name, expression) in &fields {
            if ast.name == "TsModuleDeclaration" && name == "body" {
                output.push_str("      ...(this.body === undefined ? {} : { body: this.body }),\n");
            } else {
                let value = if name == "type" && fixed_type.is_some() {
                    expression.clone()
                } else {
                    format!("this.{name}")
                };
                writeln!(output, "      {name}: {value},").unwrap();
            }
        }
        output.push_str("    };\n");
    }
    writeln!(output, "  }}\n\n  [inspectSymbol]() {{\n    return Object.setPrototypeOf(this.toJSON(), Debug{}.prototype);\n  }}\n}}\n\nconst Debug{} = class {} {{}};\n", ast.name, ast.name, ast.name).unwrap();
}

fn write_getter(output: &mut String, ast: &AstStruct, name: &str, key: &str, expression: &str) {
    let cache = should_cache(expression);
    writeln!(
        output,
        "  get {name}() {{\n    const internal = this.#internal;"
    )
    .unwrap();
    if cache {
        writeln!(
            output,
            "    const cached = internal.${key};\n    if (cached !== undefined) return cached;"
        )
        .unwrap();
    }
    if supports_overlay(&ast.name) && !name.starts_with('#') {
        writeln!(output, "    if (internal.overrides?.{name} !== undefined) return {}internal.overrides.{name}();", if cache { format!("internal.${key} = ") } else { String::new() }).unwrap();
    }
    let context = [
        "source",
        "sourceType",
        "spans",
        "payloads",
        "extra",
        "_i32",
        "_dv",
        "str",
        "enumValue",
        "optionalEnum",
        "node",
        "optionalNode",
        "nodeArray",
        "nodeArrayPair",
        "appendNode",
        "prependNode",
    ];
    let identifiers: Vec<_> = expression
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '$')
        .collect();
    let uses_span = identifiers.contains(&"start") || identifiers.contains(&"end");
    let uses_payload = identifiers.contains(&"payload") || identifiers.contains(&"extraId");
    let names: Vec<_> = context
        .into_iter()
        .filter(|name| {
            identifiers.contains(name)
                || (uses_span && matches!(*name, "spans" | "_i32"))
                || (uses_payload && matches!(*name, "payloads" | "_i32"))
        })
        .collect();
    output.push_str(if expression.contains("typeOverride") {
        "    const { ast, index, typeOverride } = internal;\n"
    } else {
        "    const { ast, index } = internal;\n"
    });
    if !names.is_empty() {
        writeln!(output, "    const {{ {} }} = ast;", names.join(", ")).unwrap();
    }
    if uses_span {
        output.push_str("    const span = spans + index * 8;\n    const start = _i32[span >> 2], end = _i32[(span >> 2) + 1];\n");
    }
    if uses_payload {
        output.push_str("    const payload = payloads + index * 16;\n");
    }
    if expression.contains("extraId") {
        writeln!(
            output,
            "    const extraId = _i32[(payload + {}) >> 2];",
            ast.layout.extra_start_offset().unwrap()
        )
        .unwrap();
    }
    writeln!(
        output,
        "    return {}{expression};\n  }}\n",
        if cache {
            format!("internal.${key} = ")
        } else {
            String::new()
        }
    )
    .unwrap();
}

fn supports_overlay(name: &str) -> bool {
    matches!(
        name,
        "BindingIdentifier"
            | "IdentifierReference"
            | "ArrayPattern"
            | "ObjectPattern"
            | "AssignmentPattern"
            | "BindingRestElement"
    )
}

fn should_cache(expression: &str) -> bool {
    !(matches!(
        expression,
        "start" | "end" | "sourceType" | "false" | "true" | "null" | "undefined"
    ) || (expression.starts_with('"') && expression.ends_with('"'))
        || expression.starts_with("enumValue(")
        || expression.starts_with("optionalEnum(")
        || expression.starts_with("_dv.get")
        || (expression.starts_with("_i32[") && !expression.contains('?')))
}

fn write_assignment_parameter(output: &mut String, ts: bool) {
    output.push_str("class AssignmentParameter {\n  type = \"AssignmentPattern\";\n  #internal;\n  constructor(fields, ast) {\n    if (ast?.token !== TOKEN) constructorError();\n    this.#internal = { fields, $left: void 0, $right: void 0, $decorators: void 0 };\n  }\n");
    for name in ["start", "end", "left", "right"] {
        let cache = matches!(name, "left" | "right");
        writeln!(
            output,
            "  get {name}() {{\n    const internal = this.#internal;"
        )
        .unwrap();
        if cache {
            writeln!(output, "    const cached = internal.${name};\n    if (cached !== undefined) return cached;\n    return internal.${name} = internal.fields.{name}();").unwrap();
        } else {
            writeln!(output, "    return internal.fields.{name}();").unwrap();
        }
        output.push_str("  }\n");
    }
    if ts {
        output.push_str("  get decorators() {\n    const internal = this.#internal, cached = internal.$decorators;\n    if (cached !== undefined) return cached;\n    return internal.$decorators = internal.fields.decorators();\n  }\n  get optional() { return false; }\n  get typeAnnotation() { return null; }\n");
    }
    output.push_str("  toJSON() {\n    return { type: \"AssignmentPattern\", start: this.start, end: this.end, left: this.left, right: this.right,");
    if ts {
        output.push_str(" decorators: this.decorators, optional: false, typeAnnotation: null,");
    }
    output.push_str(" };\n  }\n  [inspectSymbol]() { return Object.setPrototypeOf(this.toJSON(), DebugAssignmentParameter.prototype); }\n}\nconst DebugAssignmentParameter = class AssignmentPattern {};\n\n");
}

fn special_fields(
    fields: &mut Vec<(String, String)>,
    ast: &AstStruct,
    raw: &[DecodedField],
    ts: bool,
) {
    let name = ast.name.as_str();
    let mut set = |key: &str, value: &str| {
        if let Some(field) = fields.iter_mut().find(|(k, _)| k == key) {
            field.1 = value.into();
        } else {
            fields.push((key.into(), value.into()));
        }
    };
    match name {
        "Program" => {
            set("body", &eager::combined_body_expression(raw));
            set("sourceType", "sourceType");
            if ts {
                set("start", "this.body.length !== 0 ? this.body[0].start : end");
            }
        }
        "FunctionBody" | "TsModuleBlock" => {
            set(
                "type",
                if name == "FunctionBody" {
                    "\"BlockStatement\""
                } else {
                    "\"TSModuleBlock\""
                },
            );
            set("body", &eager::combined_body_expression(raw));
        }
        "FormalParameter" => {
            // A parameter projects its pattern, without spreading (and thus reading) its fields.
            set(
                "project",
                if ts {
                    r#"f_initializer === null
                  ? f_pattern.withFields({ optional: () => f_optional, typeAnnotation: () => f_typeAnnotation, end: () => end, decorators: () => f_decorators })
                  : new AssignmentParameter({ type: () => "AssignmentPattern", start: () => f_pattern.start, end: () => end,
                      left: () => f_pattern.withFields({ optional: () => f_optional, typeAnnotation: () => f_typeAnnotation,
                        end: () => f_typeAnnotation?.end ?? (f_optional ? f_pattern.end + 1 : f_pattern.end) }),
                      right: () => f_initializer, decorators: () => f_decorators, optional: () => false, typeAnnotation: () => null }, ast)"#
                } else {
                    r#"f_initializer === null ? f_pattern.withFields({ end: () => end })
                  : new AssignmentParameter({ type: () => "AssignmentPattern", start: () => f_pattern.start, end: () => end,
                      left: () => f_pattern.withFields({}), right: () => f_initializer }, ast)"#
                },
            );
        }
        "TsIndexSignatureParameter" => {
            set(
                "project",
                "f_name.withFields({ start: () => start, end: () => end, typeAnnotation: () => f_typeAnnotation })",
            );
        }
        "FormalParameterRest" => {
            set("type", "\"RestElement\"");
            if ts {
                set("optional", "false");
                set("value", "null");
            }
        }
        "VariableDeclarator" if ts => {
            set(
                "id",
                "f_id.withFields({ typeAnnotation: () => f_typeAnnotation, end: () => f_typeAnnotation?.end ?? f_id.end })",
            );
        }
        "CatchClause" if ts => {
            set(
                "param",
                "f_param === null ? null : f_param.withFields({ typeAnnotation: () => f_typeAnnotation, end: () => f_typeAnnotation?.end ?? f_param.end })",
            );
        }
        "FormalParameters" => {
            set(
                "params",
                "f_rest === null ? f_items : appendNode(f_items, () => f_rest)",
            );
        }
        "Function" | "TsFunction" | "ArrowFunctionExpression" => {
            set(
                "type",
                match name {
                    "Function" => "f_functionType",
                    "TsFunction" => "typeOverride ?? \"TSDeclareFunction\"",
                    _ => "\"ArrowFunctionExpression\"",
                },
            );
            set(
                "params",
                if ts && name != "ArrowFunctionExpression" {
                    "f_thisParam === null ? f_params.params : prependNode(f_params.params, () => f_thisParam)"
                } else {
                    "f_params.params"
                },
            );
            if name == "ArrowFunctionExpression" {
                set("id", "null");
                set("generator", "false");
            } else {
                set("expression", "false");
            }
            if name == "TsFunction" {
                set("body", "null");
            }
            if ts && name == "Function" {
                set("declare", "false");
            }
        }
        "Directive" => {
            set("type", "\"ExpressionStatement\"");
            set("directive", "f_value");
        }
        "StringLiteral" | "NumericLiteral" | "BigIntLiteral" | "BooleanLiteral" | "NullLiteral"
        | "RegExpLiteral" => {
            set("type", "\"Literal\"");
            match name {
                "NumericLiteral" => set("raw", "f_raw ?? String(this.value)"),
                "BigIntLiteral" => {
                    set("raw", "`${f_raw}n`");
                    set("value", "BigInt(f_raw.replaceAll(\"_\", \"\"))");
                    set("bigint", "this.value.toString()");
                }
                "BooleanLiteral" => set("raw", "this.value ? \"true\" : \"false\""),
                "NullLiteral" => {
                    set("value", "null");
                    set("raw", "\"null\"");
                }
                "RegExpLiteral" => {
                    set(
                        "value",
                        "(() => { try { return new RegExp(f_pattern, f_flags); } catch { return null; } })()",
                    );
                    set("raw", "`/${f_pattern}/${f_flags}`");
                    set(
                        "regex",
                        "{ pattern: f_pattern, flags: [...f_flags].sort().join(\"\") }",
                    );
                }
                _ => {}
            }
        }
        "TemplateElement" => {
            set(
                "value",
                "{ raw: f_raw, cooked: f_isCookedUndefined ? null : f_cooked }",
            );
            if ts {
                set("start", "start - 1");
                set("end", "f_tail ? end + 1 : end + 2");
            }
        }
        "Class" => set("type", "f_classType"),
        "MethodDefinition" | "TsMethodDefinition" => {
            set(
                "type",
                if ts {
                    "f_abstract ? \"TSAbstractMethodDefinition\" : \"MethodDefinition\""
                } else {
                    "\"MethodDefinition\""
                },
            );
        }
        "PropertyDefinition" => {
            set(
                "type",
                if ts {
                    "f_abstract ? (f_accessor ? \"TSAbstractAccessorProperty\" : \"TSAbstractPropertyDefinition\") : (f_accessor ? \"AccessorProperty\" : \"PropertyDefinition\")"
                } else {
                    "f_accessor ? \"AccessorProperty\" : \"PropertyDefinition\""
                },
            );
        }
        "UnaryExpression" => set("prefix", "true"),
        "BindingProperty" => {
            set("type", "\"Property\"");
            set("kind", "\"init\"");
            set("method", "false");
            if ts {
                set("optional", "false");
            }
        }
        "ArrayPattern" | "ObjectPattern" => {
            let key = if name == "ArrayPattern" {
                "elements"
            } else {
                "properties"
            };
            set(
                key,
                &format!("f_rest === null ? f_{key} : appendNode(f_{key}, () => f_rest)"),
            );
            if ts {
                set("decorators", "[]");
                set("optional", "false");
                set("typeAnnotation", "null");
            }
        }
        "JsxText" => set("raw", "f_value"),
        "TsFunctionType"
        | "TsConstructorType"
        | "TsMethodSignature"
        | "TsCallSignatureDeclaration"
        | "TsConstructSignatureDeclaration" => {
            set(
                "params",
                if matches!(
                    name,
                    "TsFunctionType" | "TsMethodSignature" | "TsCallSignatureDeclaration"
                ) {
                    "f_thisParam === null ? f_params.params : prependNode(f_params.params, () => f_thisParam)"
                } else {
                    "f_params.params"
                },
            );
            if name == "TsMethodSignature" {
                set("accessibility", "null");
                set("readonly", "false");
                set("static", "false");
            }
        }
        "TsMappedType" => set("optional", "f_optional === null ? false : f_optional"),
        "TsModuleDeclaration" => {
            set("global", "false");
            set("body", "f_body === null ? undefined : f_body");
        }
        "TsGlobalDeclaration" => {
            set("type", "\"TSModuleDeclaration\"");
            set("kind", "\"global\"");
            set("global", "true");
        }
        "TsThisParameter" => {
            set("type", "\"Identifier\"");
            set("decorators", "[]");
            set("name", "\"this\"");
            set("optional", "false");
        }
        _ => {}
    }
    if !ts {
        fields.retain(|(key, _)| !match name {
            "Function" | "ArrowFunctionExpression" => {
                matches!(key.as_str(), "typeParameters" | "returnType")
            }
            "FormalParameterRest" => matches!(
                key.as_str(),
                "decorators" | "optional" | "typeAnnotation" | "value"
            ),
            "Class" => matches!(
                key.as_str(),
                "typeParameters" | "superTypeArguments" | "implements" | "abstract" | "declare"
            ),
            "MethodDefinition" => matches!(key.as_str(), "override" | "optional" | "accessibility"),
            "PropertyDefinition" => matches!(
                key.as_str(),
                "typeAnnotation"
                    | "declare"
                    | "override"
                    | "optional"
                    | "definite"
                    | "readonly"
                    | "accessibility"
            ),
            _ => false,
        });
    }
    // Test nullable slots without constructing their nodes. In particular, a
    // parameter array's length must not deserialize its rest or default value.
    for field in raw {
        if let Some(index) = field
            .expression
            .strip_prefix("optionalNode(")
            .and_then(|expr| expr.strip_suffix(')'))
        {
            for (_, expression) in fields.iter_mut() {
                *expression = expression.replace(
                    &format!("f_{} === null", field.name),
                    &format!("({index} === NULL)"),
                );
            }
        }
    }
    // Internal AST fields consumed by ESTree projections must never appear in serialized nodes.
    fields.retain(|(key, _)| !match name {
        "Program" | "FunctionBody" | "TsModuleBlock" => key == "directives",
        "FormalParameter" | "TsIndexSignatureParameter" => key != "project",
        "FormalParameters" => key != "params",
        "VariableDeclarator" | "CatchClause" => key == "typeAnnotation",
        "Function" | "TsFunction" => matches!(key.as_str(), "functionType" | "thisParam"),
        "Directive" => key == "value",
        "RegExpLiteral" => matches!(key.as_str(), "pattern" | "flags"),
        "TemplateElement" => matches!(key.as_str(), "raw" | "cooked" | "isCookedUndefined"),
        "Class" => key == "classType",
        "MethodDefinition" | "TsMethodDefinition" => key == "abstract",
        "PropertyDefinition" => matches!(key.as_str(), "accessor" | "abstract"),
        "ArrayPattern" | "ObjectPattern" => key == "rest",
        "TsFunctionType" | "TsMethodSignature" | "TsCallSignatureDeclaration" => key == "thisParam",
        _ => false,
    });
}

const LAZY_ARRAYS: &str = r#"  const nodeArray = (start, end, holes) => new NodeArray(start, end - start, 1, offset => {
    const index = _i32[(extra >> 2) + offset * 2];
    return holes && index === NULL ? null : node(index);
  }, ast);
  const nodeArrayPair = (firstStart, firstEnd, secondStart, secondEnd) => {
    const firstLength = firstEnd - firstStart;
    return new NodeArray(0, firstLength + secondEnd - secondStart, 1, offset =>
      node(_i32[(extra >> 2) + (offset < firstLength ? firstStart + offset : secondStart + offset - firstLength) * 2]), ast);
  };
  const appendNode = (array, construct) => new NodeArray(0, array.length + 1, 1,
    offset => offset < array.length ? array[offset] : construct(), ast);
  const prependNode = (array, construct) => new NodeArray(0, array.length + 1, 1,
    offset => offset === 0 ? construct() : array[offset - 1], ast);

"#;
