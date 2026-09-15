//! Value converters operate on named storage inputs, never binary offsets.

use std::collections::{BTreeSet, HashSet};

use super::{
    estree_field_name,
    types::{output_type, value_type},
};
use crate::schema::{AstStruct, EstreeFieldValue, Schema};

pub enum Input {
    Field {
        name: &'static str,
        ty: &'static str,
    },
    /// A final output field, after field conversion and list combination.
    Output {
        name: &'static str,
        ty: &'static str,
    },
    Context(Context),
    Converter(&'static str),
}

pub enum Context {
    Source,
    SourceType,
    IsTs,
    OriginalStart,
    OriginalEnd,
}

impl Context {
    pub fn expression(&self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::SourceType => "sourceType",
            Self::IsTs => "isTs",
            Self::OriginalStart => "start",
            Self::OriginalEnd => "end",
        }
    }
}

pub struct Converter {
    pub name: &'static str,
    pub inputs: &'static [(&'static str, Input)],
    pub output_type: &'static str,
    /// `{{result}}` and input placeholders are bound by the decoder generator.
    pub body: &'static str,
}

const fn field(name: &'static str, ty: &'static str) -> Input {
    Input::Field { name, ty }
}

const CONVERTERS: &[Converter] = &[
    Converter {
        name: "NumericLiteralRaw",
        inputs: &[
            ("raw", field("raw", "Option<AstUtf8>")),
            ("value", field("value", "f64")),
        ],
        output_type: "string",
        body: "const {{result}} = {{raw}} ?? String({{value}});",
    },
    Converter {
        name: "BooleanLiteralRaw",
        inputs: &[("value", field("value", "bool"))],
        output_type: "\"true\" | \"false\"",
        body: "const {{result}} = {{value}} ? \"true\" : \"false\";",
    },
    Converter {
        name: "BigIntValue",
        inputs: &[("raw", field("raw", "AstUtf8"))],
        output_type: "bigint",
        body: "const {{result}} = BigInt({{raw}}.replaceAll(\"_\", \"\"));",
    },
    Converter {
        name: "BigIntDecimal",
        inputs: &[("value", Input::Converter("BigIntValue"))],
        output_type: "string",
        body: "const {{result}} = {{value}}.toString();",
    },
    Converter {
        name: "SourceRaw",
        inputs: &[
            ("source", Input::Context(Context::Source)),
            ("start", Input::Context(Context::OriginalStart)),
            ("end", Input::Context(Context::OriginalEnd)),
        ],
        output_type: "string",
        body: "const {{result}} = {{source}}.slice({{start}}, {{end}});",
    },
    Converter {
        name: "RegExpValue",
        inputs: &[
            ("pattern", field("pattern", "AstUtf8")),
            ("flags", field("flags", "AstUtf8")),
        ],
        output_type: "RegExp | null",
        body: "let {{result}} = null;\ntry {\n  {{result}} = new RegExp({{pattern}}, {{flags}});\n} catch {}",
    },
    Converter {
        name: "RegExpRaw",
        inputs: &[
            ("pattern", field("pattern", "AstUtf8")),
            ("flags", field("flags", "AstUtf8")),
        ],
        output_type: "string",
        body: "const {{result}} = \"/\" + {{pattern}} + \"/\" + {{flags}};",
    },
    Converter {
        name: "RegExpRegex",
        inputs: &[
            ("pattern", field("pattern", "AstUtf8")),
            ("flags", field("flags", "AstUtf8")),
        ],
        output_type: "{ pattern: string; flags: string }",
        body: "const {{result}} = { pattern: {{pattern}}, flags: [...{{flags}}].sort().join(\"\") };",
    },
    Converter {
        name: "TemplateElementValue",
        inputs: &[
            ("raw", field("raw", "AstUtf8")),
            ("cooked", field("cooked", "AstWtf8")),
            ("undefined", field("is_cooked_undefined", "bool")),
        ],
        output_type: "{ raw: string; cooked: string | null }",
        body: "const {{result}} = { raw: {{raw}}, cooked: {{undefined}} ? null : {{cooked}} };",
    },
    Converter {
        name: "MappedTypeOptional",
        inputs: &[("optional", field("optional", "TsMappedTypeModifier"))],
        output_type: "boolean | \"+\" | \"-\"",
        body: "const {{result}} = {{optional}} === null ? false : {{optional}};",
    },
    Converter {
        name: "ProgramSourceType",
        inputs: &[("sourceType", Input::Context(Context::SourceType))],
        output_type: "SourceType",
        body: "const {{result}} = {{sourceType}};",
    },
    Converter {
        name: "ProgramStart",
        inputs: &[
            (
                "body",
                Input::Output {
                    name: "body",
                    ty: "Array<Statement>",
                },
            ),
            ("isTs", Input::Context(Context::IsTs)),
            ("start", Input::Context(Context::OriginalStart)),
            ("end", Input::Context(Context::OriginalEnd)),
        ],
        output_type: "number",
        body: "const {{result}} = {{isTs}} ? ({{body}}.length !== 0 ? {{body}}[0].start : {{end}}) : {{start}};",
    },
    Converter {
        name: "TemplateElementStart",
        inputs: &[
            ("isTs", Input::Context(Context::IsTs)),
            ("start", Input::Context(Context::OriginalStart)),
        ],
        output_type: "number",
        body: "const {{result}} = {{isTs}} ? {{start}} - 1 : {{start}};",
    },
    Converter {
        name: "TemplateElementEnd",
        inputs: &[
            ("isTs", Input::Context(Context::IsTs)),
            ("end", Input::Context(Context::OriginalEnd)),
            ("tail", field("tail", "bool")),
        ],
        output_type: "number",
        body: "const {{result}} = {{isTs}} ? ({{tail}} ? {{end}} + 1 : {{end}} + 2) : {{end}};",
    },
];

