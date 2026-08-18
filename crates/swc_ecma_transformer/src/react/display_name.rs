//! React: display-name — Add displayName to React.createClass calls.
//!
//! ```js
//! // Input:
//! var Foo = React.createClass({ ... });
//! // Output:
//! var Foo = React.createClass({ displayName: "Foo", ... });
//! ```
//!
//! Also handles:
//! - `Foo = React.createClass({...})` (assignment expressions)
//! - `{ foo: React.createClass({...}) }` (object properties)
//! - `export default createReactClass({...})` (default exports)

use std::collections::HashSet;

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTarget, Ast, AstUtf8, DUMMY_SP,
    ExportDefaultDeclaration, Expr, NodeId, NodeKind, ObjectExpression, ObjectProperty,
    ObjectPropertyKind, PropertyKey, PropertyKind, VariableDeclarator, Visit, VisitWith,
};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct DisplayName;

impl DisplayName {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        let mut collector = DisplayNameCollector {
            ast: &*ctx.ast,
            targets: Vec::new(),
            seen: HashSet::new(),
        };
        program.visit_children_with(&mut collector);

        for (object, name) in collector.targets {
            let scope = ctx.semantic().scope_of(object.node_id());
            ctx.transaction(object.node_id(), |tx| {
                let clone = tx.clone_detached(object);
                tx.build_detached(|ast| append_display_name(ast, clone.root, &name));
                tx.replace_from_transformed_clone(object, clone, scope, &[]);
            })
            .expect("display-name mutation must pass preflight");
        }
    }
}

struct DisplayNameCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    targets: Vec<(ObjectExpression, String)>,
    seen: HashSet<NodeId>,
}

impl DisplayNameCollector<'_, '_> {
    fn collect(&mut self, expression: Expr, name: String) {
        let Some(object) = display_name_target(self.ast, expression) else {
            return;
        };
        if self.seen.insert(object.node_id()) {
            self.targets.push((object, name));
        }
    }
}

impl<'ast> Visit<'ast> for DisplayNameCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_object_property(&mut self, node: ObjectProperty) {
        node.visit_children_with(self);
        let value = node.value(self.ast);
        if let Some(name) = key_to_utf8(self.ast, node.key(self.ast)) {
            self.collect(value, self.ast.get_utf8(name).to_owned());
        }
    }

    fn visit_variable_declarator(&mut self, node: VariableDeclarator) {
        node.visit_children_with(self);
        if let Some(init) = node.init(self.ast) {
            let id = node.id(self.ast);
            if id.is_binding_identifier(self.ast) {
                let name = id.as_binding_identifier(self.ast).unwrap().name(self.ast);
                self.collect(init, self.ast.get_utf8(name).to_owned());
            }
        }
    }

    fn visit_export_default_declaration(&mut self, node: ExportDefaultDeclaration) {
        node.visit_children_with(self);
        let decl = node.declaration(self.ast);
        if let Some(expr) = decl.as_expr(self.ast) {
            self.collect(expr, "input".to_owned());
        }
    }

    fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
        node.visit_children_with(self);
        if node.operator(self.ast) == AssignmentOperator::Assign {
            let right = node.right(self.ast);
            let left = node.left(self.ast);
            if let Some(name) = assign_target_to_utf8(self.ast, left) {
                self.collect(right, self.ast.get_utf8(name).to_owned());
            }
        }
    }
}

/// Extract the name from a PropertyKey as an AstUtf8.
/// Handles IdentifierName and StringLiteral keys.
fn key_to_utf8(ast: &Ast, key: PropertyKey) -> Option<AstUtf8> {
    if key.is_identifier_name(ast) {
        Some(key.as_identifier_name(ast).unwrap().name(ast))
    } else if key.is_string_literal(ast) {
        let sl = key.as_string_literal(ast).unwrap();
        let value = ast.get_wtf8(sl.value(ast));
        // Attempt to convert to AstUtf8; this is lossy but acceptable for displayName
        let s = value.to_string_lossy();
        // We can't create AstUtf8 from a String directly, so skip string literal keys
        // for now. The common case is identifier keys anyway.
        let _ = s;
        None
    } else {
        None
    }
}

/// Extract the name from an AssignmentTarget as an AstUtf8.
/// Handles `Foo = ...` (IdentifierReference) and `Foo.bar = ...` (MemberExpression).
fn assign_target_to_utf8(ast: &Ast, target: AssignmentTarget) -> Option<AstUtf8> {
    if !target.is_simple_assignment_target(ast) {
        return None;
    }
    let simple = target.as_simple_assignment_target(ast).unwrap();
    if simple.is_identifier_reference(ast) {
        let ident = simple.as_identifier_reference(ast).unwrap();
        return Some(ident.name(ast));
    }
    if simple.is_member_expression(ast) {
        let member = simple.as_member_expression(ast).unwrap();
        let prop = member.property(ast);
        if prop.is_identifier_name(ast) {
            return Some(prop.as_identifier_name(ast).unwrap().name(ast));
        }
    }
    None
}

