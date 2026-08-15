//! React: jsx-self — Add `__self` prop to all JSX elements.
//!
//! ```js
//! // Input:
//! <div />
//!
//! // Output:
//! <div __self={this} />
//! ```

use swc_next_ecma_ast::{Ast, AstUtf8, JsxAttributeItem, JsxElement, VisitMut, VisitMutWith};

use crate::state::TransformerCtx;

pub(super) struct JsxSelf;

impl JsxSelf {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        let self_name = ctx.ast.add_utf8("__self");
        let mut visitor = JsxSelfVisitor {
            ast: ctx.ast,
            self_name,
        };
        program.visit_mut_children_with(&mut visitor);
    }
}

struct JsxSelfVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    self_name: AstUtf8,
}

impl<'ast> VisitMut<'ast> for JsxSelfVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_jsx_element(&mut self, node: JsxElement) -> Option<JsxElement> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);

        let name = self
            .ast
            .jsx_attribute_name_jsx_identifier(Default::default(), self.self_name);
        let value = self.ast.jsx_expression_expr_this_expression(Default::default());
        let attr_value = self
            .ast
            .jsx_attribute_value_jsx_expression_container(Default::default(), value);
        let self_attr = self.ast.jsx_attribute_item_jsx_attribute(
            Default::default(),
            name,
            Some(attr_value),
        );

        let opening = node.opening_element(self.ast);
        let mut attrs: Vec<JsxAttributeItem> = Vec::new();
        let existing = opening.attributes(self.ast);
        for i in 0..existing.len() {
            attrs.push(existing.get_node(self.ast, i).unwrap());
        }
        attrs.push(self_attr);
        let new_attrs = self.ast.add_typed_sub_range(attrs);
        opening.set_attributes(self.ast, new_attrs);

        Some(node)
    }
}