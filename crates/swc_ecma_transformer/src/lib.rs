use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, BlockStatement, CatchClause, Class, ClassBody, ClassType, Decl,
    ExportDefaultDeclarationKind, Expr, ForInStatement, ForOfStatement, ForStatement,
    FormalParameters, Function, FunctionBody, NodeId, Program, StaticBlock, SwitchCase,
    TraverseAction, TraverseMut, TraverseMutCtx, TraverseMutWith, TsCallSignatureDeclaration,
    TsConstructSignatureDeclaration, TsEnumBody, TsFunction, TsIndexSignature,
    TsInterfaceDeclaration, TsMethodSignature, TsModuleBlock, TsThisParameter, TsType,
    TsTypeAliasDeclaration, TsTypeAnnotation, TsTypeParameter, TsTypeParameterDeclaration,
};
use swc_next_ecma_semantic::{AnalyzeOptions, ScopeModel, Semantic, analyze};

use crate::{
    assumptions::Assumptions,
    context::{ContainerOwner, TransformCtx, TransformerState},
};

mod context;
mod helpers;
mod jsx_pragma;
mod used_names;

pub mod assumptions;
pub mod es2022;
pub mod normalize;
pub mod proposal;
pub mod typescript;

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2022: es2022::ES2022Options,
    pub proposal: proposal::ProposalOptions,
    pub typescript: Option<typescript::TypeScriptOptions>,
}

/// Semantic indexes for the transformed AST.
pub struct TransformerReturn<'a, 'id> {
    pub semantic: ScopeModel<'a, 'id>,
}

pub struct Transformer<'id> {
    x0_es2022: es2022::ES2022<'id>,
}

impl<'id> Transformer<'id> {
    /// Run the Semantic-based TypeScript phase, then synchronized scope transforms.
    /// Normalize runtime syntax before analysis when TypeScript is enabled, otherwise normalize all syntax.
    pub fn transform<'ast>(
        ast: &mut Ast<'ast, 'id>,
        mut semantic: Semantic<'ast, 'id>,
        mut options: TransformerOptions,
    ) -> TransformerReturn<'ast, 'id> {
        if let Some(typescript) = options.typescript.take() {
            typescript::transform(ast.root_program(), ast, &semantic, typescript);
            semantic = analyze(ast, AnalyzeOptions::default()).semantic;
        }

        Self::transform_with_scopes(ast, ScopeModel::new(semantic), options)
    }

    /// Run synchronized transforms on an already normalized runtime AST and its ScopeModel.
    /// TypeScript must be disabled; use `transform` to run TypeScript lowering first.
    fn transform_with_scopes<'ast>(
        ast: &mut Ast<'ast, 'id>,
        semantic: ScopeModel<'ast, 'id>,
        options: TransformerOptions,
    ) -> TransformerReturn<'ast, 'id> {
        debug_assert!(options.typescript.is_none());
        let program = ast.root_program();

        let mut transformer = Self {
            x0_es2022: es2022::ES2022::new(options.es2022, options.proposal),
        };

        let state = TransformerState::new(semantic);
        let mut ctx = TraverseMutCtx::new(ast, state);
        let _ = program.traverse_mut_with(&mut transformer, &mut ctx);
        TransformCtx::new(&mut ctx).finish();
        TransformerReturn {
            semantic: ctx.state.semantic,
        }
    }
}

