//! Finite node-tag selectors shared by decoding, declarations, and encoding.

use std::collections::{BTreeSet, HashMap};

use super::estree_field_name;
use crate::schema::{
    AstStruct, AstStructField, AstType, EstreeTagCondition, EstreeTagSelector, EstreeValue, Schema,
    TypeId,
};

pub fn tag_literal(tag: &str) -> String {
    EstreeValue::String(tag.into()).js()
}

impl AstStruct {
    pub fn estree_tag_fields(&self) -> BTreeSet<&str> {
        let mut fields = BTreeSet::new();
        if let Some(field) = &self.attrs.estree.type_from {
            fields.insert(field.as_str());
        }
        if let Some(selector) = &self.attrs.estree.type_via {
            selector.fields(&mut fields);
        }
        fields
    }

    pub fn estree_emits_field(&self, field: &AstStructField) -> bool {
        !field.attrs.estree_skip
            && field.attrs.estree_concat.is_none()
            && self.attrs.estree.type_from.as_ref() != Some(&field.name)
    }

    pub fn estree_tags(&self, schema: &Schema) -> Vec<String> {
        if let Some(name) = &self.attrs.estree.type_from {
            let field = self
                .fields
                .iter()
                .find(|field| &field.name == name)
                .expect("unknown type_from field");
            let name = schema.types[field.type_id].name();
            let scalar = schema
                .scalar_enums
                .get(name)
                .expect("type_from requires a scalar enum mapping");
            return scalar
                .variants
                .iter()
                .filter(|variant| !variant.skip)
                .map(|variant| {
                    let crate::schema::EstreeValue::String(value) = &variant.value else {
                        panic!("node tags must be strings");
                    };
                    value.clone()
                })
                .collect();
        }
        if let Some(selector) = &self.attrs.estree.type_via {
            let mut tags = Vec::new();
            selector.tags(&mut tags);
            return tags;
        }
        vec![self.attrs.estree.tag(&self.name)]
    }

    pub fn estree_tag_expression(&self) -> String {
        let field = |name: &str| {
            let field = self.fields.iter().find(|field| field.name == name).unwrap();
            format!("tag_{}", estree_field_name(field))
        };
        if let Some(name) = &self.attrs.estree.type_from {
            return field(name);
        }
        if let Some(selector) = &self.attrs.estree.type_via {
            return selector.js(&field);
        }
        tag_literal(&self.attrs.estree.tag(&self.name))
    }
}

impl EstreeTagSelector {
    fn fields<'a>(&'a self, fields: &mut BTreeSet<&'a str>) {
        if let Self::If {
            condition,
            consequent,
            alternate,
        } = self
        {
            condition.fields(fields);
            consequent.fields(fields);
            alternate.fields(fields);
        }
    }

    fn tags(&self, tags: &mut Vec<String>) {
        match self {
            Self::Tag(tag) => {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
            Self::If {
                consequent,
                alternate,
                ..
            } => {
                alternate.tags(tags);
                consequent.tags(tags);
            }
        }
    }

    fn js(&self, field: &impl Fn(&str) -> String) -> String {
        match self {
            Self::Tag(tag) => tag_literal(tag),
            Self::If {
                condition,
                consequent,
                alternate,
            } => format!(
                "({} ? {} : {})",
                condition.js(field),
                consequent.js(field),
                alternate.js(field)
            ),
        }
    }

    fn evaluate(&self, is_ts: bool, field: &impl Fn(&str) -> bool) -> &str {
        match self {
            Self::Tag(tag) => tag,
            Self::If {
                condition,
                consequent,
                alternate,
            } => if condition.evaluate(is_ts, field) {
                consequent
            } else {
                alternate
            }
            .evaluate(is_ts, field),
        }
    }

    /// With TS tags enabled, each tag must identify the stored boolean flags.
    /// JS tags may intentionally discard TS-only distinctions (e.g. abstract).
    pub fn inverse(&self) -> HashMap<&str, BTreeSet<&str>> {
        let mut fields = BTreeSet::new();
        self.fields(&mut fields);
        let fields: Vec<_> = fields.into_iter().collect();
        assert!(
            fields.len() <= 8,
            "Tag selectors support at most eight boolean fields"
        );
        let mut inverse = HashMap::new();
        for mask in 0..(1 << fields.len()) {
            let enabled: BTreeSet<_> = fields
                .iter()
                .enumerate()
                .filter_map(|(bit, name)| ((mask & (1 << bit)) != 0).then_some(*name))
                .collect();
            let tag = self.evaluate(true, &|name| enabled.contains(name));
            if let Some(previous) = inverse.insert(tag, enabled.clone()) {
                assert_eq!(previous, enabled, "Ambiguous encoder flags for tag {tag}");
            }
        }
        inverse
    }
}

impl EstreeTagCondition {
    fn fields<'a>(&'a self, fields: &mut BTreeSet<&'a str>) {
        match self {
            Self::Field(name) => {
                fields.insert(name);
            }
            Self::IsTs => {}
            Self::Not(value) => value.fields(fields),
            Self::And(left, right) | Self::Or(left, right) => {
                left.fields(fields);
                right.fields(fields);
            }
        }
    }

    fn js(&self, field: &impl Fn(&str) -> String) -> String {
        match self {
            Self::Field(name) => field(name),
            Self::IsTs => "isTs".into(),
            Self::Not(value) => format!("!({})", value.js(field)),
            Self::And(left, right) => format!("({} && {})", left.js(field), right.js(field)),
            Self::Or(left, right) => format!("({} || {})", left.js(field), right.js(field)),
        }
    }

    fn evaluate(&self, is_ts: bool, field: &impl Fn(&str) -> bool) -> bool {
        match self {
            Self::Field(name) => field(name),
            Self::IsTs => is_ts,
            Self::Not(value) => !value.evaluate(is_ts, field),
            Self::And(left, right) => left.evaluate(is_ts, field) && right.evaluate(is_ts, field),
            Self::Or(left, right) => left.evaluate(is_ts, field) || right.evaluate(is_ts, field),
        }
    }
}

