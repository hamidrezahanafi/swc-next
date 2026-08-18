use serde::Deserialize;
use swc_next_ecma_ast::{
    Ast, Class, DUMMY_SP, Expr, FormalParameterKind, Function, FunctionType, GetSpan,
    IdentifierReference, MethodDefinition, MethodDefinitionKind, Program, StaticBlock, Stmt,
    TypedSubRange, VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::{Semantic, SymbolFlags};

use crate::{
    es2015::util::fn_env_hoister::FnEnvHoister,
    mutation::{MutationTransaction, SymbolHandle},
};

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ArrowFunctionsOptions {
    pub spec: bool,
}

/// Compile ES2015 arrow functions to function expressions.
pub(super) struct ArrowFunctions {
    _options: ArrowFunctionsOptions,
}

impl ArrowFunctions {
    pub fn new(options: ArrowFunctionsOptions) -> Self {
        Self { _options: options }
    }

    pub fn transform<'a>(
        &mut self,
        program: Program,
        ast: &mut Ast<'a>,
        semantic: &mut Semantic<'a>,
    ) {
        let root_scope = semantic.scope_of(program.node_id());
        let mut tx = MutationTransaction::begin(ast, semantic, program.node_id())
            .expect("arrow lowering requires synchronized AST and Semantic epochs");
        let clone = tx.clone_detached(program);
        let manifest = tx.build_detached_with_semantic(|ast, semantic| {
            let cloned_program = clone.root;
            let mut visitor = ArrowVisitor::new(ast, semantic, &clone, root_scope);
            cloned_program.visit_mut_children_with(&mut visitor);
            let frame = visitor.hoister.take_frame();
            visitor
                .hoister
                .inject_into_program(cloned_program, frame, visitor.ast);
            visitor.hoister.into_manifest()
        });
        for &(scope, name) in &manifest.reserved_names {
            tx.reserve_generated_name_exact(scope, name);
        }
        let scope_layout = tx.replace_from_transformed_clone(program, clone, root_scope, &[]);
        let mut symbols = Vec::with_capacity(manifest.bindings.len());
        for binding in &manifest.bindings {
            symbols.push(tx.declare(
                binding.scope,
                binding.declaration,
                binding.name,
                SymbolFlags::FUNCTION_SCOPED_VAR,
            ));
        }
        for reference in &manifest.references {
            let symbol = reference
                .binding
                .map(|binding| SymbolHandle::Pending(symbols[binding]));
            tx.reference(
                scope_layout.scope_of(reference.node),
                reference.node,
                reference.name,
                reference.flags,
                symbol,
            );
        }
        for edit in &manifest.reference_edits {
            tx.rename_reference(edit.reference, edit.name);
            tx.rebind_reference(edit.reference, Some(symbols[edit.binding]));
        }
        tx.commit()
            .expect("arrow lowering mutation must pass preflight");
    }
}

struct ArrowVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    hoister: FnEnvHoister<'visit, 'ast>,
    in_subclass: bool,
    next_method_is_derived_constructor: Option<bool>,
}

impl<'visit, 'ast> ArrowVisitor<'visit, 'ast> {
    fn new(
        ast: &'visit mut Ast<'ast>,
        semantic: &'visit Semantic<'ast>,
        clone: &'visit crate::mutation::DetachedClone<Program>,
        scope: swc_next_ecma_ast::ScopeId,
    ) -> Self {
        let scope = semantic.scope(scope).hoist_target;
        let hoister = FnEnvHoister::new(semantic, clone, scope);
        Self {
            ast,
            hoister,
            in_subclass: false,
            next_method_is_derived_constructor: None,
        }
    }

    fn convert_arrow(&mut self, arrow: swc_next_ecma_ast::ArrowFunctionExpression) -> Function {
        let (span, params, body, is_async) = (
            arrow.span(self.ast),
            arrow.params(self.ast),
            arrow.body(self.ast),
            arrow.r#async(self.ast),
        );

        let old_in_arrow = self.hoister.set_in_arrow(true);
        let params = params.visit_mut_with(self).unwrap_or(params);
        let body = body.visit_mut_with(self).unwrap_or(body);
        self.hoister.set_in_arrow(old_in_arrow);

        params.set_kind(self.ast, FormalParameterKind::FormalParameters);
        let body = if let Some(body) = body.as_function_body(self.ast) {
            body
        } else {
            let expression = body
                .as_expr(self.ast)
                .expect("an arrow body is either a function body or an expression");
            let return_statement = self.ast.return_statement(DUMMY_SP, Some(expression));
            let body_items = self
                .ast
                .add_typed_sub_range([Stmt::ReturnStatement(return_statement)]);
            self.ast
                .function_body(DUMMY_SP, TypedSubRange::empty(), body_items)
        };

        self.ast.function(
            span,
            FunctionType::FunctionExpression,
            None,
            false,
            is_async,
            params,
            body,
            None,
            None,
        )
    }
}

