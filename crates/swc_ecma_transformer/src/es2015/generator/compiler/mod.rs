use swc_next_ecma_ast::{
    Argument, ArgumentData, AssignmentOperator, AssignmentTarget, Ast, AstNode, AstStr,
    BinaryExpression, BinaryOperator, BindingPattern, ClassElementData, DUMMY_SP, Decorator, Expr,
    ForStatementInitData, ForStatementLeftData, FormalParameterKind, Function, FunctionBody,
    FunctionType, LogicalOperator, Mark, NodeId, NodeKind, NodeKindData, NumericLiteralKind,
    ObjectPropertyKind, ObjectPropertyKindData, PropertyKey, PropertyKeyData, ReferenceId, ScopeId,
    Stmt, SymbolId, SyntaxContext, TypedSubRange, UnaryOperator, VariableKind,
};
use swc_next_ecma_semantic::{ReferenceFlags, Semantic, SubtreeEffect, SymbolFlags, scope};

use crate::{
    mutation::{
        DetachedClone, DetachedScopeCreator, DetachedScopeRegion, MutationTransaction,
        ReservedPrivateName, ScopeHandle, SymbolHandle,
    },
    state::PrivateName,
};

mod expression;
mod statement;

pub(super) fn transform<'a>(ast: &mut Ast<'a>, semantic: &mut Semantic<'a>) {
    let generators = semantic
        .generator_functions_post_order()
        .expect("generator transform requires GENERATOR_FUNCTIONS semantic feature")
        .to_vec();
    for function in generators {
        let captures = semantic
            .function_captures(function.node_id())
            .expect("generator transform requires FUNCTION_ENV_CAPTURE semantic feature");
        let arguments = captures.arguments.to_vec();
        let new_targets = captures.new_targets.to_vec();
        lower_generator(function, ast, semantic, &arguments, &new_targets)
            .expect("generator lowering mutation must pass preflight");
    }
}