/// Additional public views requested by parent fields, retaining the child's fields.
pub fn overridden_tags(type_id: TypeId, schema: &Schema) -> BTreeSet<String> {
    schema
        .types
        .iter()
        .filter_map(|ty| {
            if let AstType::Struct(ast) = ty {
                Some(ast)
            } else {
                None
            }
        })
        .flat_map(|ast| &ast.fields)
        .filter(|field| field.type_id == type_id)
        .filter_map(|field| field.attrs.estree_type_override.clone())
        .collect()
}

pub fn validate_tags(schema: &Schema) {
    for ty in &schema.types {
        match ty {
            AstType::Struct(ast) => {
                let attrs = &ast.attrs.estree;
                assert!(
                    [
                        attrs.ty.is_some(),
                        attrs.type_from.is_some(),
                        attrs.type_via.is_some(),
                        attrs.transparent.is_some()
                    ]
                    .into_iter()
                    .filter(|value| *value)
                    .count()
                        <= 1,
                    "Conflicting ESTree output selectors on {}",
                    ast.name
                );
                for name in ast.estree_tag_fields() {
                    let field = ast
                        .fields
                        .iter()
                        .find(|field| field.name == name)
                        .unwrap_or_else(|| panic!("Unknown tag field {}.{name}", ast.name));
                    assert!(
                        field.attrs.estree_concat.is_none()
                            && field.attrs.estree_type_override.is_none()
                            && field.attrs.estree_ts_type.is_none(),
                        "Tag inputs cannot be reshaped: {}.{name}",
                        ast.name
                    );
                    if attrs.type_via.is_some() {
                        assert!(
                            matches!(&schema.types[field.type_id], AstType::Primitive(primitive) if primitive.name == "bool"),
                            "Computed tag input must be bool: {}.{name}",
                            ast.name
                        );
                    } else {
                        assert!(
                            matches!(schema.types[field.type_id], AstType::Primitive(_)),
                            "type_from requires a scalar enum field: {}.{name}",
                            ast.name
                        );
                        assert!(
                            !field.attrs.estree_ts,
                            "type_from must be available in both JS and TS modes"
                        );
                    }
                }
                if let Some(selector) = &attrs.type_via {
                    selector.inverse();
                }
                let tags = ast.estree_tags(schema);
                assert!(
                    !tags.is_empty(),
                    "Node needs at least one tag: {}",
                    ast.name
                );
                for tag in &tags {
                    validate_tag_name(tag);
                }
                if attrs.type_from.is_some() {
                    assert!(
                        !tags.contains(&attrs.name(&ast.name)),
                        "Tag conflicts with generated union alias: {}",
                        ast.name
                    );
                }
                for field in &ast.fields {
                    assert!(
                        !(field.attrs.estree_type_override.is_some()
                            && field.attrs.estree_ts_type.is_some()),
                        "Conflicting ESTree field types: {}.{}",
                        ast.name,
                        field.name
                    );
                    if let Some(tag) = &field.attrs.estree_type_override {
                        validate_tag_name(tag);
                        assert!(
                            ast.estree_emits_field(field),
                            "Tag override requires an emitted field"
                        );
                        let AstType::Struct(child) = &schema.types[field.type_id] else {
                            panic!("Tag override requires a direct child node")
                        };
                        assert!(
                            child.attrs.estree.transparent.is_none(),
                            "Cannot override a transparent value's tag"
                        );
                    }
                    if let Some(name) = &field.attrs.estree_ts_type {
                        assert!(
                            ast.estree_emits_field(field),
                            "ts_type requires an emitted field"
                        );
                        validate_narrowing(field.type_id, name, schema);
                    }
                }
            }
            AstType::Enum(ast) => {
                for variant in &ast.variants {
                    if let Some(name) = &variant.estree_ts_type {
                        validate_narrowing(
                            variant.type_id.expect("ts_type requires a node variant"),
                            name,
                            schema,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn validate_tag_name(name: &str) {
    assert!(
        !name.is_empty()
            && name
                .bytes()
                .enumerate()
                .all(|(i, c)| c.is_ascii_alphabetic()
                    || c == b'_'
                    || (i > 0 && c.is_ascii_digit())),
        "Node tag must be a TypeScript identifier: {name}"
    );
}

fn validate_narrowing(type_id: TypeId, name: &str, schema: &Schema) {
    let AstType::Struct(ast) = &schema.types[type_id] else {
        panic!("ts_type narrowing requires a direct node type")
    };
    let mut names = if ast.attrs.estree.type_from.is_some() || ast.attrs.estree.type_via.is_some() {
        ast.estree_tags(schema)
    } else {
        vec![ast.attrs.estree.name(&ast.name)]
    };
    names.extend(overridden_tags(type_id, schema));
    assert!(
        names.iter().any(|output| output == name),
        "Unknown output type {name} for {}",
        ast.name
    );
}
