use std::fmt::Write;

use crate::{
    output::RawOutput,
    schema::{AstStruct, AstType, EstreeFieldValue, Schema},
    util::{InlineStorageMode, calculate_inline_layout},
};

use crate::{
    estree::{
        ENUMS,
        converters::{ConversionPlan, ConversionStep, Input},
        estree_field_name, tag_literal,
    },
    js_minifier::minify,
};

const PRETTY_OUTPUT_PATH: &str = "packages/decoder/index.pretty.js";
const MINIFIED_OUTPUT_PATH: &str = "packages/decoder/index.js";

pub fn estree_decoder(schema: &Schema) -> (RawOutput, RawOutput) {
    let mut output = String::new();
    output.push_str(PREAMBLE);
    output.push_str("const ENUMS = Object.freeze({\n");
    for (name, values) in ENUMS {
        writeln!(output, "  {name}: [{}],", values.join(", ")).unwrap();
    }
    output.push_str("});\n\n");
    output.push_str(DECODER);

    for (kind, ty) in schema
        .types
        .iter()
        .filter_map(|ty| match ty {
            AstType::Struct(ast) => Some(ast),
            _ => None,
        })
        .enumerate()
    {
        write_node_case(&mut output, kind, ty, schema);
    }

    output.push_str(POSTAMBLE);
    let minified = minify(&output, PRETTY_OUTPUT_PATH);
    (
        RawOutput {
            path: PRETTY_OUTPUT_PATH.into(),
            content: output.into_bytes(),
        },
        RawOutput {
            path: MINIFIED_OUTPUT_PATH.into(),
            content: minified.into_bytes(),
        },
    )
}

