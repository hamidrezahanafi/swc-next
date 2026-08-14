use swc_next_ecma_ast::{
    AssignmentOperator, Ast, AstUtf8, Class, DUMMY_SP, Decl, Expr, Function, FunctionBody, GetSpan,
    IdentifierReference, NodeId, Program, ReferenceId, ScopeId, StaticBlock, Stmt, SymbolId,
    TypedSubRange, VariableKind, VisitMut, VisitMutWith,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace, Semantic, SymbolFlags};

#[derive(Default)]
pub struct CaptureFrame {
    this: Option<AstUtf8>,
    arguments: Option<AstUtf8>,
    new_target: Option<AstUtf8>,
    scope: Option<ScopeId>,
    owner: Option<NodeId>,
    this_references: Vec<NodeId>,
    argument_references: Vec<ReferenceId>,
    new_target_references: Vec<(NodeId, NodeId)>,
}

impl CaptureFrame {
    fn is_empty(&self) -> bool {
        self.this.is_none() && self.arguments.is_none() && self.new_target.is_none()
    }
}

/// Hoists the lexical function environment used by converted arrow functions.
pub struct FnEnvHoister<'semantic, 'ast> {
    semantic: &'semantic mut Semantic<'ast>,
    frames: Vec<CaptureFrame>,
    in_arrow: bool,
}

impl<'semantic, 'ast> FnEnvHoister<'semantic, 'ast> {
    pub fn new(semantic: &'semantic mut Semantic<'ast>, scope: ScopeId) -> Self {
        Self {
            semantic,
            frames: vec![CaptureFrame {
                scope: Some(scope),
                ..CaptureFrame::default()
            }],
            in_arrow: false,
        }
    }

    pub fn hoist_scope_of(&self, node: NodeId) -> ScopeId {
        let scope = self.semantic.scope_of(node);
        self.semantic.scope(scope).hoist_target
    }

    pub fn set_in_arrow(&mut self, in_arrow: bool) -> bool {
        std::mem::replace(&mut self.in_arrow, in_arrow)
    }

    pub fn push_frame(&mut self, scope: ScopeId, owner: Option<NodeId>) {
        self.frames.push(CaptureFrame {
            scope: Some(scope),
            owner,
            ..CaptureFrame::default()
        });
    }

    pub fn take_frame(&mut self) -> CaptureFrame {
        self.frames
            .pop()
            .expect("a function environment frame exists")
    }

    fn fresh_name(&mut self, base: &str, ast: &mut Ast<'ast>) -> AstUtf8 {
        let scope = self
            .frames
            .last()
            .and_then(|frame| frame.scope)
            .expect("a capture scope exists");
        self.semantic
            .reserve_generated_name_with_min_suffix(ast, scope, base, 2)
    }

    fn capture_this(&mut self, ast: &mut Ast<'ast>) -> AstUtf8 {
        if let Some(name) = self.frames.last().and_then(|frame| frame.this) {
            return name;
        }
        let name = self.fresh_name("_this", ast);
        self.frames.last_mut().expect("a capture frame exists").this = Some(name);
        name
    }