fn lower_generator<'ast>(
    function: Function,
    ast: &mut Ast<'ast>,
    semantic: &mut Semantic<'ast>,
    arguments: &[ReferenceId],
    new_targets: &[NodeId],
) -> Result<(), crate::mutation::MutationError> {
    let body = function.body(ast);
    let body_scope = semantic.scope_of(body.node_id());
    let hoist_scope = semantic.scope(body_scope).hoist_target;
    let state_flags = semantic.scope(body_scope).flags;
    let declaration_retag = if function.function_type(ast) == FunctionType::FunctionDeclaration {
        function
            .id(ast)
            .and_then(|id| semantic.declaration_of(id.node_id()))
            .map(|declaration| {
                let declaration_data = semantic.declaration(declaration);
                let symbol = declaration_data.symbol;
                let flags = declaration_data.flags | SymbolFlags::ANNEX_B_FUNCTION;
                let symbol_flags = semantic.declarations(symbol).iter().fold(
                    SymbolFlags::empty(),
                    |aggregate, candidate| {
                        aggregate
                            | if candidate.node == declaration_data.node {
                                flags
                            } else {
                                candidate.flags
                            }
                    },
                );
                (declaration, symbol, flags, symbol_flags)
            })
    } else {
        None
    };
    let mut tx = MutationTransaction::begin(ast, semantic, function.node_id())?;
    let clone = tx.clone_detached(body);
    let state_name = tx.reserve_private_name(hoist_scope, "_state");
    let build = tx.build_detached_with_semantic(|ast, semantic| {
        GeneratorCompiler::new(
            ast,
            semantic,
            hoist_scope,
            state_name,
            clone,
            arguments,
            new_targets,
        )
        .lower(function)
    });
    for name in &build.reserved_names {
        tx.reserve_generated_name_exact(hoist_scope, *name);
    }
    let state_scope = tx.create_scope(
        body_scope,
        build.state_function.node_id(),
        scope::ScopeKind::Function,
        state_flags,
    );
    let state_region = DetachedScopeRegion {
        root: build.state_function.node_id(),
        scope: state_scope.into(),
    };
    let scope_creators: Vec<_> = build
        .generated_scopes
        .iter()
        .map(|scope| DetachedScopeCreator {
            owner: scope.owner,
            entries: scope.entries.clone(),
            owner_in_scope: scope.owner_in_scope,
            kind: scope.kind,
            flags: state_flags,
        })
        .collect();
    let scope_layout = tx.create_generated_scope_regions(
        build.clone.root,
        body_scope,
        &[state_region],
        &scope_creators,
    );
    tx.replace_from_transformed_clone(body, build.clone, body_scope, scope_layout.regions());
    for generated_scope in &build.generated_scopes {
        if generated_scope.owner_in_scope {
            tx.rescope_node(
                generated_scope.owner,
                scope_layout.scope_of(generated_scope.owner),
            );
        }
    }
    tx.replace(
        function.node_id(),
        build.replacement_function.node_id(),
        crate::mutation::IdentityPolicy::Preserve,
    );
    if let Some((declaration, symbol, declaration_flags, symbol_flags)) = declaration_retag {
        tx.update_declaration_flags(declaration, declaration_flags);
        tx.update_symbol_flags(symbol, symbol_flags);
    }

    let mut symbols = Vec::with_capacity(build.bindings.len());
    for binding in &build.bindings {
        let scope = match binding.scope {
            GeneratorScope::Outer => ScopeHandle::Existing(hoist_scope),
            GeneratorScope::State => ScopeHandle::Pending(state_scope),
        };
        symbols.push(
            tx.declare(
                scope,
                binding
                    .declaration
                    .expect("generated binding has a declaration"),
                binding.name,
                binding.flags,
            ),
        );
    }
    for reference in &build.references {
        let symbol = match reference.target {
            GeneratorReferenceTarget::Private(binding) => {
                Some(SymbolHandle::Pending(symbols[binding]))
            }
            GeneratorReferenceTarget::Existing(symbol) => Some(SymbolHandle::Existing(symbol)),
            GeneratorReferenceTarget::Unresolved => None,
        };
        tx.reference(
            scope_layout.scope_of(reference.node),
            reference.node,
            reference.name,
            reference.flags,
            symbol,
        );
    }
    for edit in &build.existing_reference_edits {
        tx.rename_reference(edit.reference, build.bindings[edit.binding].name);
        tx.rebind_reference(edit.reference, Some(symbols[edit.binding]));
    }
    tx.commit()?;
    Ok(())
}

#[derive(Clone, Copy)]
enum GeneratorScope {
    Outer,
    State,
}

struct GeneratorBinding {
    name: AstStr,
    declaration: Option<NodeId>,
    flags: SymbolFlags,
    scope: GeneratorScope,
    private_name: PrivateName,
}

#[derive(Clone, Copy)]
enum GeneratorReferenceTarget {
    Private(usize),
    Existing(SymbolId),
    Unresolved,
}

struct GeneratorReference {
    node: NodeId,
    name: AstStr,
    flags: ReferenceFlags,
    target: GeneratorReferenceTarget,
}

struct ExistingReferenceEdit {
    reference: ReferenceId,
    binding: usize,
}

struct GeneratedScope {
    owner: NodeId,
    entries: Vec<NodeId>,
    owner_in_scope: bool,
    kind: scope::ScopeKind,
}

struct GeneratorBuild {
    clone: DetachedClone<FunctionBody>,
    replacement_function: Function,
    state_function: Function,
    bindings: Vec<GeneratorBinding>,
    references: Vec<GeneratorReference>,
    existing_reference_edits: Vec<ExistingReferenceEdit>,
    reserved_names: Vec<AstStr>,
    generated_scopes: Vec<GeneratedScope>,
}

