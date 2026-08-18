//! Safari 10/11 bugfix for incorrectly shared tagged-template cache objects.

use swc_next_ecma_ast::{
    Ast, BindingPattern, DUMMY_SP, Expr, FormalParameterKind, FunctionType, NodeKind,
    NumericLiteralKind, Program, Stmt, TypedSubRange, VariableKind, Visit, VisitMut, VisitMutWith,
    VisitWith,
};
use swc_next_ecma_semantic::{ReferenceFlags, SymbolFlags, scope};

use crate::{
    mutation::{DetachedScopeRegion, PrivateBinding, ScopeHandle},
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct TemplateLiteralCaching;

impl TemplateLiteralCaching {
    pub fn new() -> Self {
        Self
    }

    pub fn transform<'a>(&mut self, program: Program, ctx: &mut TransformerCtx<'_, '_, 'a>) {
        let mut counter = TaggedTemplateCounter {
            ast: ctx.ast,
            count: 0,
        };
        program.visit_children_with(&mut counter);
        if counter.count == 0 {
            return;
        }

        let root_scope = ctx.semantic().scope_of(program.node_id());
        ctx.transaction(program.node_id(), |tx| {
            let clone = tx.clone_detached(program);
            let helper = tx.declare_private(root_scope, "_", SymbolFlags::FUNCTION_SCOPED_VAR);
            let caches: Vec<_> = (0..counter.count)
                .map(|_| tx.declare_private(root_scope, "_t", SymbolFlags::FUNCTION_SCOPED_VAR))
                .collect();

            let generated = tx.build_detached(|ast| build_declarations(ast, helper, &caches));
            let function_flags = scope::ScopeFlags {
                strict: tx.semantic().scope(root_scope).flags.strict,
                ..scope::ScopeFlags::default()
            };
            let function_scope = tx.create_scope(
                root_scope,
                generated.function.node_id(),
                scope::ScopeKind::Function,
                function_flags,
            );
            let parameter_name = tx
                .ast()
                .canonicalize_utf8(generated.parameter.name(tx.ast()));
            let parameter_symbol = tx.declare(
                function_scope,
                generated.parameter.node_id(),
                parameter_name,
                SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
            );

            let references = tx.build_detached(|ast| {
                let mut visitor = TemplateLiteralCachingVisitor {
                    ast,
                    helper,
                    caches: caches.iter(),
                    references: Vec::new(),
                };
                clone.root.visit_mut_children_with(&mut visitor);
                visitor.references
            });

            let body = clone.root.body(tx.ast());
            let statements = std::iter::once(generated.statement)
                .chain(body.iter().map(|id| tx.ast().get_node_in_sub_range(id)))
                .collect::<Vec<_>>();
            let statements = tx.build_detached(|ast| ast.add_typed_sub_range(statements));
            tx.build_detached(|ast| clone.root.set_body(ast, statements));

            let regions = [DetachedScopeRegion {
                root: generated.function.node_id(),
                scope: ScopeHandle::Pending(function_scope),
            }];
            let layout = tx.replace_from_transformed_clone(program, clone, root_scope, &regions);
            tx.reference(
                ScopeHandle::Pending(function_scope),
                generated.parameter_reference.node_id(),
                parameter_name,
                ReferenceFlags::default(),
                Some(parameter_symbol),
            );
            for reference in references {
                tx.reference(
                    layout.scope_of(reference.node.node_id()),
                    reference.node.node_id(),
                    tx.ast().canonicalize_utf8(reference.node.name(tx.ast())),
                    reference.flags,
                    Some(reference.binding.symbol),
                );
            }
        })
        .expect("template literal caching mutation must pass preflight");
    }
}

struct GeneratedDeclarations {
    statement: Stmt,
    function: swc_next_ecma_ast::Function,
    parameter: swc_next_ecma_ast::BindingIdentifier,
    parameter_reference: swc_next_ecma_ast::IdentifierReference,
}

