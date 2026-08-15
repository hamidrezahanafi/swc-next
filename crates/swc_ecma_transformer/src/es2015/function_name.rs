use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, Class, Function, GetSpan, ObjectProperty, PropertyKey,
    VariableDeclarator,
};

use crate::state::TransformerCtx;

/// ES2015: function-name — Infer function/class names from binding patterns
/// and assignment targets.
///
/// ```js
/// var f = function() {}   → var f = function f() {}
/// obj.f = function() {}   → obj.f = function f() {}
/// { f: function() {} }    → { f: function f() {} }
/// ```
pub(super) struct FunctionName;

impl FunctionName {
    pub fn exit_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let init = declarator.init(ctx.ast);
        let Some(init) = init else {
            return;
        };

        let name = {
            let ast = &*ctx.ast;
            let id = declarator.id(ast);
            match id.as_binding_identifier(ast) {
                Some(id) => id.name(ast),
                None => return,
            }
        };

        let ast = &mut *ctx.ast;
        if let Some(function) = init.as_function(ast) {
            set_function_name(function, ast, name);
        } else if let Some(class) = init.as_class(ast) {
            set_class_name(class, ast, name);
        }
    }

    pub fn exit_assignment_expression(
        &mut self,
        expr: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if expr.operator(ctx.ast) != AssignmentOperator::Assign {
            return;
        }

        let (left, right) = {
            let ast = &*ctx.ast;
            (expr.left(ast), expr.right(ast))
        };

        let name = {
            let ast = &*ctx.ast;
            left.as_simple_assignment_target(ast)
                .and_then(|simple| simple.as_identifier_reference(ast))
                .map(|ident| ident.name(ast))
        };
        let Some(name) = name else {
            return;
        };

        let ast = &mut *ctx.ast;
        if let Some(function) = right.as_function(ast) {
            set_function_name(function, ast, name);
        } else if let Some(class) = right.as_class(ast) {
            set_class_name(class, ast, name);
        }
    }

    pub fn exit_object_property(
        &mut self,
        property: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let (value, property_key) = {
            let ast = &*ctx.ast;
            (property.value(ast), property.key(ast))
        };

        let name = property_key_name(property_key, ctx.ast);
        let Some(name) = name else {
            return;
        };

        let ast = &mut *ctx.ast;
        if let Some(function) = value.as_function(ast) {
            set_function_name(function, ast, name);
        } else if let Some(class) = value.as_class(ast) {
            set_class_name(class, ast, name);
        }
    }
}

fn set_function_name(function: Function, ast: &mut swc_next_ecma_ast::Ast, name: AstUtf8) {
    if function.id(ast).is_some() {
        return;
    }
    let binding = ast.binding_identifier(
        function.span(ast),
        name,
        swc_next_ecma_ast::TypedSubRange::empty(),
        None,
        false,
    );
    function.set_id(ast, Some(binding));
}

fn set_class_name(class: Class, ast: &mut swc_next_ecma_ast::Ast, name: AstUtf8) {
    if class.id(ast).is_some() {
        return;
    }
    let binding = ast.binding_identifier(
        class.span(ast),
        name,
        swc_next_ecma_ast::TypedSubRange::empty(),
        None,
        false,
    );
    class.set_id(ast, Some(binding));
}

type AstUtf8 = swc_next_ecma_ast::AstUtf8;

fn property_key_name(key: PropertyKey, ast: &swc_next_ecma_ast::Ast) -> Option<AstUtf8> {
    if let Some(ident) = key.as_identifier_name(ast) {
        return Some(ident.name(ast));
    }
    None
}