struct GeneratorCompiler<'semantic, 'ast> {
    ast: &'semantic mut Ast<'ast>,
    semantic: &'semantic Semantic<'ast>,
    scope: ScopeId,
    state_name: PrivateName,
    state_binding: usize,
    clone: DetachedClone<FunctionBody>,
    bindings: Vec<GeneratorBinding>,
    references: Vec<GeneratorReference>,
    existing_reference_edits: Vec<ExistingReferenceEdit>,
    reserved_names: Vec<AstStr>,
    generated_scopes: Vec<GeneratedScope>,
    cases: Vec<Vec<Stmt>>,
    current: usize,
    hoisted_vars: Vec<swc_next_ecma_ast::VariableDeclarator>,
    capture_vars: Vec<swc_next_ecma_ast::VariableDeclarator>,
    hoisted_functions: Vec<Stmt>,
    control: Vec<ControlTarget>,
    with_stack: Vec<PrivateName>,
}

#[derive(Clone)]
struct ControlTarget {
    break_label: usize,
    continue_label: Option<usize>,
    name: Option<Vec<u8>>,
}

impl<'semantic, 'ast> GeneratorCompiler<'semantic, 'ast> {
    fn new(
        ast: &'semantic mut Ast<'ast>,
        semantic: &'semantic Semantic<'ast>,
        scope: ScopeId,
        state_name: ReservedPrivateName,
        clone: DetachedClone<FunctionBody>,
        arguments: &[ReferenceId],
        new_targets: &[NodeId],
    ) -> Self {
        let state_name = PrivateName {
            text: state_name.text,
            context: state_name.context,
        };
        let state_binding = 0;
        let state_ast_name = ast.canonicalize_utf8(state_name.text);
        let mut compiler = Self {
            ast,
            semantic,
            scope,
            state_name,
            state_binding,
            clone,
            bindings: vec![GeneratorBinding {
                name: state_ast_name,
                declaration: None,
                flags: SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
                scope: GeneratorScope::State,
                private_name: state_name,
            }],
            references: Vec::new(),
            existing_reference_edits: Vec::new(),
            reserved_names: vec![state_ast_name],
            generated_scopes: Vec::new(),
            cases: vec![Vec::new()],
            current: 0,
            hoisted_vars: Vec::new(),
            capture_vars: Vec::new(),
            hoisted_functions: Vec::new(),
            control: Vec::new(),
            with_stack: Vec::new(),
        };
        compiler.prepare_captures(arguments, new_targets);
        compiler
    }

    #[inline]
    fn contains_yield<N: AstNode>(&self, node: N) -> bool {
        let Some(node) = self.clone.origin_of(node.node_id()) else {
            return false;
        };
        self.semantic
            .contains_effect(node, SubtreeEffect::Yield)
            .expect("generator transform requires the YIELD semantic feature")
    }

