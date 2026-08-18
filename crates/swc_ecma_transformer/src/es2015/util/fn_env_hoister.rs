use swc_next_ecma_ast::{
    AssignmentOperator, Ast, AstStr, Class, DUMMY_SP, Decl, Expr, Function, FunctionBody, GetSpan,
    IdentifierReference, Mark, NodeId, Program, ReferenceId, ScopeId, StaticBlock, Stmt,
    SyntaxContext, TypedSubRange, VariableKind, VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace, Semantic};

use crate::mutation::{DetachedClone, ReservedPrivateName};

#[derive(Default)]
pub struct CaptureFrame {
    this: Option<ReservedPrivateName>,
    arguments: Option<ReservedPrivateName>,
    new_target: Option<ReservedPrivateName>,
    scope: Option<ScopeId>,
    this_references: Vec<(NodeId, ReferenceFlags)>,
    argument_references: Vec<ReferenceId>,
    new_target_references: Vec<(NodeId, NodeId)>,
}

pub struct CaptureBinding {
    pub scope: ScopeId,
    pub declaration: NodeId,
    pub name: AstStr,
}

pub struct CaptureReference {
    pub node: NodeId,
    pub name: AstStr,
    pub flags: ReferenceFlags,
    pub binding: Option<usize>,
}

pub struct CaptureReferenceEdit {
    pub reference: ReferenceId,
    pub name: AstStr,
    pub binding: usize,
}

#[derive(Default)]
pub struct FnEnvMutationManifest {
    pub bindings: Vec<CaptureBinding>,
    pub references: Vec<CaptureReference>,
    pub reference_edits: Vec<CaptureReferenceEdit>,
    pub reserved_names: Vec<(ScopeId, AstStr)>,
}

impl CaptureFrame {
    fn is_empty(&self) -> bool {
        self.this.is_none() && self.arguments.is_none() && self.new_target.is_none()
    }
}

/// Hoists the lexical function environment used by converted arrow functions.
pub struct FnEnvHoister<'semantic, 'ast> {
    semantic: &'semantic Semantic<'ast>,
    clone: &'semantic DetachedClone<Program>,
    frames: Vec<CaptureFrame>,
    in_arrow: bool,
    manifest: FnEnvMutationManifest,
}

impl<'semantic, 'ast> FnEnvHoister<'semantic, 'ast> {
    pub fn new(
        semantic: &'semantic Semantic<'ast>,
        clone: &'semantic DetachedClone<Program>,
        scope: ScopeId,
    ) -> Self {
        Self {
            semantic,
            clone,
            frames: vec![CaptureFrame {
                scope: Some(scope),
                ..CaptureFrame::default()
            }],
            in_arrow: false,
            manifest: FnEnvMutationManifest::default(),
        }
    }

    pub fn into_manifest(self) -> FnEnvMutationManifest {
        self.manifest
    }

    pub fn hoist_scope_of(&self, node: NodeId) -> ScopeId {
        let node = self.clone.origin_of(node).unwrap_or(node);
        let scope = self.semantic.scope_of(node);
        self.semantic.scope(scope).hoist_target
    }

    pub fn set_in_arrow(&mut self, in_arrow: bool) -> bool {
        std::mem::replace(&mut self.in_arrow, in_arrow)
    }

    pub fn push_frame(&mut self, scope: ScopeId, _owner: Option<NodeId>) {
        self.frames.push(CaptureFrame {
            scope: Some(scope),
            ..CaptureFrame::default()
        });
    }

    pub fn take_frame(&mut self) -> CaptureFrame {
        self.frames
            .pop()
            .expect("a function environment frame exists")
    }