fn write_node_case(output: &mut String, kind: usize, ast: &AstStruct, schema: &Schema) {
    writeln!(output, "      case {kind}: {{").unwrap();

    let layout = calculate_inline_layout(ast, schema);
    if layout
        .as_ref()
        .is_none_or(|layout| layout.mode == InlineStorageMode::Partial)
    {
        writeln!(output, "        const extraId = _u32[(payload + 12) >> 2];").unwrap();
    }
    let mut fields = Vec::new();
    let tag_fields = ast.estree_tag_fields();
    let conversions = ConversionPlan::new(ast, schema);

    let mut extra_offset = 0usize;
    for (field_index, field) in ast.fields.iter().enumerate() {
        let skip = field.attrs.estree_skip;
        let field_ty = &schema.types[field.type_id];
        let (location, aligned_u32) = if let Some((_, byte_offset, _)) =
            layout.as_ref().and_then(|layout| {
                layout
                    .fields
                    .iter()
                    .find(|(index, _, _)| *index == field_index)
            }) {
            (format!("payload + {byte_offset}"), byte_offset % 4 == 0)
        } else {
            let location = format!("extra + (extraId + {extra_offset}) * 8");
            extra_offset += 1;
            (location, true)
        };
        if skip
            && !tag_fields.contains(field.name.as_str())
            && !conversions.fields.contains(&field.name)
        {
            continue;
        }
        let mut expression = read_expression(field_ty, schema, &location, aligned_u32);
        let field_name = estree_field_name(field);
        if conversions.fields.contains(&field.name) {
            writeln!(output, "        const storage_{field_name} = {expression};").unwrap();
            expression = format!("storage_{field_name}");
        }
        if let Some(tag) = &field.attrs.estree_type_override {
            writeln!(output, "        const child_{field_name} = {expression};").unwrap();
            writeln!(
                output,
                "        child_{field_name}.type = {};",
                tag_literal(tag)
            )
            .unwrap();
            expression = format!("child_{field_name}");
        }
        if tag_fields.contains(field.name.as_str()) {
            writeln!(output, "        const tag_{field_name} = {expression};").unwrap();
            expression = format!("tag_{field_name}");
        }
        if skip || ast.attrs.estree.type_from.as_ref() == Some(&field.name) {
            continue;
        }
        if let Some(converter) = &field.attrs.estree_via {
            expression = format!("converted_{converter}");
        }
        fields.push(DecodedField {
            name: field_name,
            expression,
            ts: field.attrs.estree_ts,
        });
    }

    // Materialize splice inputs once, before reusing them in list expressions.
    if ast
        .fields
        .iter()
        .any(|field| field.attrs.estree_concat.is_some())
    {
        for field in &mut fields {
            let storage = ast
                .fields
                .iter()
                .find(|storage| estree_field_name(storage) == field.name)
                .unwrap();
            // Converted fields are scheduled below with their output dependencies.
            if storage.attrs.estree_via.is_some() {
                continue;
            }
            writeln!(
                output,
                "        const c_{} = {};",
                field.name, field.expression
            )
            .unwrap();
            field.expression = format!("c_{}", field.name);
        }
        for target in &ast.fields {
            let sources: Vec<_> = ast.estree_concat_sources(target).collect();
            let single_optional = match sources.as_slice() {
                [source] if matches!(schema.types[source.type_id], AstType::Option(_)) => {
                    Some(estree_field_name(source))
                }
                _ => None,
            };
            let mut before = Vec::new();
            let mut after = Vec::new();
            for source in sources {
                let name = estree_field_name(source);
                let contribution = match &schema.types[source.type_id] {
                    AstType::SubRange(_) => format!("...c_{name}"),
                    AstType::Option(_) if single_optional.is_some() => format!("c_{name}"),
                    AstType::Option(_) => format!("...(c_{name} === null ? [] : [c_{name}])"),
                    _ => unreachable!("validated list source"),
                };
                if source.attrs.estree_concat.as_ref().unwrap().prepend {
                    before.push(contribution);
                } else {
                    after.push(contribution);
                }
            }
            if before.is_empty() && after.is_empty() {
                continue;
            }
            let name = estree_field_name(target);
            before.push(format!("...c_{name}"));
            before.extend(after);
            let combined = format!("[{}]", before.join(", "));
            fields
                .iter_mut()
                .find(|field| field.name == name)
                .unwrap()
                .expression = if let Some(source) = single_optional {
                // Keep the decoded destination array when an optional source is absent.
                format!("c_{source} === null ? c_{name} : {combined}")
            } else {
                combined
            };
        }
        fields.retain(|field| {
            !ast.fields.iter().any(|source| {
                source.attrs.estree_concat.is_some() && estree_field_name(source) == field.name
            })
        });
    }

    fields.extend(added_fields(ast));
    for step in &conversions.steps {
        let converter = match step {
            ConversionStep::Converter(converter) => converter,
            ConversionStep::Output(name) => {
                let expression = fields
                    .iter()
                    .find(|field| &field.name == name)
                    .map_or_else(|| name.clone(), |field| field.expression.clone());
                writeln!(output, "        const output_{name} = {expression};").unwrap();
                if let Some(field) = fields.iter_mut().find(|field| &field.name == name) {
                    field.expression = format!("output_{name}");
                }
                continue;
            }
        };
        let mut body = converter
            .body
            .replace("{{result}}", &format!("converted_{}", converter.name));
        for (binding, input) in converter.inputs {
            let expression = match input {
                Input::Field { name, .. } => {
                    let field = ast.fields.iter().find(|field| field.name == *name).unwrap();
                    format!("storage_{}", estree_field_name(field))
                }
                Input::Output { name, .. } => format!("output_{name}"),
                Input::Context(context) => context.expression().into(),
                Input::Converter(name) => format!("converted_{name}"),
            };
            body = body.replace(&format!("{{{{{binding}}}}}"), &expression);
        }
        assert!(
            !body.contains("{{"),
            "Unresolved converter input in {}",
            converter.name
        );
        for line in body.lines() {
            writeln!(output, "        {line}").unwrap();
        }
    }

    if let Some(target) = ast.estree_transparent_field() {
        let name = estree_field_name(target);
        let field = fields.iter().find(|field| field.name == name).unwrap();
        writeln!(output, "        result = {};", field.expression).unwrap();
    } else if is_special_node(&ast.name) {
        write_special_node(output, ast, &fields);
        let additional_fields: Vec<_> = fields
            .iter()
            .filter(|field| {
                field.ts
                    || ast
                        .attrs
                        .estree
                        .added_fields
                        .iter()
                        .chain(&ast.attrs.estree.overridden_fields)
                        .any(|added| added.name == field.name)
            })
            .map(|field| DecodedField {
                name: field.name.clone(),
                expression: format!("f_{}", field.name),
                ts: field.ts,
            })
            .collect();
        write_field_assignments(output, &additional_fields);
    } else {
        write_generic_node(output, ast, &fields);
    }
    writeln!(output, "        break;").unwrap();
    writeln!(output, "      }}").unwrap();
}

