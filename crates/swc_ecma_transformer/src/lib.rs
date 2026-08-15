use swc_next_ecma_ast::{
    ArrowFunctionExpression, ArrayExpression, AssignmentExpression, Ast, BinaryExpression,
    BindingProperty, BlockStatement, CallExpression, CatchClause, Class, Expr,
    LogicalExpression, MemberExpression, MetaProperty, MethodDefinition, NewExpression,
    ObjectExpression, ObjectProperty, Program, RegExpLiteral, Traverse, TraverseAction,
    TraverseCtx, TraverseWith, UnaryExpression, VariableDeclarator,
};
use swc_next_ecma_semantic::{Semantic, SemanticFeatureFlags};

use crate::{
    assumptions::Assumptions,
    state::{TransformerCtx, TransformerState},
};

pub mod assumptions;
pub mod bugfixes;
pub mod es2015;
pub mod es2016;
pub mod es2017;
pub mod es2019;
pub mod es2020;
pub mod es2021;
pub mod es2022;
pub mod es3;
mod state;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub bugfixes: bugfixes::BugfixesOptions,
    pub es3: es3::ES3Options,
    pub es2015: es2015::ES2015Options,
    pub es2016: es2016::ES2016Options,
    pub es2017: es2017::ES2017Options,
    pub es2019: es2019::ES2019Options,
    pub es2020: es2020::ES2020Options,
    pub es2021: es2021::ES2021Options,
    pub es2022: es2022::ES2022Options,
}

impl TransformerOptions {
    pub fn semantic_features(&self) -> SemanticFeatureFlags {
        let mut features = SemanticFeatureFlags::empty();
        if self.es2015.arrow_functions.is_some() {
            features |= SemanticFeatureFlags::FUNCTION_ENV_CAPTURE;
        }
        if self.es2015.generator {
            features |= SemanticFeatureFlags::YIELD
                | SemanticFeatureFlags::GENERATOR_FUNCTIONS
                | SemanticFeatureFlags::FUNCTION_ENV_CAPTURE;
        }
        if self.es2017.async_to_generator {
            features |= SemanticFeatureFlags::ASYNC_FUNCTIONS
                | SemanticFeatureFlags::AWAIT
                | SemanticFeatureFlags::FUNCTION_ENV_CAPTURE;
        }
        features
    }
}

pub struct TransformerImpl<'semantic, 'ast> {
    semantic: &'semantic mut Semantic<'ast>,
    x0_bugfixes: bugfixes::Bugfixes,
    x0_es3: es3::ES3,
    x0_es2015: es2015::ES2015,
    x0_es2016: es2016::ES2016,
    x0_es2017: es2017::ES2017,
    x0_es2019: es2019::ES2019,
    x0_es2020: es2020::ES2020,
    x0_es2021: es2021::ES2021,
    x0_es2022: es2022::ES2022,
}

impl Transformer {
    pub fn transform<'a>(
        program: Program,
        ast: &mut Ast<'a>,
        semantic: &mut Semantic<'a>,
        options: TransformerOptions,
    ) {
        let mut transformer = TransformerImpl {
            semantic,
            x0_bugfixes: bugfixes::Bugfixes::new(options.bugfixes),
            x0_es3: es3::ES3::new(options.es3),
            x0_es2015: es2015::ES2015::new(options.es2015),
            x0_es2016: es2016::ES2016::new(options.es2016),
            x0_es2017: es2017::ES2017::new(options.es2017),
            x0_es2019: es2019::ES2019::new(options.es2019),
            x0_es2020: es2020::ES2020::new(options.es2020),
            x0_es2021: es2021::ES2021::new(options.es2021),
            x0_es2022: es2022::ES2022::new(options.es2022),
        };

        let state = TransformerState {
            assumptions: options.assumptions,
        };
        let _ = program.traverse_with(&mut transformer, &mut TraverseCtx::new(ast, state));
    }
}