    fn fresh_name(&mut self, base: &str, ast: &mut Ast<'ast>) -> ReservedPrivateName {
        let scope = self
            .frames
            .last()
            .and_then(|frame| frame.scope)
            .expect("a capture scope exists");
        let base_text = ast.add_utf8(base);
        let base_name = ast.canonicalize_utf8(base_text);
        let state = self.semantic.name_allocation_state(scope, base_name);
        let locally_reserved = |name| {
            self.manifest
                .reserved_names
                .iter()
                .any(|&(reserved_scope, reserved)| reserved_scope == scope && reserved == name)
        };
        let text = if state.occupancy_count == 0 && !state.reserved && !locally_reserved(base_name)
        {
            base_text
        } else {
            let mut suffix = state.next_suffix.max(2);
            loop {
                let candidate = ast.add_utf8(&format!("{base}{suffix}"));
                let name = ast.canonicalize_utf8(candidate);
                let candidate_state = self.semantic.name_allocation_state(scope, name);
                if candidate_state.occupancy_count == 0
                    && !candidate_state.reserved
                    && !locally_reserved(name)
                {
                    break candidate;
                }
                suffix = suffix
                    .checked_add(1)
                    .expect("generated name suffix overflow");
            }
        };
        self.manifest
            .reserved_names
            .push((scope, ast.canonicalize_utf8(text)));
        let mark = ast.fresh_mark(Mark::root());
        let context = ast.apply_mark(SyntaxContext::empty(), mark);
        ReservedPrivateName { text, context }
    }

    fn capture_this(&mut self, ast: &mut Ast<'ast>) -> ReservedPrivateName {
        if let Some(name) = self.frames.last().and_then(|frame| frame.this) {
            return name;
        }
        let name = self.fresh_name("_this", ast);
        self.frames.last_mut().expect("a capture frame exists").this = Some(name);
        name
    }

    fn capture_arguments(&mut self, ast: &mut Ast<'ast>) -> ReservedPrivateName {
        if let Some(name) = self.frames.last().and_then(|frame| frame.arguments) {
            return name;
        }
        let name = self.fresh_name("_arguments", ast);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .arguments = Some(name);
        name
    }

    fn capture_new_target(&mut self, ast: &mut Ast<'ast>) -> ReservedPrivateName {
        if let Some(name) = self.frames.last().and_then(|frame| frame.new_target) {
            return name;
        }
        let name = self.fresh_name("_newtarget", ast);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .new_target = Some(name);
        name
    }

    fn is_unresolved_arguments(&self, node: IdentifierReference, ast: &Ast<'_>) -> bool {
        if ast.get_utf8(node.name(ast)) != "arguments" {
            return false;
        }
        let Some(origin) = self.clone.origin_of(node.node_id()) else {
            return false;
        };
        let Some(reference_id) = self.semantic.reference_of(origin) else {
            return false;
        };
        let reference = self.semantic.reference(reference_id);
        reference.flags.space == ReferenceSpace::Value && reference.symbol.is_none()
    }

    fn is_new_target(&self, node: swc_next_ecma_ast::MetaProperty, ast: &Ast<'_>) -> bool {
        let meta = node.meta(ast);
        let property = node.property(ast);
        ast.get_utf8(meta.name(ast)) == "new" && ast.get_utf8(property.name(ast)) == "target"
    }

    pub fn transform_this_expression(
        &mut self,
        node: swc_next_ecma_ast::ThisExpression,
        ast: &mut Ast<'ast>,
    ) -> Expr {
        if !self.in_arrow {
            return Expr::ThisExpression(node);
        }
        let name = self.capture_this(ast);
        let replacement = ast.identifier_reference(node.span(ast), name.text);
        ast.set_syntax_context(replacement.node_id(), name.context);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .this_references
            .push((node.node_id(), ReferenceFlags::default()));
        Expr::IdentifierReference(replacement)
    }

    pub fn visit_identifier_reference(&mut self, node: IdentifierReference, ast: &mut Ast<'ast>) {
        if self.in_arrow && self.is_unresolved_arguments(node, ast) {
            let origin = self
                .clone
                .origin_of(node.node_id())
                .expect("captured arguments belongs to the detached clone");
            let reference = self
                .semantic
                .reference_of(origin)
                .expect("captured arguments has a semantic reference");
            let name = self.capture_arguments(ast);
            node.set_name(ast, name.text);
            ast.set_syntax_context(node.node_id(), name.context);
            self.frames
                .last_mut()
                .expect("a capture frame exists")
                .argument_references
                .push(reference);
        }
    }