struct DecodedField {
    name: String,
    expression: String,
    ts: bool,
}

fn read_expression(ty: &AstType, schema: &Schema, location: &str, aligned_u32: bool) -> String {
    match ty {
        AstType::Struct(_) | AstType::Enum(_) => {
            format!("node({})", read_u32(location, aligned_u32))
        }
        AstType::SubRange(range) => {
            let inner = &schema.types[range.inner_type_id];
            let holes = matches!(inner, AstType::Option(_));
            let start = read_u32(location, aligned_u32);
            let end = read_u32(&format!("{location} + 4"), aligned_u32);
            format!("nodeArray({start}, {end}, {holes})")
        }
        AstType::Option(option) => match &schema.types[option.inner_type_id] {
            AstType::Struct(_) | AstType::Enum(_) => {
                format!("optionalNode({})", read_u32(location, aligned_u32))
            }
            AstType::Primitive(primitive) if primitive.name == "ImportPhase" => {
                format!("optionalEnum(\"ImportPhase\", _dv.getUint8({location}))")
            }
            AstType::Primitive(primitive) if matches!(primitive.name, "ScopeId" | "SymbolId") => {
                "undefined".into()
            }
            AstType::Primitive(primitive) if primitive.name == "Span" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("{{ start: pos({start}), end: pos({end}) }}")
            }
            AstType::Primitive(primitive) if primitive.name == "AstUtf8" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("{start} === NULL && {end} === 0 ? null : str({start}, {end})")
            }
            inner => panic!("unsupported optional decoder type: {}", inner.name()),
        },
        AstType::Primitive(primitive) => match primitive.name {
            "AstUtf8" | "AstWtf8" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("str({start}, {end})")
            }
            "bool" => format!("_dv.getUint8({location}) !== 0"),
            "u8" => format!("_dv.getUint8({location})"),
            "i8" => format!("_dv.getInt8({location})"),
            "u16" => format!("_dv.getUint16({location}, true)"),
            "i16" => format!("_dv.getInt16({location}, true)"),
            "u32" => read_u32(location, aligned_u32),
            "i32" => format!("_dv.getInt32({location}, true)"),
            "f64" => format!("_dv.getFloat64({location}, true)"),
            "Span" => {
                let start = read_u32(location, aligned_u32);
                let end = read_u32(&format!("{location} + 4"), aligned_u32);
                format!("{{ start: pos({start}), end: pos({end}) }}")
            }
            "ScopeId" | "SymbolId" => "undefined".into(),
            name if schema.repr_sizes.contains_key(name)
                || matches!(
                    name,
                    "SourceType"
                        | "ImportPhase"
                        | "MethodDefinitionKind"
                        | "Accessibility"
                        | "ClassType"
                        | "BinaryOperator"
                        | "LogicalOperator"
                        | "UnaryOperator"
                        | "UpdateOperator"
                        | "AssignmentOperator"
                        | "VariableKind"
                        | "FunctionType"
                        | "FormalParameterKind"
                        | "PropertyKind"
                        | "ImportOrExportKind"
                        | "TsTypeOperatorKind"
                        | "TsMethodSignatureKind"
                        | "TsModuleDeclarationKind"
                        | "TsMappedTypeModifier"
                ) =>
            {
                format!("enumValue(\"{name}\", _dv.getUint8({location}))")
            }
            name => panic!("unsupported primitive decoder type: {name}"),
        },
    }
}

fn read_u32(location: &str, aligned: bool) -> String {
    if aligned {
        format!("_u32[({location}) >> 2]")
    } else {
        format!("_dv.getUint32({location}, true)")
    }
}

