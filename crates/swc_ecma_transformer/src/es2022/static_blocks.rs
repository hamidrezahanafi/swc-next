//! ES2022 static blocks transform.
//!
//! ```js
//! class C { static { foo(); } }
//! // becomes
//! class C { static #_ = (() => { foo(); })(); }
//! ```

use swc_next_ecma_ast::{
    Accessibility, AstUtf8, Class, ClassElement, DUMMY_SP, Stmt, TypedSubRange,
};
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

pub(super) struct StaticBlocks;

impl StaticBlocks {
    pub(crate) fn exit_class<'a>(
        &self,
        class: Class,
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

            let ast = &mut *ctx.ast;
            let private_name = semantic.class_mut().next_private_name(ast, class.node_id());
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
    private_name: AstUtf8,
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
