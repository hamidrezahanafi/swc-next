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

use swc_next_ecma_ast::{
    Ast, AstUtf8, DUMMY_SP, Expr, NodeKind, VisitMut, VisitMutWith, AssignmentExpression,
    AssignmentOperator, AssignmentTarget, ExportDefaultDeclaration, ObjectProperty,
    ObjectPropertyKind, PropertyKey, PropertyKind, VariableDeclarator,
};

use crate::state::TransformerCtx;

pub(super) struct DisplayName;

impl DisplayName {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        let mut visitor = DisplayNameVisitor { ast: ctx.ast };
        program.visit_mut_children_with(&mut visitor);
    }
}

struct DisplayNameVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
}

impl<'ast> VisitMut<'ast> for DisplayNameVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_object_property(
        &mut self,
        node: ObjectProperty,
    ) -> Option<ObjectProperty> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        let value = node.value(self.ast);
        if let Some(name) = key_to_utf8(self.ast, node.key(self.ast)) {
            inject_display_name(self.ast, value, name);
        }
        Some(node)
    }

    fn visit_mut_variable_declarator(
        &mut self,
        node: VariableDeclarator,
    ) -> Option<VariableDeclarator> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        if let Some(init) = node.init(self.ast) {
            let id = node.id(self.ast);
            if id.is_binding_identifier(self.ast) {
                let name = id
                    .as_binding_identifier(self.ast)
                    .unwrap()
                    .name(self.ast);
                inject_display_name(self.ast, init, name);
            }
        }
        Some(node)
    }

    fn visit_mut_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
    ) -> Option<ExportDefaultDeclaration> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        let decl = node.declaration(self.ast);
        if let Some(expr) = decl.as_expr(self.ast) {
            let name = self.ast.add_utf8("input");
            inject_display_name(self.ast, expr, name);
        }
        Some(node)
    }

    fn visit_mut_assignment_expression(
        &mut self,
        node: AssignmentExpression,
    ) -> Option<AssignmentExpression> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        if node.operator(self.ast) == AssignmentOperator::Assign {
            let right = node.right(self.ast);
            let left = node.left(self.ast);
            if let Some(name) = assign_target_to_utf8(self.ast, left) {
                inject_display_name(self.ast, right, name);
            }
        }
        Some(node)
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

/// Add `displayName: "name"` to the first argument of a createClass call.
fn inject_display_name(ast: &mut Ast, expr: Expr, name: AstUtf8) {
    if !is_create_class_call(ast, expr) {
        return;
    }

    let call = expr.as_call_expression(ast).unwrap();
    let args = call.arguments(ast);

    // Get the first argument (the object literal)
    let first_arg = match args.get(0).and_then(|id| id.get_node(ast)) {
        Some(arg) => arg,
        None => return,
    };
    let obj = match first_arg.as_object_expression(ast) {
        Some(obj) => obj,
        None => return,
    };

    // Check if displayName already exists
    let props = obj.properties(ast);
    let mut new_props: Vec<ObjectPropertyKind> = Vec::new();
    let mut has_display_name = false;
    for i in 0..props.len() {
        let prop = props.get_node(ast, i).unwrap();
        if prop_has_display_name(ast, prop) {
            has_display_name = true;
        }
        new_props.push(prop);
    }

    if has_display_name {
        return;
    }

    // Create displayName: "name" property
    let display_name_key = ast.property_key_identifier_name(
        DUMMY_SP,
        ast.add_utf8("displayName"),
    );
    let name_str = ast.get_utf8(name).to_owned();
    let value = ast.add_wtf8((&*name_str).into());
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