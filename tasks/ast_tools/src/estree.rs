//! Shared names and scalar values of the decoded ESTree representation.

use std::fmt::Write;

use convert_case::{Case, Casing};

use crate::schema::{AstStructField, EstreeAttrs, EstreeValue};

impl EstreeAttrs {
    pub fn name(&self, name: &str) -> String {
        self.rename.clone().unwrap_or_else(|| default_name(name))
    }

    pub fn tag(&self, name: &str) -> String {
        self.ty.clone().unwrap_or_else(|| self.name(name))
    }
}

pub(super) fn default_name(name: &str) -> String {
    match name {
        _ if name.starts_with("Jsx") => format!("JSX{}", &name[3..]),
        _ if name.starts_with("Ts") => format!("TS{}", &name[2..]),
        _ => name.into(),
    }
}

pub(super) fn estree_field_name(field: &AstStructField) -> String {
    field.attrs.estree_rename.clone().unwrap_or_else(|| {
        field
            .name
            .strip_prefix("r#")
            .unwrap_or(&field.name)
            .to_case(Case::Camel)
    })
}

/// Values are JavaScript/TypeScript literal source, in wire discriminant order.
pub(super) const ENUMS: &[(&str, &[&str])] = &[
    ("SourceType", &["\"module\"", "\"script\"", "\"commonjs\""]),
    (
        "MethodDefinitionKind",
        &["\"constructor\"", "\"method\"", "\"get\"", "\"set\""],
    ),
    (
        "Accessibility",
        &["null", "\"public\"", "\"private\"", "\"protected\""],
    ),
    (
        "ClassType",
        &["\"ClassDeclaration\"", "\"ClassExpression\""],
    ),
    (
        "BinaryOperator",
        &[
            "\"==\"",
            "\"!=\"",
            "\"===\"",
            "\"!==\"",
            "\"<\"",
            "\"<=\"",
            "\">\"",
            "\">=\"",
            "\"+\"",
            "\"-\"",
            "\"*\"",
            "\"/\"",
            "\"%\"",
            "\"**\"",
            "\"|\"",
            "\"^\"",
            "\"&\"",
            "\"<<\"",
            "\">>\"",
            "\">>>\"",
            "\"in\"",
            "\"instanceof\"",
        ],
    ),
    ("LogicalOperator", &["\"&&\"", "\"||\"", "\"??\""]),
    (
        "UnaryOperator",
        &[
            "\"-\"",
            "\"+\"",
            "\"!\"",
            "\"~\"",
            "\"typeof\"",
            "\"void\"",
            "\"delete\"",
        ],
    ),
    ("UpdateOperator", &["\"++\"", "\"--\""]),
    (
        "AssignmentOperator",
        &[
            "\"=\"", "\"+=\"", "\"-=\"", "\"*=\"", "\"/=\"", "\"%=\"", "\"**=\"", "\"<<=\"",
            "\">>=\"", "\">>>=\"", "\"|=\"", "\"^=\"", "\"&=\"", "\"||=\"", "\"&&=\"", "\"??=\"",
        ],
    ),
    (
        "VariableKind",
        &[
            "\"var\"",
            "\"let\"",
            "\"const\"",
            "\"using\"",
            "\"await using\"",
        ],
    ),
    (
        "FunctionType",
        &["\"FunctionDeclaration\"", "\"FunctionExpression\""],
    ),
    (
        "FormalParameterKind",
        &["\"formal\"", "\"unique\"", "\"arrow\"", "\"signature\""],
    ),
    ("PropertyKind", &["\"init\"", "\"get\"", "\"set\""]),
    ("ImportOrExportKind", &["\"value\"", "\"type\""]),
    ("ImportPhase", &["\"source\"", "\"defer\""]),
    (
        "TsTypeOperatorKind",
        &["\"keyof\"", "\"unique\"", "\"readonly\""],
    ),
    (
        "TsMethodSignatureKind",
        &["\"method\"", "\"get\"", "\"set\""],
    ),
    ("TsModuleDeclarationKind", &["\"namespace\"", "\"module\""]),
    ("TsMappedTypeModifier", &["null", "true", "\"+\"", "\"-\""]),
];

impl EstreeValue {
    pub fn js(&self) -> String {
        match self {
            Self::Null => "null".into(),
            Self::Bool(value) => value.to_string(),
            Self::String(value) => {
                let mut output = String::from("\"");
                for ch in value.chars() {
                    match ch {
                        '"' => output.push_str("\\\""),
                        '\\' => output.push_str("\\\\"),
                        '\u{0000}'..='\u{001f}' | '\u{2028}' | '\u{2029}' => {
                            write!(output, "\\u{:04x}", ch as u32).unwrap();
                        }
                        _ => output.push(ch),
                    }
                }
                output.push('"');
                output
            }
            Self::EmptyArray { .. } => "[]".into(),
        }
    }

    pub fn ts_type(&self) -> String {
        match self {
            Self::EmptyArray { element_type } => format!("Array<{element_type}>"),
            _ => self.js(),
        }
    }
}
