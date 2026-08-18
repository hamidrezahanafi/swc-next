//! ES2016: Exponentiation operator transform.
//!
//! ```js
//! a ** b → Math.pow(a, b)
//! a **= b → a = Math.pow(a, b)
//! ```

use swc_next_ecma_ast::{
    Argument, AssignmentExpression, AssignmentOperator, BinaryExpression, BinaryOperator, Expr,
    GetSpan, PropertyKey, PropertyKeyData,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct Exponentiation;

impl Exponentiation {
    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if node.operator(ctx.ast) != BinaryOperator::Exponent {
            return;
        }

        let span = node.span(ctx.ast);
        let old = Expr::BinaryExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let binary = clone.root.as_binary_expression(tx.ast()).unwrap();
            let left = binary.left(tx.ast());
            let right = binary.right(tx.ast());
            let (replacement, math) =
                tx.build_detached(|ast| create_math_pow(span, left, right, ast));
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            let name = tx.ast().canonicalize_utf8(math.name(tx.ast()));
            let symbol = tx.semantic().lookup(scope, name, ReferenceSpace::Value);
            tx.reference(
                scope,
                math.node_id(),
                name,
                ReferenceFlags::default(),
                symbol,
            );
        })
        .expect("exponentiation mutation must pass preflight");
    }

    pub fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if node.operator(ctx.ast) != AssignmentOperator::ExponentAssign {
            return;
        }

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);

        if left.is_simple_assignment_target(ctx.ast) {
            let sat = left.as_simple_assignment_target(ctx.ast).unwrap();
            if sat.is_identifier_reference(ctx.ast) {
                let ident = sat.as_identifier_reference(ctx.ast).unwrap();
                self.transform_identifier_assignment(node, ident, span, ctx);
            } else if sat.is_member_expression(ctx.ast) {
                let member = sat.as_member_expression(ctx.ast).unwrap();
                self.transform_member_assignment(node, member, span, ctx);
            }
        }
    }

    fn transform_identifier_assignment(
        &self,
        node: AssignmentExpression,
        identifier: swc_next_ecma_ast::IdentifierReference,
        span: swc_next_ecma_ast::Span,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let old = Expr::AssignmentExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        let reference = ctx.semantic().reference_of(identifier.node_id()).unwrap();
        let reference_data = ctx.semantic().reference(reference);
        let mut write_flags = ReferenceFlags::default()
            .with_space(reference_data.flags.space)
            .write_only();
        let mut read_flags = ReferenceFlags::default().with_space(reference_data.flags.space);
        if reference_data.flags.is_dynamic() {
            write_flags = write_flags.dynamic();
            read_flags = read_flags.dynamic();
        }

        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let assignment = clone.root.as_assignment_expression(tx.ast()).unwrap();
            let left = assignment.left(tx.ast());
            let right = assignment.right(tx.ast());
            let cloned_identifier = left
                .as_simple_assignment_target(tx.ast())
                .and_then(|target| target.as_identifier_reference(tx.ast()))
                .unwrap();
            let (replacement, read, math) = tx.build_detached(|ast| {
                let read = ast.identifier_reference(span, cloned_identifier.name(ast));
                let context = ast.syntax_context(cloned_identifier.node_id());
                ast.set_syntax_context(read.node_id(), context);
                let (pow, math) =
                    create_math_pow(span, Expr::IdentifierReference(read), right, ast);
                let assignment =
                    ast.assignment_expression(span, left, pow, AssignmentOperator::Assign);
                (Expr::AssignmentExpression(assignment), read, math)
            });
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            tx.update_reference_flags(reference, write_flags);
            let name = tx.ast().canonicalize_utf8(read.name(tx.ast()));
            tx.reference(
                scope,
                read.node_id(),
                name,
                read_flags,
                reference_data.symbol,
            );
            let math_name = tx.ast().canonicalize_utf8(math.name(tx.ast()));
            let math_symbol = tx
                .semantic()
                .lookup(scope, math_name, ReferenceSpace::Value);
            tx.reference(
                scope,
                math.node_id(),
                math_name,
                ReferenceFlags::default(),
                math_symbol,
            );
        })
        .expect("exponentiation assignment mutation must pass preflight");
    }

    fn transform_member_assignment(
        &self,
        node: AssignmentExpression,
        source_member: swc_next_ecma_ast::MemberExpression,
        span: swc_next_ecma_ast::Span,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let old = Expr::AssignmentExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        let source_object = source_member.object(ctx.ast);
        let source_property = match ctx.ast.property_key_data(source_member.property(ctx.ast)) {
            PropertyKeyData::Expr(expression) if source_member.computed(ctx.ast) => {
                Some(expression)
            }
            _ => None,
        };
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let assignment = clone.root.as_assignment_expression(tx.ast()).unwrap();
            let left = assignment.left(tx.ast());
            let right = assignment.right(tx.ast());
            let member = left
                .as_simple_assignment_target(tx.ast())
                .and_then(|target| target.as_member_expression(tx.ast()))
                .unwrap();
            let object_copy = tx.clone_detached(source_object);
            let property_copy = source_property.map(|property| tx.clone_detached(property));
            let property = property_copy.as_ref().map_or_else(
                || member.property(tx.ast()),
                |copy| PropertyKey::Expr(copy.root),
            );
            let computed = member.computed(tx.ast());
            let (replacement, math) = tx.build_detached(|ast| {
                let pow_left =
                    ast.expr_member_expression(span, object_copy.root, property, computed, false);
                let (pow_call, math) = create_math_pow(span, pow_left, right, ast);
                let assignment = ast.expr_assignment_expression(
                    span,
                    left,
                    pow_call,
                    AssignmentOperator::Assign,
                );
                (assignment, math)
            });
            clone.replace_root(replacement);
            let layout = tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            tx.duplicate_semantics_from_detached_clone(&object_copy, &layout);
            if let Some(property_copy) = &property_copy {
                tx.duplicate_semantics_from_detached_clone(property_copy, &layout);
            }
            let name = tx.ast().canonicalize_utf8(math.name(tx.ast()));
            let symbol = tx.semantic().lookup(scope, name, ReferenceSpace::Value);
            tx.reference(
                scope,
                math.node_id(),
                name,
                ReferenceFlags::default(),
                symbol,
            );
        })
        .expect("member exponentiation assignment mutation must pass preflight");
    }
}

