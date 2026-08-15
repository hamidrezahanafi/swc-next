use swc_next_ecma_ast::{
    AssignmentExpression, Ast, ObjectExpression, ObjectProperty, Program, RegExpLiteral, Traverse,
    TraverseAction, TraverseCtx, TraverseWith, VariableDeclarator,
};
use swc_next_ecma_semantic::{Semantic, SemanticFeatureFlags};

use crate::{
    assumptions::Assumptions,
    state::{TransformerCtx, TransformerState},
};

pub mod assumptions;
pub mod es2015;
mod state;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
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
        features
    }
}

pub struct TransformerImpl<'semantic, 'ast> {
    semantic: &'semantic mut Semantic<'ast>,
    x0_es2015: es2015::ES2015,
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
            x0_es2015: es2015::ES2015::new(options.es2015),
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
        self.x0_es2015.transform(node, self.semantic, ctx);
        if self.x0_es2015.needs_node_traversal() {
            TraverseAction::Proceed
        } else {
            TraverseAction::Skip
        }
    }

    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TransformerCtx<'_, 'ast>) {
        self.x0_es2015.exit_object_property(node, ctx);
    }

    fn exit_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TransformerCtx<'_, 'ast>,
    ) {
        self.x0_es2015.exit_object_expression(node, ctx);
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
        self.x0_es2015.exit_assignment_expression(node, ctx);
    }

    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TransformerCtx<'_, 'ast>) {
        self.x0_es2015.exit_reg_exp_literal(node, ctx);
    }
}