fn added_fields(ast: &AstStruct) -> impl Iterator<Item = DecodedField> + '_ {
    ast.attrs
        .estree
        .added_fields
        .iter()
        .chain(&ast.attrs.estree.overridden_fields)
        .map(|field| DecodedField {
            name: field.name.clone(),
            expression: match &field.value {
                EstreeFieldValue::Constant(value) => value.js(),
                EstreeFieldValue::From(name) => {
                    let source = ast.fields.iter().find(|field| &field.name == name).unwrap();
                    format!("storage_{}", estree_field_name(source))
                }
                EstreeFieldValue::Via(name) => format!("converted_{name}"),
            },
            ts: field.ts,
        })
}

fn write_generic_node(output: &mut String, ast: &AstStruct, fields: &[DecodedField]) {
    if fields.iter().any(|field| field.ts) {
        writeln!(output, "        if (isTs) {{").unwrap();
        write_object_literal(output, ast, fields.iter(), 10);
        writeln!(output, "        }} else {{").unwrap();
        write_object_literal(output, ast, fields.iter().filter(|field| !field.ts), 10);
        writeln!(output, "        }}").unwrap();
    } else {
        write_object_literal(output, ast, fields.iter(), 8);
    }
}

fn write_object_literal<'a>(
    output: &mut String,
    ast: &AstStruct,
    fields: impl Iterator<Item = &'a DecodedField>,
    indent: usize,
) {
    let padding = " ".repeat(indent);
    let fields: Vec<_> = fields.collect();
    let positions = ["start", "end"].map(|name| {
        fields.iter().find(|field| field.name == name).map_or_else(
            || name.into(),
            |field| format!("{name}: {}", field.expression),
        )
    });
    writeln!(
        output,
        "{padding}result = {{ type: {}, {}, {},",
        ast.estree_tag_expression(),
        positions[0],
        positions[1]
    )
    .unwrap();
    for field in fields
        .into_iter()
        .filter(|field| !matches!(field.name.as_str(), "start" | "end"))
    {
        writeln!(output, "{padding}  {}: {},", field.name, field.expression).unwrap();
    }
    writeln!(output, "{padding}}};").unwrap();
}

fn write_field_assignments(output: &mut String, fields: &[DecodedField]) {
    for field in fields.iter().filter(|field| !field.ts) {
        writeln!(
            output,
            "        result.{} = {};",
            field.name, field.expression
        )
        .unwrap();
    }
    if fields.iter().any(|field| field.ts) {
        writeln!(output, "        if (isTs) {{").unwrap();
        for field in fields.iter().filter(|field| field.ts) {
            writeln!(
                output,
                "          result.{} = {};",
                field.name, field.expression
            )
            .unwrap();
        }
        writeln!(output, "        }}").unwrap();
    }
}

fn is_special_node(name: &str) -> bool {
    matches!(name, "TsModuleDeclaration")
}

fn write_special_node(output: &mut String, ast: &AstStruct, fields: &[DecodedField]) {
    let name = ast.name.as_str();
    let tag = ast.attrs.estree.tag(name);
    for field in fields {
        writeln!(
            output,
            "        const f_{} = {};",
            field.name, field.expression
        )
        .unwrap();
    }

    match name {
        "TsModuleDeclaration" => write!(
            output,
            r#"        result = {{
          type: "{tag}", start, end,
          id: f_id, kind: f_kind, declare: f_declare,
        }};
        if (f_body !== null) result.body = f_body;
"#,
        )
        .unwrap(),
        _ => unreachable!("missing special ESTree decoder for {name}"),
    }
}

const PREAMBLE: &str = r#"// generated by tasks/ast_tools, do not edit

const NULL = 0xffffffff;
const HEADER_SIZE = 36;
const NATIVE_LITTLE_ENDIAN =
  new Uint8Array(new Uint32Array([1]).buffer)[0] === 1;
const textDecoder = new TextDecoder("utf-8", { ignoreBOM: true });

"#;

const DECODER: &str = r#"const OPTION_SOURCE_TYPES = ["module", "script", "commonjs", "unambiguous"];
const OPTION_LANGS = ["js", "ts", "jsx", "tsx", "dts"];
const OPTION_COMMENT_MODES = ["none", "flat"];

