use swc_next_ecma_ast::{ObjectProperty, Program, RegExpLiteral};
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

pub mod arrow;
mod generator;
pub mod shorthand;
pub mod sticky_regex;
pub(crate) mod util;

pub struct ES2015 {
    arrow: Option<arrow::ArrowFunctions>,
    generator: bool,
    shorthand_properties: bool,
    sticky_regex: bool,
}

#[derive(Default)]
pub struct ES2015Options {
    pub arrow_functions: Option<arrow::ArrowFunctionsOptions>,
    pub generator: bool,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,
}

impl ES2015 {
    pub fn new(options: ES2015Options) -> Self {
        Self {
            arrow: options.arrow_functions.map(arrow::ArrowFunctions::new),
            generator: options.generator,
            shorthand_properties: options.shorthand_properties,
            sticky_regex: options.sticky_regex,
        }
    }
}

impl ES2015 {
    pub(crate) fn transform<'a>(
        &mut self,
        node: Program,
        semantic: &Semantic,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        // Semantic-dependent generator facts must be collected from the
        // original tree. Lowering waits until after arrow conversion so
        // lexical `this`/`arguments` inside generator-local arrows keep the
        // same environment as SWC's established ES2015 pass order.
        let generator_analysis = self
            .generator
            .then(|| generator::Generator::analyze(node, ctx.ast, semantic));
        if let Some(arrow) = &mut self.arrow {
            arrow.transform(node, ctx.ast, semantic);
        }
        if let Some(generator_analysis) = generator_analysis {
            generator::Generator.transform(generator_analysis, ctx.ast, semantic);
        }
    }

    pub(crate) fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.shorthand_properties {
            shorthand::Shorthand.exit_object_property(node, ctx);
        }
    }

    pub(crate) fn exit_reg_exp_literal(
        &mut self,
        node: RegExpLiteral,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.sticky_regex {
            sticky_regex::StickyRegex.exit_reg_exp_literal(node, ctx);
        }
    }
}
