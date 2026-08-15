//! ES2022: static-blocks — Transform `class A { static {} }` to IIFE.
//!
//! Input:
//! ```js
//! class C {
//!   static { foo(); }
//! }
//! ```
//!
//! Output:
//! ```js
//! class C {
//!   static #_ = (() => { foo(); })();
//! }
//! ```

use swc_next_ecma_ast::{
    Class, ClassElement, Stmt,
};

use crate::state::TransformerCtx;

pub(super) struct StaticBlocks;

impl StaticBlocks {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn exit_class(&self, node: Class, ctx: &mut TransformerCtx<'_, '_>) {
        let body = node.body(ctx.ast);
        let body_elements = body.body(ctx.ast);
        let len = body_elements.len();
        if len == 0 {
            return;
        }

        let mut has_static_block = false;
        for i in 0..len {
            let element = body_elements.get_node(ctx.ast, i).unwrap();
            if element.is_static_block(ctx.ast) {
                has_static_block = true;
                break;
            }
        }

        if !has_static_block {
            return;
        }

        // Collect existing private names to avoid conflicts
        let mut existing_names: Vec<String> = Vec::new();
        for i in 0..len {
            let element = body_elements.get_node(ctx.ast, i).unwrap();
            if element.is_property_definition(ctx.ast) {
                let prop = element.as_property_definition(ctx.ast).unwrap();
                if prop.key(ctx.ast).is_private_identifier(ctx.ast) {
                    let pi = prop.key(ctx.ast).as_private_identifier(ctx.ast).unwrap();
                    let name = ctx.ast.get_utf8(pi.name(ctx.ast)).to_owned();
                    existing_names.push(name);
                }
            }
        }

        // Generate unique private name
        let private_name = if existing_names.contains(&"_".to_string()) {
            let mut counter = 2;
            loop {
                let name = format!("_{}", counter);
                if !existing_names.contains(&name) {
                    break name;
                }
                counter += 1;
            }
        } else {
            "_".to_string()
        };

        let mut new_body: Vec<ClassElement> = Vec::new();

        for i in 0..len {
            let element = body_elements.get_node(ctx.ast, i).unwrap();
            if element.is_static_block(ctx.ast) {
                let block = element.as_static_block(ctx.ast).unwrap();
                let stmts = block.body(ctx.ast);
                let stmts_len = stmts.len();

                if stmts_len == 0 {
                    // Skip empty static blocks
                    continue;
                }

                // Build IIFE: (() => { stmts; })()
                let mut iife_stmts: Vec<Stmt> = Vec::new();
                for j in 0..stmts_len {
                    iife_stmts.push(stmts.get_node(ctx.ast, j).unwrap());
                }
                let iife_stmts_range = ctx.ast.add_typed_sub_range(iife_stmts);
                let empty_directives = ctx.ast.add_typed_sub_range(Vec::new());
                let empty_args = ctx.ast.add_typed_sub_range(Vec::new());
                let empty_params_range = ctx.ast.add_typed_sub_range(Vec::new());
                let private_name_utf8 = ctx.ast.add_utf8(&private_name);
                let empty_decorators = ctx.ast.add_typed_sub_range(Vec::new());

                let arrow_body = ctx.ast.arrow_function_body_function_body(
                    swc_next_ecma_ast::DUMMY_SP,
                    empty_directives,
                    iife_stmts_range,
                );
                let params = ctx.ast.formal_parameters(
                    swc_next_ecma_ast::DUMMY_SP,
                    empty_params_range,
                    None,
                    swc_next_ecma_ast::FormalParameterKind::FormalParameters,
                );
                let arrow = ctx.ast.expr_arrow_function_expression(
                    swc_next_ecma_ast::DUMMY_SP,
                    false,
                    false,
                    params,
                    arrow_body,
                    None,
                    None,
                );
                let call_expr = ctx.ast.expr_call_expression(
                    swc_next_ecma_ast::DUMMY_SP,
                    arrow,
                    empty_args,
                    false,
                    None,
                );

                let private_key = ctx.ast.property_key_private_identifier(
                    swc_next_ecma_ast::DUMMY_SP,
                    private_name_utf8,
                );
                let prop_def = ctx.ast.property_definition(
                    swc_next_ecma_ast::DUMMY_SP,
                    empty_decorators,
                    private_key,
                    Some(call_expr),
                    false,
                    true,  // static
                    false, // accessor
                    None,  // type_annotation
                    false, // declare
                    false, // override
                    false, // optional
                    false, // definite
                    false, // readonly
                    false, // abstract
                    swc_next_ecma_ast::Accessibility::None,
                );
                new_body.push(ClassElement::PropertyDefinition(prop_def));
            } else {
                new_body.push(element);
            }
        }

        let new_body_range = ctx.ast.add_typed_sub_range(new_body);
        body.set_body(ctx.ast, new_body_range);
    }
}