use swc_next_ecma_ast::{
    Ast, ObjectProperty, Program, RegExpLiteral, Traverse, TraverseAction, TraverseCtx,
    TraverseWith,
};
use swc_next_ecma_semantic::{Semantic, SemanticFeatureFlags};

use crate::{
    assumptions::Assumptions,
    normalize::Normalizer,
    state::{TransformerCtx, TransformerState},
};

pub mod assumptions;
pub mod es2015;
pub mod es2022;
pub mod normalize;
mod state;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
    pub es2022: es2022::ES2022Options,
}

impl TransformerOptions {
    /// Semantic information that must be collected before this transform runs.
    pub fn semantic_features(&self) -> SemanticFeatureFlags {
        let mut features = SemanticFeatureFlags::empty();
        if self.es2022.static_blocks {
            features |= SemanticFeatureFlags::CLASS;
        }
        if self.es2015.arrow_functions.is_some() {
            features |= SemanticFeatureFlags::GENERATED_NAMES;
        }
        features
    }
}

pub struct TransformerImpl<'semantic, 'ast> {
    semantic: &'semantic mut Semantic<'ast>,
    x0_es2015: es2015::ES2015,
    x0_es2022: es2022::ES2022,
}

impl Transformer {
    pub fn transform<'a>(
        ast: &mut Ast<'a>,
        semantic: &mut Semantic<'a>,
        options: TransformerOptions,
    ) {
        let required_features = options.semantic_features();
        assert!(
            semantic.features().contains(required_features),
            "semantic analysis did not collect all features required by the transformer"
        );
        let program = ast.root_program();

        Normalizer::normalize(program, ast);
        let mut transformer = TransformerImpl {
            semantic,
            x0_es2015: es2015::ES2015::new(options.es2015),
            x0_es2022: es2022::ES2022::new(options.es2022),
        };

        let state = TransformerState {
            assumptions: options.assumptions,
        };
        let _ = program.traverse_with(&mut transformer, &mut TraverseCtx::new(ast, state));
    }
}

impl<'a> Traverse<'a, TransformerState> for TransformerImpl<'_, 'a> {
    fn enter_program(
        &mut self,
        node: Program,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) -> TraverseAction<Program> {
        self.x0_es2015.transform(node, self.semantic, ctx);
        TraverseAction::Proceed
    }

    fn exit_object_property(&mut self, node: ObjectProperty, ctx: &mut TransformerCtx<'_, 'a>) {
        self.x0_es2015.exit_object_property(node, ctx);
    }

    fn exit_reg_exp_literal(&mut self, node: RegExpLiteral, ctx: &mut TransformerCtx<'_, 'a>) {
        self.x0_es2015.exit_reg_exp_literal(node, ctx);
    }

    fn exit_class(&mut self, node: swc_next_ecma_ast::Class, ctx: &mut TransformerCtx<'_, 'a>) {
        self.x0_es2022.exit_class(node, self.semantic, ctx);
    }
}