export function encodeOptions(options) {
  if (options === undefined || options === null) return 0;
  if (typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("options must be an object");
  }

  let value = 1;
  value |= encodeOptionEnum(options.sourceType, "sourceType", OPTION_SOURCE_TYPES) << 1;
  value |= encodeOptionEnum(options.lang, "lang", OPTION_LANGS) << 4;
  value |= encodeOptionBoolean(options.preserveParens, "preserveParens") << 7;
  value |= encodeOptionEnum(options.comments, "comments", OPTION_COMMENT_MODES) << 9;
  return value >>> 0;
}

function encodeOptionEnum(value, name, values) {
  if (value === undefined || value === null) return 0;
  const index = values.indexOf(value);
  if (index === -1) {
    throw new TypeError(`${name} must be one of: ${values.join(", ")}`);
  }
  return index + 1;
}

function encodeOptionBoolean(value, name) {
  if (value === undefined || value === null) return 0;
  if (typeof value !== "boolean") {
    throw new TypeError(`${name} must be a boolean`);
  }
  return value ? 2 : 1;
}

function buildPosMap(source, byteLength) {
  const map = new Uint32Array(byteLength + 1);
  let byte = 0;
  for (let utf16 = 0; utf16 < source.length; utf16++) {
    const first = source.charCodeAt(utf16);
    const isPair =
      first >= 0xd800 &&
      first <= 0xdbff &&
      utf16 + 1 < source.length &&
      source.charCodeAt(utf16 + 1) >= 0xdc00 &&
      source.charCodeAt(utf16 + 1) <= 0xdfff;
    const width =
      first < 0x80 ? 1 :
      first < 0x800 ? 2 :
      isPair ? 4 : 3;
    for (let i = 0; i < width; i++) map[byte + i] = utf16;
    byte += width;
    if (width === 4) {
      map[byte - 1] = utf16 + 1;
      utf16++;
    }
  }
  map[byteLength] = source.length;
  return map;
}

function decodePool(bytes) {
  let hasSurrogate = false;
  for (const byte of bytes) {
    if (byte === 0xed) {
      hasSurrogate = true;
      break;
    }
  }
  if (!hasSurrogate) return textDecoder.decode(bytes);

  let result = "";
  for (let i = 0; i < bytes.length;) {
    const first = bytes[i];
    if (first < 0x80) {
      result += String.fromCharCode(first);
      i++;
    } else if (first < 0xe0) {
      result += String.fromCharCode(((first & 0x1f) << 6) | (bytes[i + 1] & 0x3f));
      i += 2;
    } else if (first < 0xf0) {
      result += String.fromCharCode(
        ((first & 0x0f) << 12) |
        ((bytes[i + 1] & 0x3f) << 6) |
        (bytes[i + 2] & 0x3f),
      );
      i += 3;
    } else {
      result += String.fromCodePoint(
        ((first & 0x07) << 18) |
        ((bytes[i + 1] & 0x3f) << 12) |
        ((bytes[i + 2] & 0x3f) << 6) |
        (bytes[i + 3] & 0x3f),
      );
      i += 4;
    }
  }
  return result;
}

function copyLittleEndianU32(view) {
  const u32 = new Uint32Array(view.byteLength >> 2);
  for (let offset = 0; offset + 4 <= view.byteLength; offset += 4) {
    u32[offset >> 2] = view.getUint32(offset, true);
  }
  return u32;
}