impl<'ast> VisitMut<'ast> for ArrowVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if let Some(arrow) = node.as_arrow_function_expression(self.ast) {
            let function = self.convert_arrow(arrow);
            unsafe {
                self.ast.replace_data(arrow, function);
            }
            None
        } else if let Some(this) = node.as_this_expression(self.ast) {
            let replacement = self.hoister.transform_this_expression(this, self.ast);
            if let Some(replacement) = replacement.as_identifier_reference(self.ast) {
                unsafe {
                    self.ast.replace_data(this, replacement);
                }
            }
            None
        } else if let Some(meta) = node.as_meta_property(self.ast) {
            let replacement = self.hoister.transform_meta_property(meta, self.ast);
            if let Some(replacement) = replacement.as_identifier_reference(self.ast) {
                unsafe {
                    self.ast.replace_data(meta, replacement);
                }
            }
            None
        } else {
            node.visit_mut_children_with(self)
        }
    }

    fn visit_mut_identifier_reference(
        &mut self,
        node: IdentifierReference,
    ) -> Option<IdentifierReference> {
        self.hoister.visit_identifier_reference(node, self.ast);
        None
    }

    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        let derived_constructor = self
            .next_method_is_derived_constructor
            .take()
            .unwrap_or(false);
        let old_in_arrow = self.hoister.set_in_arrow(false);
        let scope = self.hoister.hoist_scope_of(node.body(self.ast).node_id());
        self.hoister.push_frame(scope, Some(node.node_id()));
        node.visit_mut_children_with(self);
        let frame = self.hoister.take_frame();
        self.hoister
            .inject_into_function(node, frame, derived_constructor, self.ast);
        self.hoister.set_in_arrow(old_in_arrow);
        None
    }

    fn visit_mut_class(&mut self, node: Class) -> Option<Class> {
        let old = self.in_subclass;
        self.in_subclass = node.super_class(self.ast).is_some();
        node.visit_mut_children_with(self);
        self.in_subclass = old;
        None
    }

    fn visit_mut_method_definition(&mut self, node: MethodDefinition) -> Option<MethodDefinition> {
        let decorators = node.decorators(self.ast);
        let key = node.key(self.ast);
        if let Some(decorators) = decorators.visit_mut_with(self) {
            node.set_decorators(self.ast, decorators);
        }
        if let Some(key) = key.visit_mut_with(self) {
            node.set_key(self.ast, key);
        }

        self.next_method_is_derived_constructor =
            Some(self.in_subclass && node.kind(self.ast) == MethodDefinitionKind::Constructor);
        let value = node.value(self.ast);
        if let Some(value) = value.visit_mut_with(self) {
            node.set_value(self.ast, value);
        }
        debug_assert!(self.next_method_is_derived_constructor.is_none());
        None
    }

    fn visit_mut_static_block(&mut self, node: StaticBlock) -> Option<StaticBlock> {
        let old_in_arrow = self.hoister.set_in_arrow(false);
        let scope = self.hoister.hoist_scope_of(node.node_id());
        self.hoister.push_frame(scope, None);
        node.visit_mut_children_with(self);
        let frame = self.hoister.take_frame();
        self.hoister.inject_into_static_block(node, frame, self.ast);
        self.hoister.set_in_arrow(old_in_arrow);
        None
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

    fn lower_and_assert(source: &str) {
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.arrow_functions = Some(super::ArrowFunctionsOptions::default());
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

    #[test]
    fn lowers_expression_arrow_with_consistent_semantics() {
        lower_and_assert("const arrow = value => value;");
    }

    #[test]
    fn lowers_arrow_environment_captures_with_consistent_semantics() {
        lower_and_assert(
            "function outer(value) { return () => this.value + arguments[0] + new.target; }",
        );
    }

    #[test]
    fn lowers_nested_arrows_with_consistent_semantics() {
        lower_and_assert("function outer(value) { return () => () => value + this.value; }");
    }

    #[test]
    fn lowers_derived_constructor_arrow_with_consistent_semantics() {
        lower_and_assert(
            "class Derived extends Base { constructor() { const get = () => this; super(); return get(); } }",
        );
    }
}