    pub fn transform_meta_property(
        &mut self,
        node: swc_next_ecma_ast::MetaProperty,
        ast: &mut Ast<'ast>,
    ) -> Expr {
        if self.in_arrow && self.is_new_target(node, ast) {
            let name = self.capture_new_target(ast);
            let replacement = ast.identifier_reference(node.span(ast), name.text);
            ast.set_syntax_context(replacement.node_id(), name.context);
            self.frames
                .last_mut()
                .expect("a capture frame exists")
                .new_target_references
                .push((node.node_id(), node.node_id()));
            Expr::IdentifierReference(replacement)
        } else {
            Expr::MetaProperty(node)
        }
    }

    fn record_capture_binding(
        &mut self,
        scope: ScopeId,
        declarator: swc_next_ecma_ast::VariableDeclarator,
        name: ReservedPrivateName,
        ast: &mut Ast<'ast>,
    ) -> usize {
        let identifier = declarator
            .id(ast)
            .as_binding_identifier(ast)
            .expect("capture binding is an identifier");
        ast.set_syntax_context(identifier.node_id(), name.context);
        let binding = self.manifest.bindings.len();
        self.manifest.bindings.push(CaptureBinding {
            scope,
            declaration: identifier.node_id(),
            name: ast.canonicalize_utf8(name.text),
        });
        binding
    }

    fn build_capture_declaration(
        &mut self,
        frame: CaptureFrame,
        derived_constructor: bool,
        ast: &mut Ast<'ast>,
    ) -> Option<swc_next_ecma_ast::VariableDeclaration> {
        if frame.is_empty() {
            return None;
        }
        let CaptureFrame {
            this,
            arguments,
            new_target,
            scope,
            this_references,
            argument_references,
            new_target_references,
        } = frame;
        let scope = scope.expect("a capture scope exists");
        let mut declarators = Vec::with_capacity(3);

        if let Some(name) = this {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name.text,
                TypedSubRange::empty(),
                None,
                false,
            );
            let init = (!derived_constructor).then(|| ast.expr_this_expression(DUMMY_SP));
            let declarator = ast.variable_declarator(DUMMY_SP, id, init, false);
            let binding = self.record_capture_binding(scope, declarator, name, ast);
            let generated_name = ast.canonicalize_utf8(name.text);
            for (reference, flags) in this_references {
                self.manifest.references.push(CaptureReference {
                    node: reference,
                    name: generated_name,
                    flags,
                    binding: Some(binding),
                });
            }
            declarators.push(declarator);
        }