export function decode(buffer, source) {
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const byteOffset = buffer.byteOffset ?? 0;
  const byteLength = buffer.byteLength;
  const bytes = new Uint8Array(arrayBuffer, byteOffset, byteLength);
  const _dv = new DataView(arrayBuffer, byteOffset, byteLength);
  const _u32 = NATIVE_LITTLE_ENDIAN && (byteOffset & 3) === 0
    ? new Uint32Array(arrayBuffer, byteOffset, byteLength >> 2)
    : copyLittleEndianU32(_dv);
  const nodeCount = _u32[0];
  const extraCount = _u32[1];
  const stringPoolLength = _u32[2];
  const sourceLength = _u32[3];
  const commentCount = _u32[4];
  const diagnosticCount = _u32[6];
  const programIndex = _u32[7];
  const flags = _u32[8];
  const isTs = (flags & 1) !== 0;

  const spans = HEADER_SIZE;
  const kinds = spans + nodeCount * 8;
  const payloads = (kinds + nodeCount + 3) & ~3;
  const extra = payloads + nodeCount * 16;
  const strings = extra + extraCount * 8;
  const comments = strings + ((stringPoolLength + 3) & ~3);
  const diagnostics = comments + commentCount * 16;
  const positions =
    source.length === sourceLength ? null : buildPosMap(source, sourceLength);

  const pos = positions === null ? byte => byte : byte => positions[byte];
  const str = (start, end) => {
    if (start === end) return "";
    if (start < sourceLength) return source.slice(pos(start), pos(end));
    return decodePool(bytes.subarray(
      strings + start - sourceLength,
      strings + end - sourceLength,
    ));
  };
  const enumValue = (name, value) => {
    const values = ENUMS[name];
    return values !== undefined && value < values.length ? values[value] : value;
  };
  const sourceType = enumValue("SourceType", (flags >> 1) & 3);
  const optionalEnum = (name, value) =>
    value === 0xff ? null : enumValue(name, value);
  const optionalNode = index => index === NULL ? null : node(index);
  const nodeArray = (start, end, holes) => {
    const result = new Array(end - start);
    for (let i = start; i < end; i++) {
      const index = _u32[(extra >> 2) + i * 2];
      result[i - start] = holes && index === NULL ? null : node(index);
    }
    return result;
  };

  const _decodeComments = () => {
    const result = new Array(commentCount);
    for (let index = 0; index < commentCount; index++) {
      const offset = comments + index * 16;
      const start = _u32[offset >> 2];
      const end = _u32[(offset >> 2) + 1];
      const kind = _dv.getUint16(offset + 12, true);
      const first = source.charCodeAt(pos(start));
      const valueStart = start + (first === 0x3c ? 4 : first === 0x2d ? 3 : 2);
      const valueEnd = end - (kind === 1 ? 2 : 0);
      result[index] = {
        type: kind === 1 ? "Block" : "Line",
        value: str(valueStart, valueEnd),
        start: pos(start),
        end: pos(end),
      };
    }
    return result;
  };

  const _decodeDiagnostics = () => {
    let offset = diagnostics;
    const readU8 = () => bytes[offset++];
    const readU32 = () => {
      const value = _dv.getUint32(offset, true);
      offset += 4;
      return value;
    };
    const readDiagnosticString = () => {
      const length = readU32();
      const value = textDecoder.decode(bytes.subarray(offset, offset + length));
      offset += length;
      return value;
    };
    const severities = ["error", "warning", "hint", "info"];
    const result = new Array(diagnosticCount);
    for (let index = 0; index < diagnosticCount; index++) {
      const severityValue = readU8();
      const severity = severities[severityValue];
      if (severity === undefined) {
        throw new RangeError(`Unknown diagnostic severity ${severityValue}`);
      }
      const start = pos(readU32());
      const end = pos(readU32());
      const message = readDiagnosticString();
      const help = readU8() === 0 ? undefined : readDiagnosticString();
      const labelCount = readU32();
      const labels = new Array(labelCount);
      for (let label = 0; label < labelCount; label++) {
        labels[label] = {
          start: pos(readU32()),
          end: pos(readU32()),
          message: readDiagnosticString(),
        };
      }
      result[index] = { severity, message, start, end, help, labels };
    }
    return result;
  };

  function node(index) {
    const span = spans + index * 8;
    const start = pos(_u32[span >> 2]);
    const end = pos(_u32[(span >> 2) + 1]);
    const payload = payloads + index * 16;
    let result;
    switch (bytes[kinds + index]) {
"#;

const POSTAMBLE: &str = r#"      default:
        throw new RangeError(`Unknown AST node kind ${bytes[kinds + index]}`);
    }
    return result;
  }

  let _program, _comments, _diagnostics;
  return {
    get program() {
      return _program !== undefined
        ? _program
        : (_program = node(programIndex));
    },
    get comments() {
      return _comments !== undefined
        ? _comments
        : (_comments = _decodeComments());
    },
    get diagnostics() {
      return _diagnostics !== undefined
        ? _diagnostics
        : (_diagnostics = _decodeDiagnostics());
    },
  };
}
"#;