fn create_math_pow(
    span: swc_next_ecma_ast::Span,
    left: Expr,
    right: Expr,
    ast: &mut swc_next_ecma_ast::Ast<'_>,
) -> (Expr, swc_next_ecma_ast::IdentifierReference) {
    let math_name = ast.add_utf8("Math");
    let pow_name = ast.add_utf8("pow");
    let math = ast.identifier_reference(span, math_name);
    let pow_prop = PropertyKey::IdentifierName(ast.identifier_name(span, pow_name));
    let callee = ast.expr_member_expression(
        span,
        Expr::IdentifierReference(math),
        pow_prop,
        false,
        false,
    );
    let args = ast.add_typed_sub_range([Argument::Expr(left), Argument::Expr(right)]);
    (
        ast.expr_call_expression(span, callee, args, false, None),
        math,
    )
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{
        AssignmentExpression, AssignmentOperator, Ast, BinaryExpression, BinaryOperator, NodeKind,
        Visit, VisitWith,
    };
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    struct Collector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        binary: Option<BinaryExpression>,
        assignment: Option<AssignmentExpression>,
    }

    impl<'ast> Visit<'ast> for Collector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_binary_expression(&mut self, node: BinaryExpression) {
            node.visit_children_with(self);
            if node.operator(self.ast) == BinaryOperator::Exponent {
                self.binary = Some(node);
            }
        }

        fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
            node.visit_children_with(self);
            if node.operator(self.ast) == AssignmentOperator::ExponentAssign {
                self.assignment = Some(node);
            }
        }
    }

    #[test]
    fn exponentiation_splits_compound_identifier_accesses() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function f(a, b) { let x = a ** b; x **= b; return x; }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let (binary, assignment) = {
            let mut collector = Collector {
                ast: &ast,
                binary: None,
                assignment: None,
            };
            parse.program.visit_children_with(&mut collector);
            (collector.binary.unwrap(), collector.assignment.unwrap())
        };
        let mut options = TransformerOptions::default();
        options.es2016.exponentiation = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let assignment_identifier = assignment
            .left(&ast)
            .as_simple_assignment_target(&ast)
            .and_then(|target| target.as_identifier_reference(&ast))
            .unwrap();
        let assignment_reference = semantic
            .reference_of(assignment_identifier.node_id())
            .unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(binary.node_id()), NodeKind::CallExpression);
        assert_eq!(assignment.operator(&ast), AssignmentOperator::Assign);
        assert!(
            semantic
                .reference(assignment_reference)
                .flags
                .is_write_only()
        );
        assert_eq!(
            semantic
                .iter_references()
                .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"Math")
                .count(),
            2
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn member_assignment_duplicates_object_and_computed_key_semantics() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("function f(obj, key, n) { obj[key] **= n; }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2016.exponentiation = true;
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

        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