    fn capture_arguments(&mut self, ast: &mut Ast<'ast>) -> AstUtf8 {
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

    fn capture_new_target(&mut self, ast: &mut Ast<'ast>) -> AstUtf8 {
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
        let Some(reference_id) = self.semantic.reference_of(node.node_id()) else {
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
        let replacement = ast.identifier_reference(node.span(ast), name);
        self.frames
            .last_mut()
            .expect("a capture frame exists")
            .this_references
            .push(replacement.node_id());
        Expr::IdentifierReference(replacement)
    }

    pub fn visit_identifier_reference(&mut self, node: IdentifierReference, ast: &mut Ast<'ast>) {
        if self.in_arrow && self.is_unresolved_arguments(node, ast) {
            let reference = self
                .semantic
                .reference_of(node.node_id())
                .expect("captured arguments has a semantic reference");
            let name = self.capture_arguments(ast);
            node.set_name(ast, name);
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
            let replacement = ast.identifier_reference(node.span(ast), name);
            self.frames
                .last_mut()
                .expect("a capture frame exists")
                .new_target_references
                .push((node.node_id(), replacement.node_id()));
            Expr::IdentifierReference(replacement)
        } else {
            Expr::MetaProperty(node)
        }
    }

    fn register_capture_binding(
        &mut self,
        scope: ScopeId,
        declarator: swc_next_ecma_ast::VariableDeclarator,
        name: AstUtf8,
        ast: &Ast<'ast>,
    ) -> SymbolId {
        self.semantic
            .register_node(declarator.node_id(), None, scope);
        let identifier = declarator
            .id(ast)
            .as_binding_identifier(ast)
            .expect("capture binding is an identifier");
        if identifier.node_id() != declarator.node_id() {
            self.semantic
                .register_node(identifier.node_id(), Some(declarator.node_id()), scope);
        }
        self.semantic.declare(
            ast,
            scope,
            identifier.node_id(),
            ast.canonicalize_utf8(name),
            SymbolFlags::FUNCTION_SCOPED_VAR,
        )
    }

    fn build_capture_declaration(
        &mut self,
        frame: CaptureFrame,
        derived_constructor: bool,
        parent: NodeId,
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
            owner,
            this_references,
            argument_references,
            new_target_references,
        } = frame;
        let scope = scope.expect("a capture scope exists");
        let mut declarators = Vec::with_capacity(3);

        if let Some(name) = this {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
                TypedSubRange::empty(),
                None,
                false,
            );
            let init = (!derived_constructor).then(|| ast.expr_this_expression(DUMMY_SP));
            let declarator = ast.variable_declarator(DUMMY_SP, id, init, false);
            let symbol = self.register_capture_binding(scope, declarator, name, ast);
            if let Some(init) = init {
                self.semantic
                    .register_node(init.node_id(), Some(declarator.node_id()), scope);
            }
            let generated_name = ast.canonicalize_utf8(name);
            for reference in this_references {
                self.semantic.register_node(reference, None, scope);
                self.semantic.reference_generated(
                    ast,
                    scope,
                    reference,
                    generated_name,
                    ReferenceFlags::default(),
                    Some(symbol),
                );
            }
            declarators.push(declarator);
        }

        if let Some(name) = arguments {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
                TypedSubRange::empty(),
                None,
                false,
            );
            let arguments_name = ast.add_utf8("arguments");
            let init = ast.expr_identifier_reference(DUMMY_SP, arguments_name);
            let declarator = ast.variable_declarator(DUMMY_SP, id, Some(init), false);
            let symbol = self.register_capture_binding(scope, declarator, name, ast);
            self.semantic
                .register_node(init.node_id(), Some(declarator.node_id()), scope);
            let initializer_reference = self.semantic.reference_generated(
                ast,
                scope,
                init.node_id(),
                ast.canonicalize_utf8(arguments_name),
                ReferenceFlags::default(),
                None,
            );
            let generated_name = ast.canonicalize_utf8(name);
            for reference in &argument_references {
                self.semantic
                    .rename_reference(ast, *reference, generated_name);
                self.semantic.rebind_reference(*reference, Some(symbol));
            }
            if let Some(owner) = owner {
                self.semantic.retarget_function_capture_arguments(
                    owner,
                    &argument_references,
                    initializer_reference,
                );
            }
            declarators.push(declarator);
        }

        if let Some(name) = new_target {
            let id = ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
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
            let symbol = self.register_capture_binding(scope, declarator, name, ast);
            self.semantic
                .register_node(init.node_id(), Some(declarator.node_id()), scope);
            let generated_name = ast.canonicalize_utf8(name);
            let mut old_targets = Vec::with_capacity(new_target_references.len());
            for (old, reference) in new_target_references {
                old_targets.push(old);
                self.semantic.register_node(reference, None, scope);
                self.semantic.reference_generated(
                    ast,
                    scope,
                    reference,
                    generated_name,
                    ReferenceFlags::default(),
                    Some(symbol),
                );
            }
            if let Some(owner) = owner {
                self.semantic.retarget_function_capture_new_targets(
                    owner,
                    &old_targets,
                    init.node_id(),
                );
            }
            declarators.push(declarator);
        }

        let declarator_nodes: Vec<_> = declarators.iter().map(|node| node.node_id()).collect();
        let declarators = ast.add_typed_sub_range(declarators);
        let declaration = ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
        self.semantic
            .register_node(declaration.node_id(), Some(parent), scope);
        for declarator in declarator_nodes {
            self.semantic
                .reparent_node(declarator, Some(declaration.node_id()));
        }
        Some(declaration)
    }

    pub fn inject_into_program(
        &mut self,
        program: Program,
        frame: CaptureFrame,
        ast: &mut Ast<'ast>,
    ) {
        let Some(declaration) =
            self.build_capture_declaration(frame, false, program.node_id(), ast)
        else {
            return;
        };
        let statement = Stmt::Declaration(Decl::VariableDeclaration(declaration));
        let body = prepend_statement(program.body(ast), statement, ast);
        program.set_body(ast, body);
    }

    pub fn inject_into_function(
        &mut self,
        function: Function,
        frame: CaptureFrame,
        derived_constructor: bool,
        ast: &mut Ast<'ast>,
    ) {
        let body = function.body(ast);

        if derived_constructor && let Some(this_name) = frame.this {
            rewrite_derived_super_calls(body, this_name, ast);
        }

        let Some(declaration) =
            self.build_capture_declaration(frame, derived_constructor, body.node_id(), ast)
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
        let Some(declaration) = self.build_capture_declaration(frame, false, block.node_id(), ast)
        else {
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

fn rewrite_derived_super_calls<'ast>(body: FunctionBody, this_name: AstUtf8, ast: &mut Ast<'ast>) {
    struct InitThis<'visit, 'ast> {
        ast: &'visit mut Ast<'ast>,
        this_name: AstUtf8,
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
                    self.this_name,
                );
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

    body.visit_mut_with(&mut InitThis { ast, this_name });
}
