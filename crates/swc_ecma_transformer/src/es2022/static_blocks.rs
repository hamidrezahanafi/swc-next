//! ES2022 static blocks transform.
//!
//! ```js
//! class C { static { foo(); } }
//! // becomes
//! class C { static #_ = (() => { foo(); })(); }
//! ```

use swc_next_ecma_ast::{Accessibility, ClassElement, ClassId, DUMMY_SP, Stmt, TypedSubRange};
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

pub(super) struct StaticBlocks;

impl StaticBlocks {
    pub(crate) fn exit_class<'a>(
        &self,
        class: ClassId,
        semantic: &mut Semantic<'a>,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        let body = class.body(ctx.ast);
        let mut elements = body.body(ctx.ast);
        let original_len = elements.len();
        let mut write_index = 0;
        for (read_index, element_id) in elements.iter().enumerate() {
            let element = ctx.ast.get_node_in_sub_range(element_id);
            let Some(block) = element.as_static_block(ctx.ast) else {
                if write_index != read_index {
                    let output_id = elements.get(write_index).expect("write index is in range");
                    elements.replace_slot(ctx.ast, output_id, element);
                }
                write_index += 1;
                continue;
            };
            let statements = block.body(ctx.ast);
            if statements.is_empty() {
                continue;
            }

            let private_name = semantic.class_mut().next_private_name(ctx.ast, class);
            let ast = &mut *ctx.ast;
            let property = build_static_block_property(ast, statements, private_name);
            let output_id = elements.get(write_index).expect("write index is in range");
            elements.replace_slot(ast, output_id, ClassElement::PropertyDefinition(property));
            write_index += 1;
        }

        if write_index != original_len {
            elements.split_off(write_index);
            body.set_body(ctx.ast, elements);
        }
    }
}

fn build_static_block_property<'a>(
    ast: &mut swc_next_ecma_ast::Ast<'a>,
    statements: TypedSubRange<Stmt>,
    private_name: &str,
) -> swc_next_ecma_ast::PropertyDefinition {
    let arrow_body =
        ast.arrow_function_body_function_body(DUMMY_SP, TypedSubRange::empty(), statements);
    let params = ast.formal_parameters(
        DUMMY_SP,
        TypedSubRange::empty(),
        None,
        swc_next_ecma_ast::FormalParameterKind::FormalParameters,
    );
    let arrow =
        ast.expr_arrow_function_expression(DUMMY_SP, false, false, None, params, None, arrow_body);
    let call = ast.expr_call_expression(DUMMY_SP, arrow, None, TypedSubRange::empty(), false);
    let private_name = ast.add_utf8(private_name);
    let key = ast.property_key_private_identifier(DUMMY_SP, private_name);
    ast.property_definition(
        DUMMY_SP,
        TypedSubRange::empty(),
        key,
        None,
        Some(call),
        false,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        Accessibility::None,
    )
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{Transformer, TransformerOptions, es2022::ES2022Options};

    #[test]
    fn transforms_static_blocks_and_reserves_each_generated_private_name() {
        let allocator = Allocator::new();
        let parse = Parser::init(
            &allocator,
            "class C { #_() {} static {} static { foo(); } static { bar(); } }",
            Options::default(),
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut ast: Ast<'_> = parse.ast;
        let class = ast
            .root_program()
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_class(&ast))
            .unwrap();
        let original_body_start = class.body(&ast).body(&ast).start;
        let options = TransformerOptions {
            es2022: ES2022Options {
                static_blocks: true,
            },
            ..TransformerOptions::default()
        };
        let mut semantic = analyze(
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(&mut ast, &mut semantic, options);

        let elements: Vec<_> = class
            .body(&ast)
            .body(&ast)
            .iter()
            .map(|element| ast.get_node_in_sub_range(element))
            .collect();
        assert_eq!(class.body(&ast).body(&ast).start, original_body_start);
        assert_eq!(elements.len(), 3);
        assert!(
            elements
                .iter()
                .all(|element| !element.is_static_block(&ast))
        );
        let private_names: Vec<_> = elements
            .iter()
            .filter_map(|element| element.as_property_definition(&ast))
            .filter_map(|property| property.key(&ast).as_private_identifier(&ast))
            .map(|private| ast.get_utf8(private.name(&ast)))
            .collect();
        assert_eq!(private_names, ["_2", "_3"]);
        let class_info = semantic.class().get(&ast, class.node_id()).unwrap();
        assert!(class_info.has_name(b"_"));
        assert!(class_info.has_name(b"_2"));
        assert!(class_info.has_name(b"_3"));
        assert_eq!(class_info.used_names().count(), 3);
    }
}
