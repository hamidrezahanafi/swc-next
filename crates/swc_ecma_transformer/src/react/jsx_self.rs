//! React: jsx-self — Add `__self` prop to all JSX elements.
//!
//! ```js
//! // Input:
//! <div />
//!
//! // Output:
//! <div __self={this} />
//! ```

use swc_next_ecma_ast::{Ast, JsxAttributeItem, JsxElement, JsxOpeningElement, Visit, VisitWith};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct JsxSelf;

impl JsxSelf {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        let mut collector = JsxSelfCollector {
            ast: &*ctx.ast,
            openings: Vec::new(),
        };
        program.visit_children_with(&mut collector);

        // The collector is postorder. Publishing inner openings first means
        // an outer opening cloned later observes any nested JSX in attribute
        // expressions that was already transformed.
        for opening in collector.openings {
            let scope = ctx.semantic().scope_of(opening.node_id());
            ctx.transaction(opening.node_id(), |tx| {
                let clone = tx.clone_detached(opening);
                tx.build_detached(|ast| append_self_attribute(ast, clone.root));
                tx.replace_from_transformed_clone(opening, clone, scope, &[]);
            })
            .expect("jsx-self mutation must pass preflight");
        }
    }
}

struct JsxSelfCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    openings: Vec<JsxOpeningElement>,
}

impl<'ast> Visit<'ast> for JsxSelfCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_jsx_element(&mut self, node: JsxElement) {
        node.visit_children_with(self);
        self.openings.push(node.opening_element(self.ast));
    }
}

fn append_self_attribute(ast: &mut Ast<'_>, opening: JsxOpeningElement) {
    let self_name = ast.add_utf8("__self");
    let name = ast.jsx_attribute_name_jsx_identifier(Default::default(), self_name);
    let value = ast.jsx_expression_expr_this_expression(Default::default());
    let attr_value = ast.jsx_attribute_value_jsx_expression_container(Default::default(), value);
    let self_attr =
        ast.jsx_attribute_item_jsx_attribute(Default::default(), name, Some(attr_value));

    let existing = opening.attributes(ast);
    let mut attrs: Vec<JsxAttributeItem> = (0..existing.len())
        .map(|index| existing.get_node(ast, index).unwrap())
        .collect();
    attrs.push(self_attr);
    let new_attrs = ast.add_typed_sub_range(attrs);
    opening.set_attributes(ast, new_attrs);
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, Lang, VisitWith};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use super::JsxSelfCollector;
    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn nested_attribute_jsx_is_published_postorder() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("const x = <Outer child={<Inner />} />;", &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                lang: Lang::Jsx,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.react.jsx_self = true;
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

        let mut collector = JsxSelfCollector {
            ast: &ast,
            openings: Vec::new(),
        };
        parse.program.visit_children_with(&mut collector);
        let attribute_counts: Vec<_> = collector
            .openings
            .into_iter()
            .map(|opening| opening.attributes(&ast).len())
            .collect();
        assert_eq!(attribute_counts, [1, 2]);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