impl<'ast> Traverse<'ast, TransformerState> for TransformerImpl<'_, 'ast> {
    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) -> TraverseAction<Program> {
        self.x0_bugfixes.transform_program(node, ctx);
        self.x0_es2015.transform(node, self.semantic, ctx);
        self.x0_es2017.transform_program(node, ctx);
        self.x0_es2020.transform_program(node, ctx);
        if self.x0_es3.needs_node_traversal()
            || self.x0_bugfixes.needs_node_traversal()
            || self.x0_es2015.needs_node_traversal()
            || self.x0_es2016.needs_node_traversal()
            || self.x0_es2017.needs_node_traversal()
            || self.x0_es2019.needs_node_traversal()
            || self.x0_es2020.needs_node_traversal()
            || self.x0_es2021.needs_node_traversal()
            || self.x0_es2022.needs_node_traversal()
        {
            TraverseAction::Proceed
        } else {
            TraverseAction::Skip
        }
    }

    fn exit_program(
        &mut self,
        node: Program,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_program(node, ctx);
    }

    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TransformerCtx<'_, 'ast>) {
        self.x0_es3.exit_object_property(node, ctx);
        self.x0_es2015.exit_object_property(node, ctx);
    }

    fn enter_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) -> TraverseAction<ObjectExpression> {
        self.x0_es2015.enter_object_expression(node, ctx);
        TraverseAction::Proceed
    }

    fn exit_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_object_expression(node, ctx);
    }

    fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_binary_expression(node, ctx);
        self.x0_es2016.exit_binary_expression(node, ctx);
        self.x0_es2022.exit_binary_expression(node, ctx);
    }

    fn exit_logical_expression(
        &mut self,
        node: LogicalExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2020.exit_logical_expression(node, ctx);
    }

    fn exit_expression(
        &mut self,
        node: Expr,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) -> TraverseAction<Expr> {
        let action = self.x0_bugfixes.exit_expression(node, ctx);
        if !matches!(action, TraverseAction::Proceed) {
            return action;
        }
        let action = self.x0_es2015.exit_expression(node, ctx);
        if !matches!(action, TraverseAction::Proceed) {
            return action;
        }
        if let Some(replacement) = self.x0_es2020.exit_expression(node, ctx) {
            TraverseAction::Replace(replacement)
        } else {
            TraverseAction::Proceed
        }
    }

    fn enter_method_definition(
        &mut self,
        _node: MethodDefinition,
        _ctx: &mut TransformerCtx<'_, 'ast>,
    ) -> TraverseAction<MethodDefinition> {
        self.x0_bugfixes.enter_method_definition();
        TraverseAction::Proceed
    }

    fn exit_method_definition(
        &mut self,
        _node: MethodDefinition,
        _ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_bugfixes.exit_method_definition();
    }

    fn enter_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) -> TraverseAction<BinaryExpression> {
        self.x0_es2015.enter_binary_expression(node, ctx);
        TraverseAction::Proceed
    }

    fn exit_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_unary_expression(node, ctx);
    }

    fn exit_variable_declarator(
        &mut self,
        node: VariableDeclarator,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_variable_declarator(node, ctx);
    }

    fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es3.exit_assignment_expression(node, ctx);
        self.x0_es2015.exit_assignment_expression(node, ctx);
        self.x0_es2016.exit_assignment_expression(node, ctx);
        self.x0_es2020.exit_assignment_expression(node, ctx);
        self.x0_es2021.exit_assignment_expression(node, ctx);
    }

    fn exit_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es3.exit_member_expression(node, ctx);
    }

    fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_block_statement(node, ctx);
    }

    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TransformerCtx<'_, 'ast>) {
        self.x0_es2015.exit_reg_exp_literal(node, ctx);
    }

    fn exit_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_meta_property(node, ctx);
    }

    fn exit_array_expression(
        &mut self,
        node: ArrayExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_array_expression(node, ctx);
    }

    fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_call_expression(node, ctx);
    }

    fn exit_new_expression(
        &mut self,
        node: NewExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_new_expression(node, ctx);
    }

    fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) -> TraverseAction<CatchClause> {
        self.x0_es2019.enter_catch_clause(node, ctx);
        TraverseAction::Proceed
    }

    fn exit_class(&mut self, node: Class, ctx: &mut TransformerCtx<'_, 'ast>) {
        self.x0_es2022.exit_class(node, ctx);
    }

    fn enter_arrow_function_expression(
        &mut self,
        _node: ArrowFunctionExpression,
        _ctx: &mut TransformerCtx<'_, 'ast>,
    ) -> TraverseAction<ArrowFunctionExpression> {
        self.x0_bugfixes.enter_arrow_function_expression();
        TraverseAction::Proceed
    }

    fn exit_arrow_function_expression(
        &mut self,
        _node: ArrowFunctionExpression,
        _ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_bugfixes.exit_arrow_function_expression();
    }

    fn exit_binding_property(
        &mut self,
        node: BindingProperty,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_bugfixes.exit_binding_property(node, ctx);
    }
}