    fn prepare_captures(&mut self, arguments: &[ReferenceId], new_targets: &[NodeId]) {
        if !arguments.is_empty() {
            let name = self.reserve_private_name("_arguments");
            let source_name = self.ast.add_utf8("arguments");
            let initializer = self.ast.expr_identifier_reference(DUMMY_SP, source_name);
            let declaration = self.named_declarator(name, Some(initializer));
            let binding = self.record_binding(
                declaration,
                name,
                SymbolFlags::FUNCTION_SCOPED_VAR,
                GeneratorScope::Outer,
            );
            self.references.push(GeneratorReference {
                node: initializer.node_id(),
                name: self.ast.canonicalize_utf8(source_name),
                flags: ReferenceFlags::default(),
                target: GeneratorReferenceTarget::Unresolved,
            });
            for reference_id in arguments {
                let origin = self.semantic.reference(*reference_id).node;
                let reference_node = self
                    .clone
                    .clone_of(origin)
                    .expect("captured arguments reference belongs to the cloned body");
                let NodeKindData::IdentifierReference(reference) =
                    self.ast.kind_data(reference_node)
                else {
                    continue;
                };
                reference.set_name(self.ast, name.text);
                self.ast
                    .set_syntax_context(reference.node_id(), name.context);
                self.existing_reference_edits.push(ExistingReferenceEdit {
                    reference: *reference_id,
                    binding,
                });
            }
            self.capture_vars.push(declaration);
        }
        if !new_targets.is_empty() {
            let name = self.reserve_private_name("_newtarget");
            let meta_name = self.ast.add_utf8("new");
            let property_name = self.ast.add_utf8("target");
            let meta = self.ast.identifier_name(DUMMY_SP, meta_name);
            let property = self.ast.identifier_name(DUMMY_SP, property_name);
            let initializer = self.ast.expr_meta_property(DUMMY_SP, meta, property);
            let declaration = self.named_declarator(name, Some(initializer));
            let binding = self.record_binding(
                declaration,
                name,
                SymbolFlags::FUNCTION_SCOPED_VAR,
                GeneratorScope::Outer,
            );
            for meta_node in new_targets {
                let cloned = self
                    .clone
                    .clone_of(*meta_node)
                    .expect("captured new.target belongs to the cloned body");
                let NodeKindData::MetaProperty(meta) = self.ast.kind_data(cloned) else {
                    continue;
                };
                let replacement = self.ast.identifier_reference(DUMMY_SP, name.text);
                self.ast
                    .set_syntax_context(replacement.node_id(), name.context);
                unsafe {
                    self.ast.replace_data(meta, replacement);
                }
                self.references.push(GeneratorReference {
                    node: cloned,
                    name: self.ast.canonicalize_utf8(name.text),
                    flags: ReferenceFlags::default(),
                    target: GeneratorReferenceTarget::Private(binding),
                });
            }
            self.capture_vars.push(declaration);
        }
    }

    fn record_binding(
        &mut self,
        declaration: swc_next_ecma_ast::VariableDeclarator,
        name: PrivateName,
        flags: SymbolFlags,
        scope: GeneratorScope,
    ) -> usize {
        let identifier = declaration
            .id(self.ast)
            .as_binding_identifier(self.ast)
            .expect("private binding is an identifier");
        let id = self.bindings.len();
        self.bindings.push(GeneratorBinding {
            name: self.ast.canonicalize_utf8(name.text),
            declaration: Some(identifier.node_id()),
            flags,
            scope,
            private_name: name,
        });
        id
    }

