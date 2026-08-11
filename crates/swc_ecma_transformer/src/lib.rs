use swc_next_ecma_ast::{
    Ast, CatchClause, ObjectProperty, Program, RegExpLiteral, Traverse, TraverseAction,
    TraverseCtx, TraverseWith,
};
use swc_next_ecma_semantic::Semantic;

use crate::{
    assumptions::Assumptions,
    state::{TransformerCtx, TransformerState},
};

pub mod assumptions;
pub mod es2015;
pub mod es2019;
mod state;

pub struct Transformer {}

#[derive(Default)]
pub struct TransformerOptions {
    pub assumptions: Assumptions,
    pub es2015: es2015::ES2015Options,
    pub es2019: es2019::ES2019Options,
}

pub struct TransformerImpl<'a> {
    semantic: &'a Semantic<'a>,
    x0_es2015: es2015::ES2015,
    x1_es2019: es2019::ES2019,
}

impl Transformer {
    pub fn transform<'a>(
        program: Program,
        ast: &mut Ast<'a>,
        semantic: &Semantic,
        options: TransformerOptions,
    ) {
        let mut transformer = TransformerImpl {
            semantic,
            x0_es2015: es2015::ES2015::new(options.es2015),
            x1_es2019: es2019::ES2019::new(options.es2019),
        };

        let state = TransformerState {
            assumptions: options.assumptions,
        };
        let _ = program.traverse_with(&mut transformer, &mut TraverseCtx::new(ast, state));
    }
}

impl<'a> Traverse<'a, TransformerState> for TransformerImpl<'_> {
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

    fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) -> TraverseAction<CatchClause> {
        self.x1_es2019.enter_catch_clause(node, ctx);
        TraverseAction::Proceed
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_codegen::{Options as CodegenOptions, print};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use super::{Transformer, TransformerOptions, es2019::ES2019Options};

    #[test]
    fn transforms_optional_catch_binding() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("try {} catch {}", &allocator);
        let program = Parser::init(&mut ast, ParserOptions::default(), NoTokenParserConfig)
            .parse()
            .program;
        let semantic = analyze(program, &ast, AnalyzeOptions { check_syntax: true }).semantic;
        let options = TransformerOptions {
            es2019: ES2019Options {
                optional_catch_binding: true,
            },
            ..TransformerOptions::default()
        };

        Transformer::transform(program, &mut ast, &semantic, options);

        let output = print(&allocator, &ast, program, CodegenOptions::default());
        assert_eq!(output.code, "try {\n} catch (unused) {\n}\n");
    }
}