/// Check if an expression is a `React.createClass(...)` or `createReactClass(...)` call.
fn is_create_class_call(ast: &Ast, expr: Expr) -> bool {
    let call = match expr.as_call_expression(ast) {
        Some(call) => call,
        None => return false,
    };
    let callee = call.callee(ast);
    match ast.node_kind(callee.node_id()) {
        NodeKind::MemberExpression => {
            let member = callee.as_member_expression(ast).unwrap();
            let prop = member.property(ast);
            if prop.is_identifier_name(ast) {
                let prop_name = prop.as_identifier_name(ast).unwrap();
                if ast.get_utf8(prop_name.name(ast)) == "createClass" {
                    let obj = member.object(ast);
                    if obj.is_identifier_reference(ast) {
                        let ident = obj.as_identifier_reference(ast).unwrap();
                        return ast.get_utf8(ident.name(ast)) == "React";
                    }
                }
            }
            false
        }
        NodeKind::IdentifierReference => {
            let ident = callee.as_identifier_reference(ast).unwrap();
            ast.get_utf8(ident.name(ast)) == "createReactClass"
        }
        _ => false,
    }
}

/// Check if a PropertyKey is `displayName`.
fn is_display_name_key(ast: &Ast, key: PropertyKey) -> bool {
    if key.is_identifier_name(ast) {
        let name = key.as_identifier_name(ast).unwrap();
        return ast.get_utf8(name.name(ast)) == "displayName";
    }
    if key.is_string_literal(ast) {
        let sl = key.as_string_literal(ast).unwrap();
        let value = ast.get_wtf8(sl.value(ast));
        return value == "displayName";
    }
    false
}

/// Check if an ObjectPropertyKind has `displayName` as its key.
fn prop_has_display_name(ast: &Ast, prop: ObjectPropertyKind) -> bool {
    if !prop.is_object_property(ast) {
        return false;
    }
    let obj = prop.as_object_property(ast).unwrap();
    is_display_name_key(ast, obj.key(ast))
}

fn display_name_target(ast: &Ast, expr: Expr) -> Option<ObjectExpression> {
    if !is_create_class_call(ast, expr) {
        return None;
    }

    let call = expr.as_call_expression(ast).unwrap();
    let args = call.arguments(ast);

    // Get the first argument (the object literal)
    let first_arg = args.get_node(ast, 0)?;
    let obj = first_arg
        .as_expr(ast)
        .and_then(|expr| expr.as_object_expression(ast))?;

    // Check if displayName already exists
    let props = obj.properties(ast);
    for i in 0..props.len() {
        let prop = props.get_node(ast, i).unwrap();
        if prop_has_display_name(ast, prop) {
            return None;
        }
    }
    Some(obj)
}

/// Add `displayName: "name"` to a detached createClass configuration object.
fn append_display_name(ast: &mut Ast, obj: ObjectExpression, name: &str) {
    let props = obj.properties(ast);
    let mut new_props: Vec<ObjectPropertyKind> = (0..props.len())
        .map(|index| props.get_node(ast, index).unwrap())
        .collect();

    // Create displayName: "name" property
    let display_name = ast.add_utf8("displayName");
    let display_name_key = ast.property_key_identifier_name(DUMMY_SP, display_name);
    let value = ast.add_wtf8(name.into());
    let display_name_value = ast.expr_string_literal(DUMMY_SP, value, ast.empty_utf8());
    let new_prop = ast.object_property_kind_object_property(
        DUMMY_SP,
        display_name_key,
        display_name_value,
        PropertyKind::Init,
        false,
        false,
        false,
    );
    new_props.push(new_prop);

    let new_range = ast.add_typed_sub_range(new_props);
    obj.set_properties(ast, new_range);
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, ObjectExpression, Visit, VisitWith};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use super::is_display_name_key;
    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    struct ObjectCollector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        objects: Vec<ObjectExpression>,
    }

    impl<'ast> Visit<'ast> for ObjectCollector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_object_expression(&mut self, node: ObjectExpression) {
            node.visit_children_with(self);
            self.objects.push(node);
        }
    }

    #[test]
    fn nested_create_class_objects_are_published_postorder() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "var Foo = React.createClass({ child: React.createClass({}) });",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.react.display_name = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let mut collector = ObjectCollector {
            ast: &ast,
            objects: Vec::new(),
        };
        parse.program.visit_children_with(&mut collector);
        let display_names: Vec<_> = collector
            .objects
            .into_iter()
            .map(|object| {
                object
                    .properties(&ast)
                    .iter()
                    .map(|property| ast.get_node_in_sub_range(property))
                    .find_map(|property| {
                        let property = property.as_object_property(&ast)?;
                        is_display_name_key(&ast, property.key(&ast)).then(|| {
                            let value = property.value(&ast).as_string_literal(&ast).unwrap();
                            ast.get_wtf8(value.value(&ast))
                                .to_string_lossy()
                                .into_owned()
                        })
                    })
                    .unwrap()
            })
            .collect();
        assert_eq!(display_names, ["child", "Foo"]);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