    fn named_declarator(
        &mut self,
        name: PrivateName,
        initializer: Option<Expr>,
    ) -> swc_next_ecma_ast::VariableDeclarator {
        let pattern = self.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name.text,
            TypedSubRange::empty(),
            None,
            false,
        );
        let identifier = pattern
            .as_binding_identifier(self.ast)
            .expect("private binding is an identifier");
        self.ast
            .set_syntax_context(identifier.node_id(), name.context);
        self.ast
            .variable_declarator(DUMMY_SP, pattern, initializer, false)
    }

    fn lower(mut self, function: Function) -> GeneratorBuild {
        let body = self.clone.root;
        let statements = self.nodes(body.body(self.ast));
        for statement in statements {
            self.compile_stmt(statement);
        }
        if !self.case_terminated(self.current) {
            let done = self.return_op(2, None);
            self.emit(done);
        }

        let state_body = self.build_state_body();
        let state_pattern = self
            .ast
            .formal_parameter_pattern_binding_pattern_binding_identifier(
                DUMMY_SP,
                self.state_name.text,
                TypedSubRange::empty(),
                None,
                false,
            );
        let state_param = self
            .ast
            .formal_parameter_item_formal_parameter(DUMMY_SP, state_pattern);
        let state_items = self.ast.add_typed_sub_range([state_param]);
        let state_params = self.ast.formal_parameters(
            DUMMY_SP,
            state_items,
            None,
            FormalParameterKind::FormalParameters,
        );
        let state_identifier = state_pattern
            .as_binding_pattern(self.ast)
            .and_then(|pattern| pattern.as_binding_identifier(self.ast))
            .expect("state parameter is an identifier");
        self.ast
            .set_syntax_context(state_identifier.node_id(), self.state_name.context);
        self.bindings[self.state_binding].declaration = Some(state_identifier.node_id());
        let inner = self.ast.function(
            DUMMY_SP,
            FunctionType::FunctionExpression,
            None,
            false,
            false,
            state_params,
            state_body,
            None,
            None,
        );
        let helper_name = self.ast.add_utf8("_ts_generator");
        let helper = self.ast.expr_identifier_reference(DUMMY_SP, helper_name);
        self.references.push(GeneratorReference {
            node: helper.node_id(),
            name: self.ast.canonicalize_utf8(helper_name),
            flags: ReferenceFlags::default(),
            target: GeneratorReferenceTarget::Unresolved,
        });
        let this = self.ast.expr_this_expression(DUMMY_SP);
        let arguments = self
            .ast
            .add_typed_sub_range([Argument::Expr(this), Argument::Expr(Expr::Function(inner))]);
        let call = self
            .ast
            .expr_call_expression(DUMMY_SP, helper, arguments, false, None);

        let mut outer = Vec::new();
        if !self.capture_vars.is_empty() || !self.hoisted_vars.is_empty() {
            let mut variables = std::mem::take(&mut self.capture_vars);
            variables.append(&mut self.hoisted_vars);
            let declarators = self.ast.add_typed_sub_range(variables);
            outer.push(self.ast.stmt_decl_variable_declaration(
                DUMMY_SP,
                VariableKind::Var,
                declarators,
                false,
            ));
        }
        outer.append(&mut self.hoisted_functions);
        outer.push(self.ast.stmt_return_statement(DUMMY_SP, Some(call)));
        let outer = self.ast.add_typed_sub_range(outer);
        body.set_body(self.ast, outer);
        let replacement_function = self.ast.function(
            self.ast.span(function.node_id()),
            function.function_type(self.ast),
            function.id(self.ast),
            false,
            function.r#async(self.ast),
            function.params(self.ast),
            function.body(self.ast),
            function.type_parameters(self.ast),
            function.return_type(self.ast),
        );
        GeneratorBuild {
            clone: self.clone,
            replacement_function,
            state_function: inner,
            bindings: self.bindings,
            references: self.references,
            existing_reference_edits: self.existing_reference_edits,
            reserved_names: self.reserved_names,
            generated_scopes: self.generated_scopes,
        }
    }
}

impl<'semantic, 'ast> GeneratorCompiler<'semantic, 'ast> {
    fn cache(&mut self, expression: Expr) -> Expr {
        let name = self.cache_name(expression);
        self.private_reference(name)
    }

    fn cache_name(&mut self, expression: Expr) -> PrivateName {
        let name = self.new_temp();
        self.emit_assignment(name, expression);
        name
    }