fn build_declarations(
    ast: &mut Ast<'_>,
    helper: PrivateBinding,
    caches: &[PrivateBinding],
) -> GeneratedDeclarations {
    let parameter_name = ast.add_utf8("t");
    let mark = ast.fresh_mark(swc_next_ecma_ast::Mark::root());
    let context = ast.apply_mark(swc_next_ecma_ast::SyntaxContext::empty(), mark);
    let parameter_reference = ast.identifier_reference(DUMMY_SP, parameter_name);
    ast.set_syntax_context(parameter_reference.node_id(), context);
    let return_statement = ast.stmt_return_statement(
        DUMMY_SP,
        Some(Expr::IdentifierReference(parameter_reference)),
    );
    let body = ast.add_typed_sub_range([return_statement]);
    let body = ast.function_body(DUMMY_SP, TypedSubRange::empty(), body);
    let parameter_pattern = ast.formal_parameter_pattern_binding_pattern_binding_identifier(
        DUMMY_SP,
        parameter_name,
        TypedSubRange::empty(),
        None,
        false,
    );
    let parameter_node = parameter_pattern
        .as_binding_pattern(ast)
        .and_then(|pattern| pattern.as_binding_identifier(ast))
        .unwrap();
    ast.set_syntax_context(parameter_node.node_id(), context);
    let parameter_item = ast.formal_parameter_item_formal_parameter(DUMMY_SP, parameter_pattern);
    let parameters = ast.add_typed_sub_range([parameter_item]);
    let parameters = ast.formal_parameters(
        DUMMY_SP,
        parameters,
        None,
        FormalParameterKind::FormalParameters,
    );
    let function = ast.function(
        DUMMY_SP,
        FunctionType::FunctionExpression,
        None,
        false,
        false,
        parameters,
        body,
        None,
        None,
    );

    let mut declarators = Vec::with_capacity(caches.len() + 1);
    declarators.push(ast.variable_declarator(
        DUMMY_SP,
        BindingPattern::BindingIdentifier(helper.declaration),
        Some(Expr::Function(function)),
        false,
    ));
    declarators.extend(caches.iter().map(|cache| {
        ast.variable_declarator(
            DUMMY_SP,
            BindingPattern::BindingIdentifier(cache.declaration),
            None,
            false,
        )
    }));
    let declarators = ast.add_typed_sub_range(declarators);
    let declaration = ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
    GeneratedDeclarations {
        statement: Stmt::Declaration(swc_next_ecma_ast::Decl::VariableDeclaration(declaration)),
        function,
        parameter: parameter_node,
        parameter_reference,
    }
}

struct GeneratedReference {
    node: swc_next_ecma_ast::IdentifierReference,
    binding: PrivateBinding,
    flags: ReferenceFlags,
}

struct TemplateLiteralCachingVisitor<'visit, 'ast, 'bindings> {
    ast: &'visit mut Ast<'ast>,
    helper: PrivateBinding,
    caches: std::slice::Iter<'bindings, PrivateBinding>,
    references: Vec<GeneratedReference>,
}

impl TemplateLiteralCachingVisitor<'_, '_, '_> {
    fn reference(&mut self, binding: PrivateBinding, flags: ReferenceFlags) -> Expr {
        let reference = self.ast.identifier_reference(DUMMY_SP, binding.text);
        self.ast
            .set_syntax_context(reference.node_id(), binding.context);
        self.references.push(GeneratedReference {
            node: reference,
            binding,
            flags,
        });
        Expr::IdentifierReference(reference)
    }
}

impl<'ast> VisitMut<'ast> for TemplateLiteralCachingVisitor<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        if self.ast.node_kind(node.node_id()) != NodeKind::TaggedTemplateExpression {
            return Some(node);
        }

        let tagged = node.as_tagged_template_expression(self.ast).unwrap();
        let tag = tagged.tag(self.ast);
        let template = tagged.quasi(self.ast);
        let quasis = template.quasis(self.ast);
        let expressions = template.expressions(self.ast);
        let cache = *self.caches.next().expect("one cache per tagged template");

        let zeros = (0..expressions.len())
            .map(|_| {
                let raw = self.ast.add_utf8("0");
                self.ast
                    .expr_numeric_literal(DUMMY_SP, NumericLiteralKind::Decimal, raw)
            })
            .collect::<Vec<_>>();
        let zeros = self.ast.add_typed_sub_range(zeros);
        let template = self.ast.template_literal(DUMMY_SP, quasis, zeros);
        let helper = self.reference(self.helper, ReferenceFlags::default());
        let identity = self
            .ast
            .expr_tagged_template_expression(DUMMY_SP, helper, template, None);
        let read = self.reference(cache, ReferenceFlags::default());
        let target = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(DUMMY_SP, cache.text);
        let target_reference = target
            .as_simple_assignment_target(self.ast)
            .and_then(|target| target.as_identifier_reference(self.ast))
            .unwrap();
        self.ast
            .set_syntax_context(target_reference.node_id(), cache.context);
        self.references.push(GeneratedReference {
            node: target_reference,
            binding: cache,
            flags: ReferenceFlags::default().write_only(),
        });
        let assignment = self.ast.expr_assignment_expression(
            DUMMY_SP,
            target,
            identity,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        );
        let cached = self.ast.expr_logical_expression(
            DUMMY_SP,
            read,
            assignment,
            swc_next_ecma_ast::LogicalOperator::Or,
        );
        let mut arguments = vec![swc_next_ecma_ast::Argument::Expr(cached)];
        arguments.extend((0..expressions.len()).map(|index| {
            swc_next_ecma_ast::Argument::Expr(expressions.get_node(self.ast, index).unwrap())
        }));
        let arguments = self.ast.add_typed_sub_range(arguments);
        Some(
            self.ast
                .expr_call_expression(DUMMY_SP, tag, arguments, false, None),
        )
    }
}

struct TaggedTemplateCounter<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    count: usize,
}

impl<'ast> Visit<'ast> for TaggedTemplateCounter<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_expr(&mut self, node: Expr) {
        node.visit_children_with(self);
        self.count += usize::from(node.is_tagged_template_expression(self.ast));
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn caches_use_private_bindings_and_function_scope() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "let _1, _t1; function f(value) { return tag`a${value}b`; }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.bugfixes.template_literal_caching = true;
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
