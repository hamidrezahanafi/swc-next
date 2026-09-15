//! Shared names and scalar values of the decoded ESTree representation.

use convert_case::{Case, Casing};

use crate::schema::{AstStructField, EstreeAttrs};

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

pub(super) fn is_ts_field(node: &str, field: &str) -> bool {
    matches!(
        (node, field),
        ("VariableDeclaration", "declare")
            | ("VariableDeclarator", "definite")
            | ("CallExpression", "typeArguments")
            | ("NewExpression", "typeArguments")
            | ("TaggedTemplateExpression", "typeArguments")
            | ("JsxOpeningElement", "typeArguments")
            | ("ImportDeclaration", "importKind")
            | ("ImportSpecifier", "importKind")
            | ("ExportNamedDeclaration", "exportKind")
            | ("ExportAllDeclaration", "exportKind")
            | ("ExportSpecifier", "exportKind")
            | (
                "BindingRestElement",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "BindingIdentifier",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "AssignmentPattern",
                "decorators" | "optional" | "typeAnnotation"
            )
            | (
                "ObjectPattern",
                "decorators" | "optional" | "typeAnnotation"
            )
            | ("ArrayPattern", "decorators" | "optional" | "typeAnnotation")
            | ("TsIndexSignature", "static")
    )
}

const IDENTIFIER_TS_EXTRAS: &[(&str, &str)] = &[
    ("decorators", "[]"),
    ("optional", "false"),
    ("typeAnnotation", "null"),
];
const OBJECT_PROPERTY_TS_EXTRAS: &[(&str, &str)] = &[("optional", "false")];
const EXPRESSION_STATEMENT_TS_EXTRAS: &[(&str, &str)] = &[("directive", "null")];
const BINDING_REST_TS_EXTRAS: &[(&str, &str)] = &[("value", "null")];
const EXPORT_DEFAULT_TS_EXTRAS: &[(&str, &str)] = &[("exportKind", "\"value\"")];
const TS_PROPERTY_SIGNATURE_EXTRAS: &[(&str, &str)] =
    &[("accessibility", "null"), ("static", "false")];
const TS_INDEX_SIGNATURE_EXTRAS: &[(&str, &str)] = &[("accessibility", "null")];
const TS_PARAMETER_PROPERTY_EXTRAS: &[(&str, &str)] = &[("static", "false")];

pub(super) fn ts_extra_fields(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "IdentifierReference" | "IdentifierName" | "LabelIdentifier" => IDENTIFIER_TS_EXTRAS,
        "ObjectProperty" => OBJECT_PROPERTY_TS_EXTRAS,
        "ExpressionStatement" => EXPRESSION_STATEMENT_TS_EXTRAS,
        "BindingRestElement" => BINDING_REST_TS_EXTRAS,
        "ExportDefaultDeclaration" => EXPORT_DEFAULT_TS_EXTRAS,
        "TsPropertySignature" => TS_PROPERTY_SIGNATURE_EXTRAS,
        "TsIndexSignature" => TS_INDEX_SIGNATURE_EXTRAS,
        "TsParameterProperty" => TS_PARAMETER_PROPERTY_EXTRAS,
        _ => &[],
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