        if let Some(name) = arguments {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name.text,
                TypedSubRange::empty(),
                None,
                false,
            );
            let arguments_name = ast.add_utf8("arguments");
            let init = ast.expr_identifier_reference(DUMMY_SP, arguments_name);
            let declarator = ast.variable_declarator(DUMMY_SP, id, Some(init), false);
            let binding = self.record_capture_binding(scope, declarator, name, ast);
            self.manifest.references.push(CaptureReference {
                node: init.node_id(),
                name: ast.canonicalize_utf8(arguments_name),
                flags: ReferenceFlags::default(),
                binding: None,
            });
            let generated_name = ast.canonicalize_utf8(name.text);
            for reference in argument_references {
                self.manifest.reference_edits.push(CaptureReferenceEdit {
                    reference,
                    name: generated_name,
                    binding,
                });
            }
            declarators.push(declarator);
        }

        if let Some(name) = new_target {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name.text,
                TypedSubRange::empty(),
                None,
                false,
            );
            let new_name = ast.add_utf8("new");
            let target_name = ast.add_utf8("target");
            let meta = ast.identifier_name(DUMMY_SP, new_name);
            let property = ast.identifier_name(DUMMY_SP, target_name);
            let init = ast.expr_meta_property(DUMMY_SP, meta, property);
            let declarator = ast.variable_declarator(DUMMY_SP, id, Some(init), false);
            let binding = self.record_capture_binding(scope, declarator, name, ast);
            let generated_name = ast.canonicalize_utf8(name.text);
            for (_, reference) in new_target_references {
                self.manifest.references.push(CaptureReference {
                    node: reference,
                    name: generated_name,
                    flags: ReferenceFlags::default(),
                    binding: Some(binding),
                });
            }
            declarators.push(declarator);
        }

        let declarators = ast.add_typed_sub_range(declarators);
        let declaration = ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
        Some(declaration)
    }

    pub fn inject_into_program(
        &mut self,
        program: Program,
        frame: CaptureFrame,
        ast: &mut Ast<'ast>,
    ) {
        let Some(declaration) = self.build_capture_declaration(frame, false, ast) else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let body = prepend_statement(program.body(ast), statement, ast);
        program.set_body(ast, body);
    }

    pub fn inject_into_function(
        &mut self,
        function: Function,
        mut frame: CaptureFrame,
        derived_constructor: bool,
        ast: &mut Ast<'ast>,
    ) {
        let body = function.body(ast);

        if derived_constructor && let Some(this_name) = frame.this {
            frame.this_references.extend(
                rewrite_derived_super_calls(body, this_name, ast)
                    .into_iter()
                    .map(|node| (node, ReferenceFlags::default().write_only())),
            );
        }

        let Some(declaration) = self.build_capture_declaration(frame, derived_constructor, ast)
        else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let statements = prepend_statement(body.body(ast), statement, ast);
        body.set_body(ast, statements);
    }

    pub fn inject_into_static_block(
        &mut self,
        block: StaticBlock,
        frame: CaptureFrame,
        ast: &mut Ast<'ast>,
    ) {
        let Some(declaration) = self.build_capture_declaration(frame, false, ast) else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let statements = prepend_statement(block.body(ast), statement, ast);
        block.set_body(ast, statements);
    }
}

fn prepend_statement<'ast>(
    old: TypedSubRange<Stmt>,
    statement: Stmt,
    ast: &mut Ast<'ast>,
) -> TypedSubRange<Stmt> {
    let mut items = Vec::with_capacity(old.len() + 1);
    items.push(statement);
    items.extend(old.iter().map(|id| ast.get_node_in_sub_range(id)));
    ast.add_typed_sub_range(items)
}

fn rewrite_derived_super_calls<'ast>(
    body: FunctionBody,
    this_name: ReservedPrivateName,
    ast: &mut Ast<'ast>,
) -> Vec<NodeId> {
    struct InitThis<'visit, 'ast> {
        ast: &'visit mut Ast<'ast>,
        this_name: ReservedPrivateName,
        references: Vec<NodeId>,
    }

    impl<'ast> VisitMut<'ast> for InitThis<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.ast
        }

        fn visit_mut_function(&mut self, _node: Function) -> Option<Function> {
            None
        }

        fn visit_mut_class(&mut self, _node: Class) -> Option<Class> {
            None
        }

        fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
            let replacement = node.visit_mut_children_with(self);
            let node = replacement.unwrap_or(node);
            let Some(call) = node.as_call_expression(self.ast) else {
                return replacement;
            };
            let callee = call.callee(self.ast);
            if !callee.is_super(self.ast) {
                return replacement;
            }

            let call_copy = self.ast.expr_call_expression(
                call.span(self.ast),
                callee,
                call.arguments(self.ast),
                call.optional(self.ast),
                call.type_arguments(self.ast),
            );
            let left = self
                .ast
                .assignment_target_simple_assignment_target_identifier_reference(
                    DUMMY_SP,
                    self.this_name.text,
                );
            self.ast
                .set_syntax_context(left.node_id(), self.this_name.context);
            self.references.push(left.node_id());
            let this = self.ast.expr_this_expression(DUMMY_SP);
            let assignment = self.ast.expr_assignment_expression(
                DUMMY_SP,
                left,
                this,
                AssignmentOperator::Assign,
            );
            let expressions = self.ast.add_typed_sub_range([call_copy, assignment]);
            let sequence = self
                .ast
                .sequence_expression(call.span(self.ast), expressions);
            Some(Expr::SequenceExpression(sequence))
        }
    }

    let mut visitor = InitThis {
        ast,
        this_name,
        references: Vec::new(),
    };
    body.visit_mut_with(&mut visitor);
    visitor.references
}