impl<'id, 'a> TraverseMut<'a, 'id, TransformerState<'a, 'id>> for Transformer<'id> {
    fn enter_program(
        &mut self,
        node: Program<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Program<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_program(
        &mut self,
        node: Program<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Program<'id>> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_helpers(node);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_function(
        &mut self,
        node: Function<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Function<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_function(
        &mut self,
        node: Function<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Function<'id>> {
        let mut transform = TransformCtx::new(ctx);
        let body = node.body(transform.ast).node_id();
        transform.inject_statements_to(body);
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ArrowFunctionExpression<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ArrowFunctionExpression<'id>> {
        let mut transform = TransformCtx::new(ctx);
        let container = node
            .body(transform.ast)
            .as_function_body(transform.ast)
            .map_or(node.node_id(), |body| body.node_id());
        transform.inject_statements_to(container);
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_function_body(
        &mut self,
        node: FunctionBody<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<FunctionBody<'id>> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_function_body(
        &mut self,
        node: FunctionBody<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<FunctionBody<'id>> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_block_statement(
        &mut self,
        node: BlockStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<BlockStatement<'id>> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<BlockStatement<'id>> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_static_block(
        &mut self,
        node: StaticBlock<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<StaticBlock<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_static_block(
        &mut self,
        node: StaticBlock<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<StaticBlock<'id>> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_module_block(
        &mut self,
        node: TsModuleBlock<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsModuleBlock<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_module_block(
        &mut self,
        node: TsModuleBlock<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsModuleBlock<'id>> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_class(
        &mut self,
        node: Class<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Class<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_class(
        &mut self,
        _node: Class<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Class<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_catch_clause(
        &mut self,
        node: CatchClause<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<CatchClause<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_catch_clause(
        &mut self,
        _node: CatchClause<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<CatchClause<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_for_statement(
        &mut self,
        node: ForStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ForStatement<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_for_statement(
        &mut self,
        _node: ForStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ForStatement<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_for_in_statement(
        &mut self,
        node: ForInStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ForInStatement<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_for_in_statement(
        &mut self,
        _node: ForInStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ForInStatement<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_for_of_statement(
        &mut self,
        node: ForOfStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ForOfStatement<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_for_of_statement(
        &mut self,
        _node: ForOfStatement<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ForOfStatement<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_function(
        &mut self,
        node: TsFunction<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsFunction<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_function(
        &mut self,
        _node: TsFunction<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsFunction<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_enum_body(
        &mut self,
        node: TsEnumBody<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsEnumBody<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_enum_body(
        &mut self,
        _node: TsEnumBody<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsEnumBody<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsInterfaceDeclaration<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_interface_declaration(
        &mut self,
        _node: TsInterfaceDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsInterfaceDeclaration<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeAliasDeclaration<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_type_alias_declaration(
        &mut self,
        _node: TsTypeAliasDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeAliasDeclaration<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_formal_parameters(
        &mut self,
        node: FormalParameters<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<FormalParameters<'id>> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_formal_parameters(
        &mut self,
        node: FormalParameters<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<FormalParameters<'id>> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_this_parameter(
        &mut self,
        node: TsThisParameter<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsThisParameter<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_this_parameter(
        &mut self,
        _node: TsThisParameter<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsThisParameter<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeParameterDeclaration<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_type_parameter_declaration(
        &mut self,
        _node: TsTypeParameterDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeParameterDeclaration<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_type_annotation(
        &mut self,
        node: TsTypeAnnotation<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeAnnotation<'id>> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_ts_type_annotation(
        &mut self,
        node: TsTypeAnnotation<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeAnnotation<'id>> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_type_parameter(
        &mut self,
        node: TsTypeParameter<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeParameter<'id>> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_ts_type_parameter(
        &mut self,
        node: TsTypeParameter<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsTypeParameter<'id>> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_type(
        &mut self,
        node: TsType<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsType<'id>> {
        if let Some(scope) = ctx.state.semantic.scope_of(node.node_id()) {
            ctx.state.scope_stack.push(scope);
        }
        TraverseAction::Continue
    }

    fn exit_ts_type(
        &mut self,
        node: TsType<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsType<'id>> {
        if ctx.state.semantic.scope_of(node.node_id()).is_some() {
            ctx.state.scope_stack.pop().unwrap();
        }
        TraverseAction::Continue
    }

    fn enter_ts_method_signature(
        &mut self,
        node: TsMethodSignature<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsMethodSignature<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_method_signature(
        &mut self,
        _node: TsMethodSignature<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsMethodSignature<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_call_signature_declaration(
        &mut self,
        node: TsCallSignatureDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsCallSignatureDeclaration<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_call_signature_declaration(
        &mut self,
        _node: TsCallSignatureDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsCallSignatureDeclaration<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_construct_signature_declaration(
        &mut self,
        node: TsConstructSignatureDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsConstructSignatureDeclaration<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_construct_signature_declaration(
        &mut self,
        _node: TsConstructSignatureDeclaration<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsConstructSignatureDeclaration<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_ts_index_signature(
        &mut self,
        node: TsIndexSignature<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsIndexSignature<'id>> {
        let scope = ctx.state.semantic.scope_of(node.node_id()).unwrap();
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_ts_index_signature(
        &mut self,
        _node: TsIndexSignature<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<TsIndexSignature<'id>> {
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_node(
        &mut self,
        node: NodeId<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) {
        ctx.state.path.push(node);
    }

    fn exit_node(
        &mut self,
        node: NodeId<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) {
        let popped = ctx.state.path.pop();
        debug_assert_eq!(popped, Some(node));
    }

    fn enter_switch_case(
        &mut self,
        _node: SwitchCase<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<SwitchCase<'id>> {
        let parent = TransformCtx::new(ctx).parent().expect("switch case parent");
        let scope = ctx.state.semantic.scope_of(parent).expect("switch scope");
        ctx.state.scope_stack.push(scope);
        TraverseAction::Continue
    }

    fn exit_switch_case(
        &mut self,
        node: SwitchCase<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<SwitchCase<'id>> {
        let mut transform = TransformCtx::new(ctx);
        transform.inject_statements_to(node.node_id());
        ctx.state.scope_stack.pop().unwrap();
        TraverseAction::Continue
    }

    fn enter_class_body(
        &mut self,
        node: ClassBody<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ClassBody<'id>> {
        let mut ctx = TransformCtx::new(ctx);
        self.x0_es2022.enter_class_body(node, &mut ctx);
        ctx.container_flush(ContainerOwner::ClassBody(node));
        TraverseAction::Continue
    }

    fn exit_class_body(
        &mut self,
        node: ClassBody<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ClassBody<'id>> {
        let mut ctx = TransformCtx::new(ctx);
        ctx.container_flush(ContainerOwner::ClassBody(node));
        TraverseAction::Continue
    }

    fn exit_decl(
        &mut self,
        node: Decl<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Decl<'id>> {
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassDeclaration
        {
            let _ = self
                .x0_es2022
                .exit_class(class, &mut TransformCtx::new(ctx));
        }
        TraverseAction::Continue
    }

    fn exit_export_default_declaration_kind(
        &mut self,
        node: ExportDefaultDeclarationKind<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<ExportDefaultDeclarationKind<'id>> {
        // Default exports dispatch Class directly rather than through Decl.
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassDeclaration
        {
            let action = self.exit_decl(Decl::Class(class), ctx);
            debug_assert!(matches!(action, TraverseAction::Continue));
        }
        TraverseAction::Continue
    }

    fn exit_expr(
        &mut self,
        node: Expr<'id>,
        ctx: &mut TraverseMutCtx<'_, 'a, 'id, TransformerState<'a, 'id>>,
    ) -> TraverseAction<Expr<'id>> {
        if let Some(class) = node.as_class(ctx.ast)
            && class.class_type(ctx.ast) == ClassType::ClassExpression
        {
            return self
                .x0_es2022
                .exit_class(class, &mut TransformCtx::new(ctx));
        }
        TraverseAction::Continue
    }
}