pub fn converter(name: &str) -> &'static Converter {
    CONVERTERS
        .iter()
        .find(|converter| converter.name == name)
        .unwrap_or_else(|| panic!("Unknown ESTree converter `{name}`"))
}

/// Dependencies are emitted once per node in topological order. Storage reads
/// remain separate from output fields, including fields replaced by converters.
pub struct ConversionPlan {
    pub steps: Vec<ConversionStep>,
    pub fields: BTreeSet<String>,
    completed: HashSet<String>,
}

pub enum ConversionStep {
    Converter(&'static Converter),
    Output(String),
}

impl ConversionPlan {
    pub fn new(ast: &AstStruct, schema: &Schema) -> Self {
        let mut plan = Self {
            steps: Vec::new(),
            fields: BTreeSet::new(),
            completed: HashSet::new(),
        };
        let mut active = HashSet::new();
        for field in &ast.fields {
            if let Some(name) = &field.attrs.estree_via {
                assert!(
                    ast.estree_emits_field(field)
                        && field.attrs.estree_type_override.is_none()
                        && field.attrs.estree_ts_type.is_none()
                        && !ast.estree_tag_fields().contains(field.name.as_str()),
                    "Conflicting ESTree field conversion: {}.{}",
                    ast.name,
                    field.name
                );
                assert!(
                    ast.estree_concat_sources(field).next().is_none(),
                    "Cannot combine and convert the same ESTree list"
                );
                plan.visit(converter(name), ast, schema, &mut active);
            }
        }
        for field in &ast.attrs.estree.added_fields {
            plan.visit_value(&field.value, ast, schema, &mut active);
        }
        let mut overrides = HashSet::new();
        for field in &ast.attrs.estree.overridden_fields {
            assert!(
                matches!(field.name.as_str(), "start" | "end"),
                "Only start and end can use override_field"
            );
            assert!(
                overrides.insert(&field.name),
                "Duplicate ESTree override {}.{}",
                ast.name,
                field.name
            );
            assert!(!field.ts, "Position overrides must be unconditional");
            plan.visit_value(&field.value, ast, schema, &mut active);
            assert_eq!(
                value_type(ast, &field.value, schema),
                "number",
                "Position overrides must produce number"
            );
        }
        plan
    }

    fn visit(
        &mut self,
        converter: &'static Converter,
        ast: &AstStruct,
        schema: &Schema,
        active: &mut HashSet<String>,
    ) {
        let key = format!("converter:{}", converter.name);
        if self.completed.contains(&key) {
            return;
        }
        assert!(
            active.insert(key.clone()),
            "ESTree computation dependency cycle at {}",
            converter.name
        );
        for (_, input) in converter.inputs {
            match input {
                Input::Field { name, ty } => {
                    let field = ast
                        .fields
                        .iter()
                        .find(|field| field.name == *name)
                        .unwrap_or_else(|| {
                            panic!(
                                "Converter {} needs storage field {}.{name}",
                                converter.name, ast.name
                            )
                        });
                    assert_eq!(
                        schema.types[field.type_id].name(),
                        *ty,
                        "Wrong input type for {} on {}.{name}",
                        converter.name,
                        ast.name
                    );
                    self.fields.insert((*name).into());
                }
                Input::Converter(name) => self.visit(self::converter(name), ast, schema, active),
                Input::Output { name, ty } => {
                    self.visit_output(name, ast, schema, active);
                    assert_eq!(
                        output_type(ast, name, schema),
                        *ty,
                        "Wrong output input type for {} on {}.{name}",
                        converter.name,
                        ast.name
                    );
                }
                Input::Context(_) => {}
            }
        }
        active.remove(&key);
        self.completed.insert(key);
        self.steps.push(ConversionStep::Converter(converter));
    }
    fn visit_value(
        &mut self,
        value: &EstreeFieldValue,
        ast: &AstStruct,
        schema: &Schema,
        active: &mut HashSet<String>,
    ) {
        match value {
            EstreeFieldValue::Constant(_) => {}
            EstreeFieldValue::From(name) => {
                assert!(
                    ast.fields.iter().any(|field| &field.name == name),
                    "Unknown ESTree source {}.{name}",
                    ast.name
                );
                self.fields.insert(name.clone());
            }
            EstreeFieldValue::Via(name) => self.visit(converter(name), ast, schema, active),
        }
    }

    fn visit_output(
        &mut self,
        name: &str,
        ast: &AstStruct,
        schema: &Schema,
        active: &mut HashSet<String>,
    ) {
        let key = format!("output:{name}");
        if self.completed.contains(&key) {
            return;
        }
        assert!(
            active.insert(key.clone()),
            "ESTree computation dependency cycle at {}.{name}",
            ast.name
        );
        if let Some(field) = ast
            .attrs
            .estree
            .overridden_fields
            .iter()
            .chain(&ast.attrs.estree.added_fields)
            .find(|field| field.name == name)
        {
            self.visit_value(&field.value, ast, schema, active);
        } else if !matches!(name, "start" | "end") {
            let field = ast
                .fields
                .iter()
                .find(|field| ast.estree_emits_field(field) && estree_field_name(field) == name)
                .unwrap_or_else(|| panic!("Unknown ESTree output {}.{name}", ast.name));
            if let Some(via) = &field.attrs.estree_via {
                self.visit(converter(via), ast, schema, active);
            }
        }
        active.remove(&key);
        self.completed.insert(key);
        self.steps.push(ConversionStep::Output(name.into()));
    }
}