    fn new_temp(&mut self) -> PrivateName {
        let name = self.reserve_private_name("_tmp");
        let pattern = self.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name.text,
            TypedSubRange::empty(),
            None,
            false,
        );
        let identifier = pattern
            .as_binding_identifier(self.ast)
            .expect("private binding is an identifier");
        self.ast
            .set_syntax_context(identifier.node_id(), name.context);
        let declarator = self.ast.variable_declarator(DUMMY_SP, pattern, None, false);
        self.record_binding(
            declarator,
            name,
            SymbolFlags::FUNCTION_SCOPED_VAR,
            GeneratorScope::Outer,
        );
        self.hoisted_vars.push(declarator);
        name
    }

    fn private_reference(&mut self, name: PrivateName) -> Expr {
        let expression = self.ast.expr_identifier_reference(DUMMY_SP, name.text);
        let identifier = expression
            .as_identifier_reference(self.ast)
            .expect("private reference is an identifier");
        self.ast
            .set_syntax_context(identifier.node_id(), name.context);
        self.record_private_reference(identifier.node_id(), name, ReferenceFlags::default());
        expression
    }

    fn emit_assignment(&mut self, name: PrivateName, expression: Expr) {
        let left = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(DUMMY_SP, name.text);
        self.ast.set_syntax_context(left.node_id(), name.context);
        self.record_private_reference(left.node_id(), name, ReferenceFlags::default().write_only());
        let assignment = self.ast.expr_assignment_expression(
            DUMMY_SP,
            left,
            expression,
            AssignmentOperator::Assign,
        );
        let statement = self.ast.stmt_expression_statement(DUMMY_SP, assignment);
        self.emit(statement);
    }

    fn reserve_private_name(&mut self, base: &str) -> PrivateName {
        let base_text = self.ast.add_utf8(base);
        let base_name = self.ast.canonicalize_utf8(base_text);
        let state = self.semantic.name_allocation_state(self.scope, base_name);
        let locally_reserved = |name| self.reserved_names.contains(&name);
        let text = if state.occupancy_count == 0 && !state.reserved && !locally_reserved(base_name)
        {
            base_text
        } else {
            let mut suffix = state.next_suffix.max(1);
            loop {
                let candidate = self.ast.add_utf8(&format!("{base}{suffix}"));
                let name = self.ast.canonicalize_utf8(candidate);
                let candidate_state = self.semantic.name_allocation_state(self.scope, name);
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
        let name = self.ast.canonicalize_utf8(text);
        self.reserved_names.push(name);
        let mark = self.ast.fresh_mark(Mark::root());
        let context = self.ast.apply_mark(SyntaxContext::empty(), mark);
        PrivateName { text, context }
    }

    fn record_private_reference(&mut self, node: NodeId, name: PrivateName, flags: ReferenceFlags) {
        let binding = self
            .bindings
            .iter()
            .position(|binding| {
                binding.private_name.context == name.context
                    && binding.private_name.text == name.text
            })
            .expect("private reference has a generated binding");
        self.references.push(GeneratorReference {
            node,
            name: self.ast.canonicalize_utf8(name.text),
            flags,
            target: GeneratorReferenceTarget::Private(binding),
        });
    }

    fn state_call(&mut self, property: &str) -> Expr {
        let object = self.private_reference(self.state_name);
        let property_name = self.ast.add_utf8(property);
        let property = self
            .ast
            .property_key_identifier_name(DUMMY_SP, property_name);
        let member = self
            .ast
            .expr_member_expression(DUMMY_SP, object, property, false, false);
        self.ast
            .expr_call_expression(DUMMY_SP, member, TypedSubRange::empty(), false, None)
    }

    fn return_op(&mut self, opcode: u32, value: Option<Expr>) -> Stmt {
        let opcode = self.number(opcode as usize);
        let mut elements = vec![Some(Argument::Expr(opcode))];
        if let Some(value) = value {
            elements.push(Some(Argument::Expr(value)));
        }
        let elements = self.ast.add_typed_sub_range(elements);
        let array = self.ast.expr_array_expression(DUMMY_SP, elements);
        self.ast.stmt_return_statement(DUMMY_SP, Some(array))
    }

    fn jump(&mut self, label: usize) -> Stmt {
        let label = self.number(label);
        self.return_op(3, Some(label))
    }

    fn emit_try_push(
        &mut self,
        start: usize,
        catch: Option<usize>,
        finally: Option<usize>,
        end: usize,
    ) {
        let elements = [
            Some(Argument::Expr(self.number(start))),
            catch.map(|label| Argument::Expr(self.number(label))),
            finally.map(|label| Argument::Expr(self.number(label))),
            Some(Argument::Expr(self.number(end))),
        ];
        let elements = self.ast.add_typed_sub_range(elements);
        let record = self.ast.expr_array_expression(DUMMY_SP, elements);

        let state = self.private_reference(self.state_name);
        let trys_name = self.ast.add_utf8("trys");
        let trys_key = self.ast.property_key_identifier_name(DUMMY_SP, trys_name);
        let trys = self
            .ast
            .expr_member_expression(DUMMY_SP, state, trys_key, false, false);
        let push_name = self.ast.add_utf8("push");
        let push_key = self.ast.property_key_identifier_name(DUMMY_SP, push_name);
        let push = self
            .ast
            .expr_member_expression(DUMMY_SP, trys, push_key, false, false);
        let arguments = self.ast.add_typed_sub_range([Argument::Expr(record)]);
        let call = self
            .ast
            .expr_call_expression(DUMMY_SP, push, arguments, false, None);
        let statement = self.ast.stmt_expression_statement(DUMMY_SP, call);
        self.emit(statement);
    }

    fn emit_conditional_jump(&mut self, test: Expr, label: usize, when_false: bool) {
        let test = if when_false {
            self.ast
                .expr_unary_expression(DUMMY_SP, test, UnaryOperator::LogicalNot)
        } else {
            test
        };
        let jump = self.jump(label);
        let statement = self.ast.stmt_if_statement(DUMMY_SP, test, jump, None);
        self.emit(statement);
    }

    fn number(&mut self, value: usize) -> Expr {
        let raw = self.ast.add_utf8(&value.to_string());
        self.ast
            .expr_numeric_literal(DUMMY_SP, NumericLiteralKind::Decimal, raw)
    }

    fn build_state_body(&mut self) -> swc_next_ecma_ast::FunctionBody {
        let body = if self.cases.len() == 1 {
            self.ast
                .add_typed_sub_range(std::mem::take(&mut self.cases[0]))
        } else {
            let object = self.private_reference(self.state_name);
            let label_name = self.ast.add_utf8("label");
            let property = self.ast.property_key_identifier_name(DUMMY_SP, label_name);
            let discriminant = self
                .ast
                .expr_member_expression(DUMMY_SP, object, property, false, false);
            let case_bodies = std::mem::take(&mut self.cases);
            let mut cases = Vec::with_capacity(case_bodies.len());
            for (label, statements) in case_bodies.into_iter().enumerate() {
                let raw = self.ast.add_utf8(&label.to_string());
                let test =
                    self.ast
                        .expr_numeric_literal(DUMMY_SP, NumericLiteralKind::Decimal, raw);
                let consequent = self.ast.add_typed_sub_range(statements);
                cases.push(self.ast.switch_case(DUMMY_SP, Some(test), consequent));
            }
            let entries = cases.iter().map(|case| case.node_id()).collect();
            let cases = self.ast.add_typed_sub_range(cases);
            let switch = self
                .ast
                .stmt_switch_statement(DUMMY_SP, discriminant, cases);
            self.generated_scopes.push(GeneratedScope {
                owner: switch.node_id(),
                entries,
                owner_in_scope: false,
                kind: scope::ScopeKind::Block,
            });
            self.ast.add_typed_sub_range([switch])
        };
        self.ast
            .function_body(DUMMY_SP, TypedSubRange::empty(), body)
    }

    fn emit(&mut self, mut statement: Stmt) {
        let names: Vec<_> = self.with_stack.iter().rev().copied().collect();
        for name in names {
            let object = self.private_reference(name);
            let entry = statement.node_id();
            statement = self.ast.stmt_with_statement(DUMMY_SP, object, statement);
            self.generated_scopes.push(GeneratedScope {
                owner: statement.node_id(),
                entries: vec![entry],
                owner_in_scope: true,
                kind: scope::ScopeKind::With,
            });
        }
        self.cases[self.current].push(statement);
    }

    fn start_new_case(&mut self) -> usize {
        let label = self.new_case();
        self.current = label;
        label
    }

    fn new_case(&mut self) -> usize {
        let label = self.cases.len();
        self.cases.push(Vec::new());
        label
    }

    fn case_terminated(&self, case: usize) -> bool {
        self.cases[case].last().is_some_and(|statement| {
            matches!(
                self.ast.node_kind(statement.node_id()),
                NodeKind::ReturnStatement | NodeKind::ThrowStatement
            )
        })
    }

    fn nodes<N: swc_next_ecma_ast::ExtraDataCompact<'ast>>(
        &self,
        range: TypedSubRange<N>,
    ) -> Vec<N> {
        range
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect()
    }
}
