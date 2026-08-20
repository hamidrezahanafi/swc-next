use std::collections::{HashMap, HashSet};

use swc_next_ecma_ast::{
    Argument, ArrowFunctionBody, AssignmentOperator, AssignmentTarget, Ast, AstUtf8,
    BinaryOperator, BindingPattern, BlockStatement, Decl, ExportDefaultDeclarationKind, Expr,
    FormalParameterKind, Function, FunctionBody, FunctionType, IdentifierReference, IfStatement,
    LogicalOperator, MethodDefinitionKind, NodeId, NodeKindData, ObjectPropertyKind, Program,
    PropertyKind, SimpleAssignmentTarget, Stmt, SyntaxContext, TypedSubRange, UnaryOperator,
    VariableKind,
};
use swc_next_ecma_semantic::{
    ReferenceFlags, SymbolFlags,
    scope::{ScopeFlags, ScopeKind},
};

use crate::{
    mutation::{
        IdentityPolicy, MutationError, PrivateBinding, ReservedPrivateName, ScopeHandle,
        StatementListOwner, SymbolHandle,
    },
    state::{TransformerCtx, TransformerCtxExt},
};

use super::plan::{
    AnonymousClassExpressionPlan, AnonymousDefaultClassPlan, ClassExport, ClassExpressionPlan,
    ClassLoweringPlan, ConstructorPlan, MethodGroupPlan, MethodKeyPlan, MethodPlan,
};
use super::{helpers, helpers::ClassHelpers};

pub(super) fn lower_program<'ast>(
    program: Program,
    plans: Vec<ClassLoweringPlan>,
    expression_plans: Vec<ClassExpressionPlan>,
    anonymous_expression_plans: Vec<AnonymousClassExpressionPlan>,
    anonymous_default_plans: Vec<AnonymousDefaultClassPlan>,
    ctx: &mut TransformerCtx<'_, '_, 'ast>,
) -> Result<(), MutationError> {
    let mut expression_plans = expression_plans;
    expression_plans.sort_by_key(|plan| {
        ctx.state
            .semantic
            .ancestors(Some(plan.class.node_id()))
            .count()
    });
    let mut anonymous_expression_plans = anonymous_expression_plans;
    anonymous_expression_plans.sort_by_key(|plan| {
        ctx.state
            .semantic
            .ancestors(Some(plan.class.node_id()))
            .count()
    });
    let body = program.body(ctx.ast);
    let statements: Vec<_> = (0..body.len())
        .map(|index| body.get_node(ctx.ast, index).unwrap())
        .collect();
    let mut nested_owners = Vec::new();
    let needs_super_get = plans.iter().any(|plan| !plan.super_members.is_empty())
        || expression_plans
            .iter()
            .any(|plan| !plan.super_members.is_empty())
        || anonymous_expression_plans
            .iter()
            .any(|plan| !plan.super_members.is_empty())
        || anonymous_default_plans
            .iter()
            .any(|plan| !plan.super_members.is_empty());
    let needs_super_set = plans.iter().any(|plan| plan.has_super_writes)
        || expression_plans.iter().any(|plan| plan.has_super_writes)
        || anonymous_expression_plans
            .iter()
            .any(|plan| plan.has_super_writes)
        || anonymous_default_plans
            .iter()
            .any(|plan| plan.has_super_writes);
    let needs_class_name_tdz = expression_plans
        .iter()
        .any(|plan| !plan.definition_tdz_self_references.is_empty());
    let needs_class_helpers = needs_super_get
        || needs_class_name_tdz
        || plans.iter().any(|plan| plan.heritage.is_some())
        || expression_plans.iter().any(|plan| plan.heritage.is_some())
        || anonymous_expression_plans
            .iter()
            .any(|plan| plan.heritage.is_some())
        || anonymous_default_plans
            .iter()
            .any(|plan| plan.heritage.is_some());
    for plan in &plans {
        if !matches!(plan.owner, StatementListOwner::Program(_))
            && !nested_owners.contains(&plan.owner)
        {
            nested_owners.push(plan.owner);
        }
    }

    let mut evaluation_scopes = HashMap::new();
    for (class_scope, outer_scope) in plans
        .iter()
        .map(|plan| (plan.class_scope, plan.outer_scope))
        .chain(
            expression_plans
                .iter()
                .map(|plan| (plan.class_scope, plan.outer_scope)),
        )
        .chain(
            anonymous_expression_plans
                .iter()
                .map(|plan| (plan.class_scope, plan.outer_scope)),
        )
        .chain(
            anonymous_default_plans
                .iter()
                .map(|plan| (plan.class_scope, plan.outer_scope)),
        )
    {
        evaluation_scopes.insert(class_scope, ScopeHandle::Existing(outer_scope));
    }

    ctx.transaction(program.node_id(), move |tx| {
        let published_helpers = needs_class_helpers.then(|| {
            helpers::publish(
                program,
                needs_super_get,
                needs_super_set,
                needs_class_name_tdz,
                tx.inner(),
            )
        });
        let class_helpers = published_helpers.as_ref().map(|helpers| helpers.bindings);
        for plan in expression_plans {
            let evaluation_scope = remapped_evaluation_scope(plan.outer_scope, &evaluation_scopes);
            let class_scope = plan.class_scope;
            let nested_evaluation_scope = lower_named_expression(
                plan,
                class_helpers,
                evaluation_scope,
                &evaluation_scopes,
                tx.inner(),
            );
            evaluation_scopes.insert(class_scope, nested_evaluation_scope);
        }
        for plan in anonymous_expression_plans {
            let evaluation_scope = remapped_evaluation_scope(plan.outer_scope, &evaluation_scopes);
            lower_anonymous_expression(plan, class_helpers, evaluation_scope, tx.inner());
        }
        let mut output = Vec::with_capacity(statements.len());
        for statement in statements {
            if let Some(plan) = anonymous_default_plans
                .iter()
                .find(|plan| plan.statement.node_id() == statement.node_id())
                .cloned()
            {
                output.extend(lower_anonymous_default_export(plan, class_helpers, tx.inner()));
                continue;
            }
            let Some(plan) = plans
                .iter()
                .find(|plan| plan.statement.node_id() == statement.node_id())
                .cloned()
            else {
                output.push(statement);
                continue;
            };
            let lowered = lower_declaration(plan, class_helpers, tx.inner());
            output.extend(lowered);
        }

        if let Some(published) = published_helpers {
            let insertion = output
                .iter()
                .position(|statement| !statement.is_import_declaration(tx.ast()))
                .unwrap_or(output.len());
            output.splice(insertion..insertion, published.statements);
        }

        tx.replace_statements(StatementListOwner::Program(program), output);
        for owner in nested_owners {
            let mut output = Vec::new();
            for statement in statements_of(owner, tx.ast()) {
                if let Some(plan) = plans
                    .iter()
                    .find(|plan| {
                        plan.owner == owner && plan.statement.node_id() == statement.node_id()
                    })
                    .cloned()
                {
                    output.extend(lower_declaration(plan, class_helpers, tx.inner()));
                } else {
                    output.push(statement);
                }
            }
            tx.replace_statements(owner, output);
        }
    })?;
    Ok(())
}

fn remapped_evaluation_scope(
    mut scope: swc_next_ecma_ast::ScopeId,
    transitions: &HashMap<swc_next_ecma_ast::ScopeId, ScopeHandle>,
) -> ScopeHandle {
    let mut visited = HashSet::new();
    while visited.insert(scope) {
        match transitions.get(&scope).copied() {
            Some(ScopeHandle::Existing(parent)) => scope = parent,
            Some(pending @ ScopeHandle::Pending(_)) => return pending,
            None => return ScopeHandle::Existing(scope),
        }
    }
    ScopeHandle::Existing(scope)
}

fn remapped_definition_reference_scope(
    mut scope: swc_next_ecma_ast::ScopeId,
    class_scope: swc_next_ecma_ast::ScopeId,
    name_scope: swc_next_ecma_ast::ScopeId,
    definition_scope: ScopeHandle,
    transitions: &HashMap<swc_next_ecma_ast::ScopeId, ScopeHandle>,
) -> ScopeHandle {
    let mut visited = HashSet::new();
    while visited.insert(scope) {
        if scope == class_scope || scope == name_scope {
            return definition_scope;
        }
        match transitions.get(&scope).copied() {
            Some(ScopeHandle::Existing(parent)) => scope = parent,
            Some(pending @ ScopeHandle::Pending(_)) => return pending,
            None => return ScopeHandle::Existing(scope),
        }
    }
    ScopeHandle::Existing(scope)
}

fn statements_of(owner: StatementListOwner, ast: &Ast<'_>) -> Vec<Stmt> {
    let statements = match owner {
        StatementListOwner::Program(node) => node.body(ast),
        StatementListOwner::Block(node) => node.body(ast),
        StatementListOwner::FunctionBody(node) => node.body(ast),
        StatementListOwner::StaticBlock(node) => node.body(ast),
        StatementListOwner::SwitchCase(node) => node.consequent(ast),
        StatementListOwner::TsModuleBlock(node) => node.body(ast),
    };
    statements
        .iter()
        .map(|id| ast.get_node_in_sub_range(id))
        .collect()
}

fn lower_declaration(
    plan: ClassLoweringPlan,
    helpers: Option<ClassHelpers>,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) -> Vec<Stmt> {
    let span = tx.ast().span(plan.class.node_id());
    let binding_name = plan.binding.name(tx.ast());
    let binding_context = tx.ast().syntax_context(plan.binding.node_id());
    let transition = plan.scope_transition();
    let statement_parent = plan.owner.node_id();
    let constructor_parent = plan.export.map_or(statement_parent, ClassExport::node_id);
    let (function, function_scope) = match plan.constructor {
        ConstructorPlan::Synthetic => {
            let (function, params, body, prelude, derived_return) = tx.build_detached(|ast| {
                let prelude = ConstructorPrelude::build(ast, span, binding_name, binding_context);
                let directives = ast.add_typed_sub_range([prelude.directive]);
                let derived_return = plan.heritage.map(|_| {
                    DerivedConstructorReturn::build(
                        ast,
                        span,
                        binding_name,
                        binding_context,
                        helpers
                            .expect("derived class helpers must be published")
                            .call_super,
                    )
                });
                let statements = ast.add_typed_sub_range(
                    std::iter::once(Stmt::IfStatement(prelude.guard.statement)).chain(
                        derived_return.map(|derived| Stmt::ReturnStatement(derived.statement)),
                    ),
                );
                let body = ast.function_body(span, directives, statements);
                let params = ast.formal_parameters(
                    span,
                    TypedSubRange::empty(),
                    None,
                    FormalParameterKind::FormalParameters,
                );
                let function = ast.function(
                    span,
                    FunctionType::FunctionDeclaration,
                    Some(plan.binding),
                    false,
                    false,
                    params,
                    body,
                    None,
                    None,
                );
                (function, params, body, prelude, derived_return)
            });
            let function_scope = tx.create_scope(
                transition.create_parent,
                function.node_id(),
                ScopeKind::Function,
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                },
            );
            tx.register_node(function.node_id(), Some(constructor_parent), function_scope);
            tx.register_node(params.node_id(), Some(function.node_id()), function_scope);
            tx.register_node(body.node_id(), Some(function.node_id()), function_scope);
            prelude.register(
                tx,
                body.node_id(),
                function_scope.into(),
                SymbolHandle::Existing(plan.symbol),
            );
            if let Some(derived_return) = derived_return {
                derived_return.register(
                    tx,
                    body.node_id(),
                    function_scope.into(),
                    SymbolHandle::Existing(plan.symbol),
                    helpers
                        .expect("derived class helpers must be published")
                        .call_super
                        .symbol
                        .into(),
                );
            }
            (function, ScopeHandle::Pending(function_scope))
        }
        ConstructorPlan::Existing {
            definition,
            function,
            scope,
        } => {
            let original_body = function.body(tx.ast());
            let params = function.params(tx.ast());
            if plan.heritage.is_some() {
                lower_explicit_derived_constructor_body(
                    original_body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: binding_name,
                        class_context: binding_context,
                        class_symbol: SymbolHandle::Existing(plan.symbol),
                        helpers: helpers.expect("derived class helpers must be published"),
                    },
                    &plan.constructor_super_calls,
                    &plan.super_members,
                    &plan.constructor_this_nodes,
                    tx,
                );
            } else if plan.super_members.is_empty()
                || !lower_base_constructor_body(
                    original_body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: binding_name,
                        class_context: binding_context,
                        class_symbol: SymbolHandle::Existing(plan.symbol),
                        helpers: helpers.expect("super member helpers must be published"),
                    },
                    &plan.super_members,
                    tx,
                )
            {
                let directives: Vec<_> = original_body
                    .directives(tx.ast())
                    .iter()
                    .map(|id| tx.ast().get_node_in_sub_range(id))
                    .collect();
                let statements: Vec<_> = original_body
                    .body(tx.ast())
                    .iter()
                    .map(|id| tx.ast().get_node_in_sub_range(id))
                    .collect();
                let (replacement_body, prelude) = tx.build_detached(|ast| {
                    let prelude =
                        ConstructorPrelude::build(ast, span, binding_name, binding_context);
                    let directives = ast
                        .add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
                    let statements = ast.add_typed_sub_range(
                        std::iter::once(Stmt::IfStatement(prelude.guard.statement))
                            .chain(statements),
                    );
                    let replacement_body = ast.function_body(span, directives, statements);
                    (replacement_body, prelude)
                });
                tx.replace(
                    original_body.node_id(),
                    replacement_body.node_id(),
                    IdentityPolicy::Preserve,
                );
                prelude.register(
                    tx,
                    original_body.node_id(),
                    scope.into(),
                    SymbolHandle::Existing(plan.symbol),
                );
            }
            let replacement_function = tx.build_detached(|ast| {
                ast.function(
                    span,
                    FunctionType::FunctionDeclaration,
                    Some(plan.binding),
                    false,
                    false,
                    params,
                    original_body,
                    None,
                    None,
                )
            });
            tx.replace(
                function.node_id(),
                replacement_function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.reparent_node(function.node_id(), Some(constructor_parent));
            tx.reparent_scope(scope, transition.create_parent);
            tx.retire_node(definition.key(tx.ast()).node_id());
            tx.retire_node(definition.node_id());
            (function, ScopeHandle::Existing(scope))
        }
    };
    tx.reparent_node(plan.binding.node_id(), Some(function.node_id()));
    tx.rescope_node(plan.binding.node_id(), function_scope);
    tx.update_declaration_flags(plan.declaration, plan.output_flags);
    tx.update_symbol_flags(plan.symbol, plan.output_flags);

    let mut output =
        Vec::with_capacity(1 + usize::from(plan.heritage.is_some()) + plan.methods.len());
    if let Some(export) = plan.export {
        let replacement = tx.build_detached(|ast| match export {
            ClassExport::Named(export) => ast.stmt_export_named_declaration(
                ast.span(export.node_id()),
                Some(Decl::Function(function)),
                export.specifiers(ast),
                export.source(ast),
                export.attributes(ast),
                export.export_kind(ast),
            ),
            ClassExport::Default(export) => ast.stmt_export_default_declaration(
                ast.span(export.node_id()),
                ExportDefaultDeclarationKind::Function(function),
            ),
        });
        tx.replace(
            export.node_id(),
            replacement.node_id(),
            IdentityPolicy::Preserve,
        );
        output.push(plan.statement);
    } else {
        output.push(Stmt::Declaration(Decl::Function(function)));
    }
    if let Some(heritage) = plan.heritage {
        let inheritance = tx.build_detached(|ast| {
            LoweredInheritance::build(
                ast,
                heritage,
                binding_name,
                binding_context,
                helpers
                    .expect("derived class helpers must be published")
                    .inherits,
            )
        });
        let statement = inheritance.statement;
        inheritance.register(
            tx,
            statement_parent,
            transition.create_parent.into(),
            SymbolHandle::Existing(plan.symbol),
            helpers
                .expect("derived class helpers must be published")
                .inherits
                .symbol
                .into(),
        );
        output.push(statement);
    }
    let super_members = plan.super_members;
    for method in plan.methods {
        let mut lowered = tx
            .build_detached(|ast| LoweredMethod::build(ast, method, binding_name, binding_context));
        if !super_members.is_empty() {
            lowered.prepare_super_rewrites(
                tx,
                &super_members,
                binding_name,
                binding_context,
                helpers.expect("super member helpers must be published"),
            );
        }
        let method_statement = lowered.statement;
        lowered.register(
            tx,
            statement_parent,
            transition.create_parent.into(),
            SymbolHandle::Existing(plan.symbol),
        );
        output.push(method_statement);
    }
    tx.retire_node(plan.body.node_id());
    tx.retire_node(plan.class.node_id());
    tx.retire_scope(transition.retire);
    output
}

struct LoweredInheritance {
    statement: Stmt,
    heritage: Expr,
    helper_reference: IdentifierReference,
    class_reference: IdentifierReference,
}

impl LoweredInheritance {
    fn build(
        ast: &mut Ast<'_>,
        heritage: Expr,
        binding_name: AstUtf8,
        binding_context: SyntaxContext,
        helper: PrivateBinding,
    ) -> Self {
        let span = ast.span(heritage.node_id());
        let helper_reference = ast.identifier_reference(span, helper.text);
        ast.set_syntax_context(helper_reference.node_id(), helper.context);
        let class_reference = ast.identifier_reference(span, binding_name);
        ast.set_syntax_context(class_reference.node_id(), binding_context);
        let arguments = ast.add_typed_sub_range([
            Argument::Expr(Expr::IdentifierReference(class_reference)),
            Argument::Expr(heritage),
        ]);
        let call = ast.expr_call_expression(
            span,
            Expr::IdentifierReference(helper_reference),
            arguments,
            false,
            None,
        );
        let statement = ast.stmt_expression_statement(span, call);
        Self {
            statement,
            heritage,
            helper_reference,
            class_reference,
        }
    }

    fn register(
        self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        parent: swc_next_ecma_ast::NodeId,
        scope: ScopeHandle,
        class_symbol: SymbolHandle,
        helper_symbol: SymbolHandle,
    ) {
        tx.adopt_generated_subtree(self.statement, Some(parent), scope);
        if tx.semantic().is_node_active(self.heritage.node_id()) {
            tx.rescope_adopted_subtree(self.heritage.node_id(), scope);
        }
        tx.reference(
            scope,
            self.helper_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.helper_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(helper_symbol),
        );
        tx.reference(
            scope,
            self.class_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.class_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(class_symbol),
        );
    }
}

fn lower_anonymous_default_export(
    plan: AnonymousDefaultClassPlan,
    helpers: Option<ClassHelpers>,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) -> Vec<Stmt> {
    let span = tx.ast().span(plan.class.node_id());
    let is_derived = plan.heritage.is_some();
    let class_helpers = helpers;
    let name = tx.reserve_private_name(plan.outer_scope, "_class");
    let binding = tx.build_detached(|ast| {
        let binding = ast.binding_identifier(span, name.text, TypedSubRange::empty(), None, false);
        ast.set_syntax_context(binding.node_id(), name.context);
        binding
    });
    let constructor = match plan.constructor {
        ConstructorPlan::Synthetic => tx.build_detached(|ast| {
            let prelude = ConstructorPrelude::build(ast, span, name.text, name.context);
            let directives = ast.add_typed_sub_range([prelude.directive]);
            let statements = ast.add_typed_sub_range([Stmt::IfStatement(prelude.guard.statement)]);
            let body = ast.function_body(span, directives, statements);
            let params = ast.formal_parameters(
                span,
                TypedSubRange::empty(),
                None,
                FormalParameterKind::FormalParameters,
            );
            let function = ast.function(
                span,
                FunctionType::FunctionDeclaration,
                Some(binding),
                false,
                false,
                params,
                body,
                None,
                None,
            );
            ExpressionConstructorLowering::Synthetic {
                function,
                params,
                body,
                prelude,
            }
        }),
        ConstructorPlan::Existing {
            definition,
            function,
            scope,
        } => {
            let body = function.body(tx.ast());
            let directives: Vec<_> = body
                .directives(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let statements: Vec<_> = body
                .body(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let params = function.params(tx.ast());
            tx.build_detached(|ast| {
                let prelude = ConstructorPrelude::build(ast, span, name.text, name.context);
                let directives =
                    ast.add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
                let statements = ast.add_typed_sub_range(
                    std::iter::once(Stmt::IfStatement(prelude.guard.statement)).chain(statements),
                );
                let replacement_body = ast.function_body(span, directives, statements);
                let replacement_function = ast.function(
                    span,
                    FunctionType::FunctionDeclaration,
                    Some(binding),
                    false,
                    false,
                    params,
                    body,
                    None,
                    None,
                );
                ExpressionConstructorLowering::Existing {
                    definition,
                    function,
                    scope,
                    params,
                    body,
                    replacement_function,
                    replacement_body,
                    prelude,
                }
            })
        }
    };
    let synthetic_derived_return = match (is_derived, &constructor) {
        (true, ExpressionConstructorLowering::Synthetic { body, .. }) => {
            let class_helpers =
                class_helpers.expect("derived default export helpers must be published");
            Some(tx.build_detached(|ast| {
                let derived = DerivedConstructorReturn::build(
                    ast,
                    span,
                    name.text,
                    name.context,
                    class_helpers.call_super,
                );
                let statements: Vec<_> = body
                    .body(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .collect();
                let statements = ast.add_typed_sub_range(
                    statements
                        .into_iter()
                        .chain(std::iter::once(Stmt::ReturnStatement(derived.statement))),
                );
                body.set_body(ast, statements);
                derived
            }))
        }
        _ => None,
    };
    let function = constructor.function();
    let inheritance = plan.heritage.map(|heritage| {
        tx.build_detached(|ast| {
            LoweredInheritance::build(
                ast,
                heritage,
                name.text,
                name.context,
                class_helpers
                    .expect("derived default export helpers must be published")
                    .inherits,
            )
        })
    });
    let inheritance_statement = inheritance
        .as_ref()
        .map(|inheritance| inheritance.statement);
    let super_members = plan.super_members.clone();
    let lowered_methods: Vec<_> = plan
        .methods
        .iter()
        .copied()
        .map(|method| {
            let mut lowered =
                tx.build_detached(|ast| LoweredMethod::build(ast, method, name.text, name.context));
            if !super_members.is_empty() {
                lowered.prepare_super_rewrites(
                    tx,
                    &super_members,
                    name.text,
                    name.context,
                    helpers.expect("super member helpers must be published"),
                );
            }
            lowered
        })
        .collect();
    let method_statements: Vec<_> = lowered_methods
        .iter()
        .map(|method| method.statement)
        .collect();
    let replacement_export = tx.build_detached(|ast| {
        ast.stmt_export_default_declaration(
            ast.span(plan.export.node_id()),
            ExportDefaultDeclarationKind::Function(function),
        )
    });
    tx.replace(
        plan.export.node_id(),
        replacement_export.node_id(),
        IdentityPolicy::Preserve,
    );

    let flags = swc_next_ecma_semantic::SymbolFlags::FUNCTION
        | swc_next_ecma_semantic::SymbolFlags::ANNEX_B_FUNCTION
        | swc_next_ecma_semantic::SymbolFlags::IS_DEFAULT
        | swc_next_ecma_semantic::SymbolFlags::EXPORTED;
    let symbol = tx.declare(
        plan.outer_scope,
        binding.node_id(),
        tx.ast().canonicalize_utf8(name.text),
        flags,
    );
    let function_scope = match constructor {
        ExpressionConstructorLowering::Synthetic {
            function,
            params,
            body,
            prelude,
        } => {
            let scope = tx.create_scope(
                plan.outer_scope,
                function.node_id(),
                ScopeKind::Function,
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                },
            );
            tx.register_node(function.node_id(), Some(plan.export.node_id()), scope);
            tx.register_node(params.node_id(), Some(function.node_id()), scope);
            tx.register_node(body.node_id(), Some(function.node_id()), scope);
            prelude.register(
                tx,
                body.node_id(),
                scope.into(),
                SymbolHandle::Pending(symbol),
            );
            if let Some(derived) = synthetic_derived_return {
                derived.register(
                    tx,
                    body.node_id(),
                    scope.into(),
                    SymbolHandle::Pending(symbol),
                    class_helpers
                        .expect("derived default export helpers must be published")
                        .call_super
                        .symbol
                        .into(),
                );
            }
            ScopeHandle::Pending(scope)
        }
        ExpressionConstructorLowering::Existing {
            definition,
            function,
            scope,
            params: _,
            body,
            replacement_function,
            replacement_body,
            prelude,
        } => {
            if is_derived {
                lower_explicit_derived_constructor_body(
                    body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: name.text,
                        class_context: name.context,
                        class_symbol: SymbolHandle::Pending(symbol),
                        helpers: class_helpers
                            .expect("derived default export helpers must be published"),
                    },
                    &plan.constructor_super_calls,
                    &plan.super_members,
                    &plan.constructor_this_nodes,
                    tx,
                );
            } else if plan.super_members.is_empty()
                || !lower_base_constructor_body(
                    body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: name.text,
                        class_context: name.context,
                        class_symbol: SymbolHandle::Pending(symbol),
                        helpers: class_helpers.expect("super member helpers must be published"),
                    },
                    &plan.super_members,
                    tx,
                )
            {
                tx.replace(
                    body.node_id(),
                    replacement_body.node_id(),
                    IdentityPolicy::Preserve,
                );
                prelude.register(
                    tx,
                    body.node_id(),
                    scope.into(),
                    SymbolHandle::Pending(symbol),
                );
            }
            tx.replace(
                function.node_id(),
                replacement_function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.reparent_node(function.node_id(), Some(plan.export.node_id()));
            tx.reparent_scope(scope, plan.outer_scope);
            tx.retire_node(definition.key(tx.ast()).node_id());
            tx.retire_node(definition.node_id());
            ScopeHandle::Existing(scope)
        }
    };
    tx.register_node(binding.node_id(), Some(function.node_id()), function_scope);
    let statement_parent = tx.semantic().scope(plan.outer_scope).node;
    if let Some(inheritance) = inheritance {
        inheritance.register(
            tx,
            statement_parent,
            ScopeHandle::Existing(plan.outer_scope),
            SymbolHandle::Pending(symbol),
            class_helpers
                .expect("derived default export helpers must be published")
                .inherits
                .symbol
                .into(),
        );
    }
    for method in lowered_methods {
        method.register(
            tx,
            statement_parent,
            ScopeHandle::Existing(plan.outer_scope),
            SymbolHandle::Pending(symbol),
        );
    }
    tx.retarget_default_export_local(plan.export.node_id(), binding.node_id(), symbol);
    tx.retire_node(plan.body.node_id());
    tx.retire_node(plan.class.node_id());
    tx.retire_scope(plan.class_scope);

    std::iter::once(plan.statement)
        .chain(inheritance_statement)
        .chain(method_statements)
        .collect()
}

fn lower_anonymous_expression(
    plan: AnonymousClassExpressionPlan,
    helpers: Option<ClassHelpers>,
    evaluation_scope: ScopeHandle,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) {
    if !plan.methods.is_empty() || plan.heritage.is_some() || !plan.super_members.is_empty() {
        lower_anonymous_expression_iife(plan, helpers, evaluation_scope, tx);
        return;
    }
    let span = tx.ast().span(plan.class.node_id());
    let name = tx.reserve_private_name_in(plan.outer_scope, evaluation_scope, "_class");
    let name_scope = tx.create_scope(
        evaluation_scope,
        plan.class.node_id(),
        ScopeKind::ExpressionName,
        ScopeFlags {
            strict: true,
            ..ScopeFlags::default()
        },
    );
    let (binding, function_scope) = match plan.constructor {
        ConstructorPlan::Synthetic => {
            let (function, binding, params, body, prelude) = tx.build_detached(|ast| {
                let binding =
                    ast.binding_identifier(span, name.text, TypedSubRange::empty(), None, false);
                ast.set_syntax_context(binding.node_id(), name.context);
                let prelude = ConstructorPrelude::build(ast, span, name.text, name.context);
                let directives = ast.add_typed_sub_range([prelude.directive]);
                let statements =
                    ast.add_typed_sub_range([Stmt::IfStatement(prelude.guard.statement)]);
                let body = ast.function_body(span, directives, statements);
                let params = ast.formal_parameters(
                    span,
                    TypedSubRange::empty(),
                    None,
                    FormalParameterKind::FormalParameters,
                );
                let function = ast.function(
                    span,
                    FunctionType::FunctionExpression,
                    Some(binding),
                    false,
                    false,
                    params,
                    body,
                    None,
                    None,
                );
                (function, binding, params, body, prelude)
            });
            tx.reparent_scope(plan.class_scope, name_scope);
            tx.retag_scope(
                plan.class_scope,
                ScopeKind::Function,
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                },
            );
            tx.replace(
                plan.class.node_id(),
                function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.register_node(
                params.node_id(),
                Some(plan.class.node_id()),
                plan.class_scope,
            );
            tx.register_node(body.node_id(), Some(plan.class.node_id()), plan.class_scope);
            let symbol = tx.declare(
                name_scope,
                binding.node_id(),
                tx.ast().canonicalize_utf8(name.text),
                swc_next_ecma_semantic::SymbolFlags::FUNCTION,
            );
            prelude.register(
                tx,
                body.node_id(),
                plan.class_scope.into(),
                SymbolHandle::Pending(symbol),
            );
            (binding, ScopeHandle::Existing(plan.class_scope))
        }
        ConstructorPlan::Existing {
            definition,
            function,
            scope,
        } => {
            let original_body = function.body(tx.ast());
            let directives: Vec<_> = original_body
                .directives(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let statements: Vec<_> = original_body
                .body(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let params = function.params(tx.ast());
            let (replacement_function, replacement_body, binding, prelude) =
                tx.build_detached(|ast| {
                    let binding = ast.binding_identifier(
                        span,
                        name.text,
                        TypedSubRange::empty(),
                        None,
                        false,
                    );
                    ast.set_syntax_context(binding.node_id(), name.context);
                    let prelude = ConstructorPrelude::build(ast, span, name.text, name.context);
                    let directives = ast
                        .add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
                    let statements = ast.add_typed_sub_range(
                        std::iter::once(Stmt::IfStatement(prelude.guard.statement))
                            .chain(statements),
                    );
                    let replacement_body = ast.function_body(span, directives, statements);
                    let replacement_function = ast.function(
                        span,
                        FunctionType::FunctionExpression,
                        Some(binding),
                        false,
                        false,
                        params,
                        original_body,
                        None,
                        None,
                    );
                    (replacement_function, replacement_body, binding, prelude)
                });
            tx.replace(
                original_body.node_id(),
                replacement_body.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.replace(
                plan.class.node_id(),
                replacement_function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.reowner_scope(scope, plan.class.node_id());
            tx.reparent_scope(scope, name_scope);
            tx.rescope_node(plan.class.node_id(), scope);
            tx.reparent_node(params.node_id(), Some(plan.class.node_id()));
            tx.reparent_node(original_body.node_id(), Some(plan.class.node_id()));
            let symbol = tx.declare(
                name_scope,
                binding.node_id(),
                tx.ast().canonicalize_utf8(name.text),
                swc_next_ecma_semantic::SymbolFlags::FUNCTION,
            );
            prelude.register(
                tx,
                original_body.node_id(),
                scope.into(),
                SymbolHandle::Pending(symbol),
            );
            tx.retire_node(definition.key(tx.ast()).node_id());
            tx.retire_node(function.node_id());
            tx.retire_node(definition.node_id());
            tx.retire_scope(plan.class_scope);
            (binding, ScopeHandle::Existing(scope))
        }
    };
    tx.register_node(
        binding.node_id(),
        Some(plan.class.node_id()),
        function_scope,
    );
    tx.retire_node(plan.body.node_id());
}

fn lower_anonymous_expression_iife(
    plan: AnonymousClassExpressionPlan,
    helpers: Option<ClassHelpers>,
    evaluation_scope: ScopeHandle,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) {
    let span = tx.ast().span(plan.class.node_id());
    let is_derived = plan.heritage.is_some();
    let class_helpers = helpers;
    let name = tx.reserve_private_name_in(plan.outer_scope, evaluation_scope, "_class");
    let binding = tx.build_detached(|ast| {
        let binding = ast.binding_identifier(span, name.text, TypedSubRange::empty(), None, false);
        ast.set_syntax_context(binding.node_id(), name.context);
        binding
    });
    let constructor = match plan.constructor {
        ConstructorPlan::Synthetic => tx.build_detached(|ast| {
            let prelude = ConstructorPrelude::build(ast, span, name.text, name.context);
            let directives = ast.add_typed_sub_range([prelude.directive]);
            let statements = ast.add_typed_sub_range([Stmt::IfStatement(prelude.guard.statement)]);
            let body = ast.function_body(span, directives, statements);
            let params = ast.formal_parameters(
                span,
                TypedSubRange::empty(),
                None,
                FormalParameterKind::FormalParameters,
            );
            let function = ast.function(
                span,
                FunctionType::FunctionDeclaration,
                Some(binding),
                false,
                false,
                params,
                body,
                None,
                None,
            );
            ExpressionConstructorLowering::Synthetic {
                function,
                params,
                body,
                prelude,
            }
        }),
        ConstructorPlan::Existing {
            definition,
            function,
            scope,
        } => {
            let body = function.body(tx.ast());
            let directives: Vec<_> = body
                .directives(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let statements: Vec<_> = body
                .body(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let params = function.params(tx.ast());
            tx.build_detached(|ast| {
                let prelude = ConstructorPrelude::build(ast, span, name.text, name.context);
                let directives =
                    ast.add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
                let statements = ast.add_typed_sub_range(
                    std::iter::once(Stmt::IfStatement(prelude.guard.statement)).chain(statements),
                );
                let replacement_body = ast.function_body(span, directives, statements);
                let replacement_function = ast.function(
                    span,
                    FunctionType::FunctionDeclaration,
                    Some(binding),
                    false,
                    false,
                    params,
                    body,
                    None,
                    None,
                );
                ExpressionConstructorLowering::Existing {
                    definition,
                    function,
                    scope,
                    params,
                    body,
                    replacement_function,
                    replacement_body,
                    prelude,
                }
            })
        }
    };
    let synthetic_derived_return = match (is_derived, &constructor) {
        (true, ExpressionConstructorLowering::Synthetic { body, .. }) => {
            let class_helpers =
                class_helpers.expect("derived expression helpers must be published");
            Some(tx.build_detached(|ast| {
                let derived = DerivedConstructorReturn::build(
                    ast,
                    span,
                    name.text,
                    name.context,
                    class_helpers.call_super,
                );
                let statements: Vec<_> = body
                    .body(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .collect();
                let statements = ast.add_typed_sub_range(
                    statements
                        .into_iter()
                        .chain(std::iter::once(Stmt::ReturnStatement(derived.statement))),
                );
                body.set_body(ast, statements);
                derived
            }))
        }
        _ => None,
    };
    let mut methods = plan.methods.clone();
    let heritage_capture = plan.heritage.map(|heritage| {
        capture_expression(heritage, "_super", tx, plan.outer_scope, evaluation_scope)
    });
    let mut captures = Vec::new();
    captures.extend(heritage_capture);
    captures.extend(capture_computed_keys(
        &mut methods,
        tx,
        plan.outer_scope,
        evaluation_scope,
    ));
    let super_members = plan.super_members.clone();
    let lowered_methods: Vec<_> = methods
        .iter()
        .copied()
        .map(|method| {
            let mut lowered =
                tx.build_detached(|ast| LoweredMethod::build(ast, method, name.text, name.context));
            if !super_members.is_empty() {
                lowered.prepare_super_rewrites(
                    tx,
                    &super_members,
                    name.text,
                    name.context,
                    helpers.expect("super member helpers must be published"),
                );
            }
            lowered
        })
        .collect();
    let method_statements: Vec<_> = lowered_methods
        .iter()
        .map(|method| method.statement)
        .collect();
    let constructor_function = constructor.function();
    let constructor_statement = Stmt::Declaration(Decl::Function(constructor_function));
    let inheritance = captures.first().filter(|_| is_derived).map(|capture| {
        tx.build_detached(|ast| {
            LoweredInheritance::build(
                ast,
                Expr::IdentifierReference(capture.reference),
                name.text,
                name.context,
                class_helpers
                    .expect("derived expression helpers must be published")
                    .inherits,
            )
        })
    });
    let inheritance_statement = inheritance
        .as_ref()
        .map(|inheritance| inheritance.statement);
    let (call, iife, iife_params, iife_body, return_statement, return_reference) = tx
        .build_detached(|ast| {
            let return_reference = ast.identifier_reference(span, name.text);
            ast.set_syntax_context(return_reference.node_id(), name.context);
            let return_statement =
                ast.return_statement(span, Some(Expr::IdentifierReference(return_reference)));
            let statements = ast.add_typed_sub_range(
                std::iter::once(constructor_statement)
                    .chain(inheritance_statement)
                    .chain(method_statements)
                    .chain(std::iter::once(Stmt::ReturnStatement(return_statement))),
            );
            let iife_body = ast.function_body(span, TypedSubRange::empty(), statements);
            let parameter_items =
                ast.add_typed_sub_range(captures.iter().map(|capture| capture.parameter));
            let iife_params = ast.formal_parameters(
                span,
                parameter_items,
                None,
                FormalParameterKind::FormalParameters,
            );
            let iife = ast.function(
                span,
                FunctionType::FunctionExpression,
                None,
                false,
                false,
                iife_params,
                iife_body,
                None,
                None,
            );
            let arguments = ast.add_typed_sub_range(
                captures
                    .iter()
                    .map(|capture| Argument::Expr(capture.expression)),
            );
            let call = ast.call_expression(span, Expr::Function(iife), arguments, false, None);
            (
                call,
                iife,
                iife_params,
                iife_body,
                return_statement,
                return_reference,
            )
        });

    let iife_scope = tx.create_scope(
        evaluation_scope,
        iife.node_id(),
        ScopeKind::Function,
        ScopeFlags {
            strict: tx.semantic().scope(plan.outer_scope).flags.strict,
            ..ScopeFlags::default()
        },
    );
    tx.replace(
        plan.class.node_id(),
        call.node_id(),
        IdentityPolicy::Preserve,
    );
    tx.rescope_node(plan.class.node_id(), evaluation_scope);
    tx.register_node(iife.node_id(), Some(plan.class.node_id()), iife_scope);
    tx.register_subtree(iife_params, Some(iife.node_id()), iife_scope);
    tx.register_node(iife_body.node_id(), Some(iife.node_id()), iife_scope);
    let symbol = tx.declare(
        iife_scope,
        binding.node_id(),
        tx.ast().canonicalize_utf8(name.text),
        swc_next_ecma_semantic::SymbolFlags::FUNCTION
            | swc_next_ecma_semantic::SymbolFlags::ANNEX_B_FUNCTION,
    );
    let capture_symbols: Vec<_> = captures
        .iter()
        .map(|capture| {
            tx.declare(
                iife_scope,
                capture.binding.node_id(),
                tx.ast().canonicalize_utf8(capture.name.text),
                swc_next_ecma_semantic::SymbolFlags::FUNCTION_SCOPED_VAR
                    | swc_next_ecma_semantic::SymbolFlags::PARAMETER,
            )
        })
        .collect();

    let constructor_scope = match constructor {
        ExpressionConstructorLowering::Synthetic {
            function,
            params,
            body,
            prelude,
        } => {
            let scope = tx.create_scope(
                iife_scope,
                function.node_id(),
                ScopeKind::Function,
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                },
            );
            tx.register_node(function.node_id(), Some(iife_body.node_id()), scope);
            tx.register_node(params.node_id(), Some(function.node_id()), scope);
            tx.register_node(body.node_id(), Some(function.node_id()), scope);
            prelude.register(
                tx,
                body.node_id(),
                scope.into(),
                SymbolHandle::Pending(symbol),
            );
            if let Some(derived) = synthetic_derived_return {
                derived.register(
                    tx,
                    body.node_id(),
                    scope.into(),
                    SymbolHandle::Pending(symbol),
                    class_helpers
                        .expect("derived expression helpers must be published")
                        .call_super
                        .symbol
                        .into(),
                );
            }
            ScopeHandle::Pending(scope)
        }
        ExpressionConstructorLowering::Existing {
            definition,
            function,
            scope,
            params: _,
            body,
            replacement_function,
            replacement_body,
            prelude,
        } => {
            if is_derived {
                lower_explicit_derived_constructor_body(
                    body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: name.text,
                        class_context: name.context,
                        class_symbol: SymbolHandle::Pending(symbol),
                        helpers: class_helpers
                            .expect("derived expression helpers must be published"),
                    },
                    &plan.constructor_super_calls,
                    &plan.super_members,
                    &plan.constructor_this_nodes,
                    tx,
                );
            } else if plan.super_members.is_empty()
                || !lower_base_constructor_body(
                    body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: name.text,
                        class_context: name.context,
                        class_symbol: SymbolHandle::Pending(symbol),
                        helpers: class_helpers.expect("super member helpers must be published"),
                    },
                    &plan.super_members,
                    tx,
                )
            {
                tx.replace(
                    body.node_id(),
                    replacement_body.node_id(),
                    IdentityPolicy::Preserve,
                );
                prelude.register(
                    tx,
                    body.node_id(),
                    scope.into(),
                    SymbolHandle::Pending(symbol),
                );
            }
            tx.replace(
                function.node_id(),
                replacement_function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.reparent_node(function.node_id(), Some(iife_body.node_id()));
            tx.reparent_scope(scope, iife_scope);
            tx.retire_node(definition.key(tx.ast()).node_id());
            tx.retire_node(definition.node_id());
            ScopeHandle::Existing(scope)
        }
    };
    if let Some(inheritance) = inheritance {
        inheritance.register(
            tx,
            iife_body.node_id(),
            ScopeHandle::Pending(iife_scope),
            SymbolHandle::Pending(symbol),
            class_helpers
                .expect("derived expression helpers must be published")
                .inherits
                .symbol
                .into(),
        );
    }
    tx.register_node(
        binding.node_id(),
        Some(constructor_function.node_id()),
        constructor_scope,
    );
    for method in lowered_methods {
        method.register(
            tx,
            iife_body.node_id(),
            ScopeHandle::Pending(iife_scope),
            SymbolHandle::Pending(symbol),
        );
    }
    for (capture, capture_symbol) in captures.into_iter().zip(capture_symbols) {
        tx.reparent_node(capture.expression.node_id(), Some(plan.class.node_id()));
        tx.rescope_adopted_subtree(capture.expression.node_id(), evaluation_scope);
        tx.reference(
            iife_scope,
            capture.reference.node_id(),
            tx.ast().canonicalize_utf8(capture.name.text),
            ReferenceFlags::default(),
            Some(capture_symbol),
        );
    }
    tx.register_subtree(
        Stmt::ReturnStatement(return_statement),
        Some(iife_body.node_id()),
        iife_scope,
    );
    tx.reference(
        iife_scope,
        return_reference.node_id(),
        tx.ast().canonicalize_utf8(return_reference.name(tx.ast())),
        ReferenceFlags::default(),
        Some(symbol),
    );
    tx.retire_node(plan.body.node_id());
    tx.retire_scope(plan.class_scope);
}

fn lower_named_expression(
    plan: ClassExpressionPlan,
    helpers: Option<ClassHelpers>,
    evaluation_scope: ScopeHandle,
    evaluation_scopes: &HashMap<swc_next_ecma_ast::ScopeId, ScopeHandle>,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) -> ScopeHandle {
    if !plan.methods.is_empty() || plan.heritage.is_some() || !plan.super_members.is_empty() {
        return lower_named_expression_iife(plan, helpers, evaluation_scope, evaluation_scopes, tx);
    }
    let span = tx.ast().span(plan.class.node_id());
    let binding_name = plan.binding.name(tx.ast());
    let binding_context = tx.ast().syntax_context(plan.binding.node_id());
    match plan.constructor {
        ConstructorPlan::Synthetic => {
            let (function, params, body, prelude) = tx.build_detached(|ast| {
                let prelude = ConstructorPrelude::build(ast, span, binding_name, binding_context);
                let directives = ast.add_typed_sub_range([prelude.directive]);
                let statements =
                    ast.add_typed_sub_range([Stmt::IfStatement(prelude.guard.statement)]);
                let body = ast.function_body(span, directives, statements);
                let params = ast.formal_parameters(
                    span,
                    TypedSubRange::empty(),
                    None,
                    FormalParameterKind::FormalParameters,
                );
                let function = ast.function(
                    span,
                    FunctionType::FunctionExpression,
                    Some(plan.binding),
                    false,
                    false,
                    params,
                    body,
                    None,
                    None,
                );
                (function, params, body, prelude)
            });
            tx.replace(
                plan.class.node_id(),
                function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.retag_scope(
                plan.class_scope,
                ScopeKind::Function,
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                },
            );
            tx.register_node(
                params.node_id(),
                Some(plan.class.node_id()),
                plan.class_scope,
            );
            tx.register_node(body.node_id(), Some(plan.class.node_id()), plan.class_scope);
            prelude.register(
                tx,
                body.node_id(),
                plan.class_scope.into(),
                SymbolHandle::Existing(plan.symbol),
            );
        }
        ConstructorPlan::Existing {
            definition,
            function,
            scope,
        } => {
            let original_body = function.body(tx.ast());
            let directives: Vec<_> = original_body
                .directives(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let statements: Vec<_> = original_body
                .body(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let params = function.params(tx.ast());
            let (replacement_function, replacement_body, prelude) = tx.build_detached(|ast| {
                let prelude = ConstructorPrelude::build(ast, span, binding_name, binding_context);
                let directives =
                    ast.add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
                let statements = ast.add_typed_sub_range(
                    std::iter::once(Stmt::IfStatement(prelude.guard.statement)).chain(statements),
                );
                let replacement_body = ast.function_body(span, directives, statements);
                let replacement_function = ast.function(
                    span,
                    FunctionType::FunctionExpression,
                    Some(plan.binding),
                    false,
                    false,
                    params,
                    original_body,
                    None,
                    None,
                );
                (replacement_function, replacement_body, prelude)
            });
            tx.replace(
                original_body.node_id(),
                replacement_body.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.replace(
                plan.class.node_id(),
                replacement_function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.reowner_scope(scope, plan.class.node_id());
            tx.reparent_scope(scope, plan.name_scope);
            tx.rescope_node(plan.class.node_id(), scope);
            tx.reparent_node(params.node_id(), Some(plan.class.node_id()));
            tx.reparent_node(original_body.node_id(), Some(plan.class.node_id()));
            tx.rescope_node(plan.binding.node_id(), scope);
            prelude.register(
                tx,
                original_body.node_id(),
                scope.into(),
                SymbolHandle::Existing(plan.symbol),
            );
            tx.retire_node(definition.key(tx.ast()).node_id());
            tx.retire_node(function.node_id());
            tx.retire_node(definition.node_id());
            tx.retire_scope(plan.class_scope);
        }
    }
    tx.update_declaration_flags(plan.declaration, plan.output_flags);
    tx.update_symbol_flags(plan.symbol, plan.output_flags);
    tx.retire_node(plan.body.node_id());
    if evaluation_scope != ScopeHandle::Existing(plan.outer_scope) {
        tx.reparent_scope(plan.name_scope, evaluation_scope);
    }
    evaluation_scope
}

enum ExpressionConstructorLowering {
    Synthetic {
        function: Function,
        params: swc_next_ecma_ast::FormalParameters,
        body: swc_next_ecma_ast::FunctionBody,
        prelude: ConstructorPrelude,
    },
    Existing {
        definition: swc_next_ecma_ast::MethodDefinition,
        function: Function,
        scope: swc_next_ecma_ast::ScopeId,
        params: swc_next_ecma_ast::FormalParameters,
        body: swc_next_ecma_ast::FunctionBody,
        replacement_function: Function,
        replacement_body: swc_next_ecma_ast::FunctionBody,
        prelude: ConstructorPrelude,
    },
}

impl ExpressionConstructorLowering {
    fn function(&self) -> Function {
        match self {
            Self::Synthetic { function, .. } | Self::Existing { function, .. } => *function,
        }
    }
}

fn lower_named_expression_iife(
    plan: ClassExpressionPlan,
    helpers: Option<ClassHelpers>,
    evaluation_scope: ScopeHandle,
    evaluation_scopes: &HashMap<swc_next_ecma_ast::ScopeId, ScopeHandle>,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) -> ScopeHandle {
    let span = tx.ast().span(plan.class.node_id());
    let is_derived = plan.heritage.is_some();
    let class_helpers = helpers;
    let binding_name = plan.binding.name(tx.ast());
    let binding_context = tx.ast().syntax_context(plan.binding.node_id());
    let tdz_sentinel = (!plan.definition_tdz_self_references.is_empty()).then(|| {
        let name = tx.reserve_private_name_in(plan.outer_scope, evaluation_scope, "_class_ref");
        let (parameter, binding) = tx.build_detached(|ast| {
            let pattern = ast.formal_parameter_pattern_binding_pattern_binding_identifier(
                span,
                name.text,
                TypedSubRange::empty(),
                None,
                false,
            );
            let binding = pattern
                .as_binding_pattern(ast)
                .and_then(|pattern| pattern.as_binding_identifier(ast))
                .expect("class TDZ sentinel parameter is an identifier");
            ast.set_syntax_context(binding.node_id(), name.context);
            let parameter = ast.formal_parameter_item_formal_parameter(span, pattern);
            (parameter, binding)
        });
        (name, parameter, binding)
    });
    let constructor = match plan.constructor {
        ConstructorPlan::Synthetic => tx.build_detached(|ast| {
            let prelude = ConstructorPrelude::build(ast, span, binding_name, binding_context);
            let directives = ast.add_typed_sub_range([prelude.directive]);
            let statements = ast.add_typed_sub_range([Stmt::IfStatement(prelude.guard.statement)]);
            let body = ast.function_body(span, directives, statements);
            let params = ast.formal_parameters(
                span,
                TypedSubRange::empty(),
                None,
                FormalParameterKind::FormalParameters,
            );
            let function = ast.function(
                span,
                FunctionType::FunctionDeclaration,
                Some(plan.binding),
                false,
                false,
                params,
                body,
                None,
                None,
            );
            ExpressionConstructorLowering::Synthetic {
                function,
                params,
                body,
                prelude,
            }
        }),
        ConstructorPlan::Existing {
            definition,
            function,
            scope,
        } => {
            let body = function.body(tx.ast());
            let directives: Vec<_> = body
                .directives(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let statements: Vec<_> = body
                .body(tx.ast())
                .iter()
                .map(|id| tx.ast().get_node_in_sub_range(id))
                .collect();
            let params = function.params(tx.ast());
            tx.build_detached(|ast| {
                let prelude = ConstructorPrelude::build(ast, span, binding_name, binding_context);
                let directives =
                    ast.add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
                let statements = ast.add_typed_sub_range(
                    std::iter::once(Stmt::IfStatement(prelude.guard.statement)).chain(statements),
                );
                let replacement_body = ast.function_body(span, directives, statements);
                let replacement_function = ast.function(
                    span,
                    FunctionType::FunctionDeclaration,
                    Some(plan.binding),
                    false,
                    false,
                    params,
                    body,
                    None,
                    None,
                );
                ExpressionConstructorLowering::Existing {
                    definition,
                    function,
                    scope,
                    params,
                    body,
                    replacement_function,
                    replacement_body,
                    prelude,
                }
            })
        }
    };

    let synthetic_derived_return = match (is_derived, &constructor) {
        (true, ExpressionConstructorLowering::Synthetic { body, .. }) => {
            let class_helpers =
                class_helpers.expect("derived expression helpers must be published");
            Some(tx.build_detached(|ast| {
                let derived = DerivedConstructorReturn::build(
                    ast,
                    span,
                    binding_name,
                    binding_context,
                    class_helpers.call_super,
                );
                let statements: Vec<_> = body
                    .body(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .collect();
                let statements = ast.add_typed_sub_range(
                    statements
                        .into_iter()
                        .chain(std::iter::once(Stmt::ReturnStatement(derived.statement))),
                );
                body.set_body(ast, statements);
                derived
            }))
        }
        _ => None,
    };

    let mut methods = plan.methods.clone();
    let heritage_capture = plan.heritage.map(|heritage| {
        capture_expression(heritage, "_super", tx, plan.outer_scope, evaluation_scope)
    });
    let mut captures = Vec::new();
    captures.extend(heritage_capture);
    captures.extend(capture_computed_keys(
        &mut methods,
        tx,
        plan.outer_scope,
        evaluation_scope,
    ));
    let definition_tdz_replacements = tdz_sentinel
        .as_ref()
        .map(|(sentinel, _, _)| {
            let helper = class_helpers
                .expect("class name TDZ helper must be published")
                .class_name_tdz_ref
                .expect("class name TDZ helper must be published");
            plan.definition_tdz_self_references
                .iter()
                .copied()
                .map(|reference| {
                    prepare_definition_tdz_replacement(
                        reference,
                        *sentinel,
                        binding_name,
                        helper,
                        tx,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let super_members = plan.super_members.clone();
    let lowered_methods: Vec<_> = methods
        .iter()
        .copied()
        .map(|method| {
            let mut lowered = tx.build_detached(|ast| {
                LoweredMethod::build(ast, method, binding_name, binding_context)
            });
            if !super_members.is_empty() {
                lowered.prepare_super_rewrites(
                    tx,
                    &super_members,
                    binding_name,
                    binding_context,
                    helpers.expect("super member helpers must be published"),
                );
            }
            lowered
        })
        .collect();
    let method_statements: Vec<_> = lowered_methods
        .iter()
        .map(|method| method.statement)
        .collect();
    let constructor_function = constructor.function();
    let constructor_statement = Stmt::Declaration(Decl::Function(constructor_function));
    let inheritance = captures.first().filter(|_| is_derived).map(|capture| {
        tx.build_detached(|ast| {
            LoweredInheritance::build(
                ast,
                Expr::IdentifierReference(capture.reference),
                binding_name,
                binding_context,
                class_helpers
                    .expect("derived expression helpers must be published")
                    .inherits,
            )
        })
    });
    let inheritance_statement = inheritance
        .as_ref()
        .map(|inheritance| inheritance.statement);
    let tdz_assignment = tdz_sentinel.as_ref().map(|(sentinel, _, _)| {
        tx.build_detached(|ast| {
            let sentinel_reference = ast.identifier_reference(span, sentinel.text);
            ast.set_syntax_context(sentinel_reference.node_id(), sentinel.context);
            let class_reference = ast.identifier_reference(span, binding_name);
            ast.set_syntax_context(class_reference.node_id(), binding_context);
            let target = AssignmentTarget::SimpleAssignmentTarget(
                SimpleAssignmentTarget::IdentifierReference(sentinel_reference),
            );
            let assignment = ast.expr_assignment_expression(
                span,
                target,
                Expr::IdentifierReference(class_reference),
                AssignmentOperator::Assign,
            );
            (
                ast.stmt_expression_statement(span, assignment),
                sentinel_reference,
                class_reference,
            )
        })
    });
    let tdz_assignment_statement = tdz_assignment.as_ref().map(|(statement, _, _)| *statement);
    let (call, iife, iife_params, iife_body, return_statement, return_reference) = tx
        .build_detached(|ast| {
            let return_reference = ast.identifier_reference(span, binding_name);
            ast.set_syntax_context(return_reference.node_id(), binding_context);
            let return_statement =
                ast.return_statement(span, Some(Expr::IdentifierReference(return_reference)));
            let statements = ast.add_typed_sub_range(
                std::iter::once(constructor_statement)
                    .chain(inheritance_statement)
                    .chain(method_statements)
                    .chain(tdz_assignment_statement)
                    .chain(std::iter::once(Stmt::ReturnStatement(return_statement))),
            );
            let iife_body = ast.function_body(span, TypedSubRange::empty(), statements);
            let parameter_items =
                ast.add_typed_sub_range(captures.iter().map(|capture| capture.parameter));
            let iife_params = ast.formal_parameters(
                span,
                parameter_items,
                None,
                FormalParameterKind::FormalParameters,
            );
            let iife = ast.function(
                span,
                FunctionType::FunctionExpression,
                None,
                false,
                false,
                iife_params,
                iife_body,
                None,
                None,
            );
            let arguments = ast.add_typed_sub_range(
                captures
                    .iter()
                    .map(|capture| Argument::Expr(capture.expression)),
            );
            let call = ast.call_expression(span, Expr::Function(iife), arguments, false, None);
            (
                call,
                iife,
                iife_params,
                iife_body,
                return_statement,
                return_reference,
            )
        });

    let tdz_wrapper = tdz_sentinel.as_ref().map(|(_, parameter, _)| {
        let helper = class_helpers
            .expect("class name TDZ helper must be published")
            .class_name_tdz_ref
            .expect("class name TDZ helper must be published");
        tx.build_detached(|ast| {
            let return_statement = ast.return_statement(span, Some(Expr::CallExpression(call)));
            let statements = ast.add_typed_sub_range([Stmt::ReturnStatement(return_statement)]);
            let body = ast.function_body(span, TypedSubRange::empty(), statements);
            let parameters = ast.add_typed_sub_range([*parameter]);
            let params = ast.formal_parameters(
                span,
                parameters,
                None,
                FormalParameterKind::FormalParameters,
            );
            let function = ast.function(
                span,
                FunctionType::FunctionExpression,
                None,
                false,
                false,
                params,
                body,
                None,
                None,
            );
            let helper_reference = ast.identifier_reference(span, helper.text);
            ast.set_syntax_context(helper_reference.node_id(), helper.context);
            let arguments = ast
                .add_typed_sub_range([Argument::Expr(Expr::IdentifierReference(helper_reference))]);
            let call = ast.call_expression(span, Expr::Function(function), arguments, false, None);
            ClassNameTdzWrapper {
                call,
                function,
                params,
                body,
                return_statement,
                helper_reference,
            }
        })
    });
    let replacement_call = tdz_wrapper.as_ref().map_or(call, |wrapper| wrapper.call);
    let outer_iife_scope = tdz_wrapper.as_ref().map(|wrapper| {
        tx.create_scope(
            evaluation_scope,
            wrapper.function.node_id(),
            ScopeKind::Function,
            ScopeFlags {
                strict: tx.semantic().scope(plan.outer_scope).flags.strict,
                ..ScopeFlags::default()
            },
        )
    });
    let iife_parent_scope = outer_iife_scope
        .map(ScopeHandle::Pending)
        .unwrap_or(evaluation_scope);
    let iife_scope = tx.create_scope(
        iife_parent_scope,
        iife.node_id(),
        ScopeKind::Function,
        ScopeFlags {
            strict: tx.semantic().scope(plan.outer_scope).flags.strict,
            ..ScopeFlags::default()
        },
    );
    tx.replace(
        plan.class.node_id(),
        replacement_call.node_id(),
        IdentityPolicy::Preserve,
    );
    tx.rescope_node(plan.class.node_id(), evaluation_scope);
    let mut sentinel_symbol = None;
    if let (Some(wrapper), Some(scope), Some((_, _, binding))) = (
        tdz_wrapper.as_ref(),
        outer_iife_scope,
        tdz_sentinel.as_ref(),
    ) {
        tx.register_node(
            wrapper.function.node_id(),
            Some(plan.class.node_id()),
            scope,
        );
        tx.register_subtree(wrapper.params, Some(wrapper.function.node_id()), scope);
        tx.register_node(
            wrapper.body.node_id(),
            Some(wrapper.function.node_id()),
            scope,
        );
        tx.register_node(
            wrapper.return_statement.node_id(),
            Some(wrapper.body.node_id()),
            scope,
        );
        tx.register_node(
            call.node_id(),
            Some(wrapper.return_statement.node_id()),
            scope,
        );
        tx.register_node(
            wrapper.helper_reference.node_id(),
            Some(plan.class.node_id()),
            evaluation_scope,
        );
        sentinel_symbol = Some(tx.declare(
            scope,
            binding.node_id(),
            tx.ast().canonicalize_utf8(binding.name(tx.ast())),
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        ));
        tx.reference(
            evaluation_scope,
            wrapper.helper_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(wrapper.helper_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(
                class_helpers
                    .expect("class name TDZ helper must be published")
                    .class_name_tdz_ref
                    .expect("class name TDZ helper must be published")
                    .symbol,
            ),
        );
    }
    let iife_parent = tdz_wrapper
        .as_ref()
        .map_or(plan.class.node_id(), |_| call.node_id());
    tx.register_node(iife.node_id(), Some(iife_parent), iife_scope);
    tx.register_subtree(iife_params, Some(iife.node_id()), iife_scope);
    tx.register_node(iife_body.node_id(), Some(iife.node_id()), iife_scope);
    let capture_symbols: Vec<_> = captures
        .iter()
        .map(|capture| {
            tx.declare(
                iife_scope,
                capture.binding.node_id(),
                tx.ast().canonicalize_utf8(capture.name.text),
                swc_next_ecma_semantic::SymbolFlags::FUNCTION_SCOPED_VAR
                    | swc_next_ecma_semantic::SymbolFlags::PARAMETER,
            )
        })
        .collect();

    let definition_evaluation_scope = outer_iife_scope
        .map(ScopeHandle::Pending)
        .unwrap_or(evaluation_scope);
    for replacement in definition_tdz_replacements {
        let scope = remapped_definition_reference_scope(
            replacement.original_scope,
            plan.class_scope,
            plan.name_scope,
            definition_evaluation_scope,
            evaluation_scopes,
        );
        tx.retire_reference(replacement.original_reference);
        tx.replace_expression(
            replacement.original,
            replacement.replacement,
            IdentityPolicy::Preserve,
        );
        tx.register_subtree_children(
            replacement.replacement,
            replacement.original.node_id(),
            scope,
        );
        for generated in replacement.references {
            let symbol: SymbolHandle = match generated.target {
                GeneratedHeritageTdzTarget::Helper(helper) => helper.symbol.into(),
                GeneratedHeritageTdzTarget::Sentinel => sentinel_symbol
                    .expect("class TDZ sentinel symbol must be declared")
                    .into(),
            };
            tx.reference(
                scope,
                generated.reference.node_id(),
                tx.ast()
                    .canonicalize_utf8(generated.reference.name(tx.ast())),
                ReferenceFlags::default(),
                Some(symbol),
            );
        }
    }

    let constructor_scope = match constructor {
        ExpressionConstructorLowering::Synthetic {
            function,
            params,
            body,
            prelude,
        } => {
            let scope = tx.create_scope(
                iife_scope,
                function.node_id(),
                ScopeKind::Function,
                ScopeFlags {
                    strict: true,
                    ..ScopeFlags::default()
                },
            );
            tx.register_node(function.node_id(), Some(iife_body.node_id()), scope);
            tx.register_node(params.node_id(), Some(function.node_id()), scope);
            tx.register_node(body.node_id(), Some(function.node_id()), scope);
            prelude.register(
                tx,
                body.node_id(),
                scope.into(),
                SymbolHandle::Existing(plan.symbol),
            );
            if let Some(derived) = synthetic_derived_return {
                derived.register(
                    tx,
                    body.node_id(),
                    scope.into(),
                    SymbolHandle::Existing(plan.symbol),
                    class_helpers
                        .expect("derived expression helpers must be published")
                        .call_super
                        .symbol
                        .into(),
                );
            }
            ScopeHandle::Pending(scope)
        }
        ExpressionConstructorLowering::Existing {
            definition,
            function,
            scope,
            params,
            body,
            replacement_function,
            replacement_body,
            prelude,
        } => {
            if is_derived {
                lower_explicit_derived_constructor_body(
                    body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: binding_name,
                        class_context: binding_context,
                        class_symbol: SymbolHandle::Existing(plan.symbol),
                        helpers: class_helpers
                            .expect("derived expression helpers must be published"),
                    },
                    &plan.constructor_super_calls,
                    &plan.super_members,
                    &plan.constructor_this_nodes,
                    tx,
                );
            } else if plan.super_members.is_empty()
                || !lower_base_constructor_body(
                    body,
                    ConstructorLoweringContext {
                        scope,
                        class_name: binding_name,
                        class_context: binding_context,
                        class_symbol: SymbolHandle::Existing(plan.symbol),
                        helpers: class_helpers.expect("super member helpers must be published"),
                    },
                    &plan.super_members,
                    tx,
                )
            {
                tx.replace(
                    body.node_id(),
                    replacement_body.node_id(),
                    IdentityPolicy::Preserve,
                );
                prelude.register(
                    tx,
                    body.node_id(),
                    scope.into(),
                    SymbolHandle::Existing(plan.symbol),
                );
            }
            tx.replace(
                function.node_id(),
                replacement_function.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.reparent_node(function.node_id(), Some(iife_body.node_id()));
            tx.reparent_scope(scope, iife_scope);
            tx.retire_node(definition.key(tx.ast()).node_id());
            tx.retire_node(definition.node_id());
            let _ = params;
            ScopeHandle::Existing(scope)
        }
    };

    if let Some(inheritance) = inheritance {
        inheritance.register(
            tx,
            iife_body.node_id(),
            ScopeHandle::Pending(iife_scope),
            SymbolHandle::Existing(plan.symbol),
            class_helpers
                .expect("derived expression helpers must be published")
                .inherits
                .symbol
                .into(),
        );
    }

    tx.reparent_node(plan.binding.node_id(), Some(constructor_function.node_id()));
    tx.rescope_node(plan.binding.node_id(), constructor_scope);
    tx.move_symbol(plan.symbol, iife_scope, std::iter::empty());
    let output_flags = plan.output_flags | swc_next_ecma_semantic::SymbolFlags::ANNEX_B_FUNCTION;
    tx.update_declaration_flags(plan.declaration, output_flags);
    tx.update_symbol_flags(plan.symbol, output_flags);

    for method in lowered_methods {
        method.register(
            tx,
            iife_body.node_id(),
            ScopeHandle::Pending(iife_scope),
            SymbolHandle::Existing(plan.symbol),
        );
    }
    if let Some((statement, sentinel_reference, class_reference)) = tdz_assignment {
        tx.register_subtree(statement, Some(iife_body.node_id()), iife_scope);
        tx.reference(
            iife_scope,
            sentinel_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(sentinel_reference.name(tx.ast())),
            ReferenceFlags::default().write_only(),
            Some(sentinel_symbol.expect("class TDZ sentinel symbol must be declared")),
        );
        tx.reference(
            iife_scope,
            class_reference.node_id(),
            tx.ast().canonicalize_utf8(class_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(plan.symbol),
        );
    }
    let capture_parent = tdz_wrapper
        .as_ref()
        .map_or(plan.class.node_id(), |_| call.node_id());
    let capture_evaluation_scope = outer_iife_scope
        .map(ScopeHandle::Pending)
        .unwrap_or(evaluation_scope);
    for (capture, capture_symbol) in captures.into_iter().zip(capture_symbols) {
        tx.reparent_node(capture.expression.node_id(), Some(capture_parent));
        tx.rescope_adopted_subtree(capture.expression.node_id(), capture_evaluation_scope);
        tx.reference(
            iife_scope,
            capture.reference.node_id(),
            tx.ast().canonicalize_utf8(capture.name.text),
            ReferenceFlags::default(),
            Some(capture_symbol),
        );
    }
    tx.register_subtree(
        Stmt::ReturnStatement(return_statement),
        Some(iife_body.node_id()),
        iife_scope,
    );
    tx.reference(
        iife_scope,
        return_reference.node_id(),
        tx.ast().canonicalize_utf8(return_reference.name(tx.ast())),
        ReferenceFlags::default(),
        Some(plan.symbol),
    );
    tx.retire_node(plan.body.node_id());
    tx.retire_scope(plan.class_scope);
    tx.retire_scope(plan.name_scope);
    capture_evaluation_scope
}

#[derive(Clone, Copy)]
struct ComputedKeyCapture {
    expression: Expr,
    parameter: swc_next_ecma_ast::FormalParameterItem,
    binding: swc_next_ecma_ast::BindingIdentifier,
    reference: IdentifierReference,
    name: crate::mutation::ReservedPrivateName,
}

struct DefinitionTdzReplacement {
    original: Expr,
    replacement: Expr,
    original_reference: swc_next_ecma_ast::ReferenceId,
    original_scope: swc_next_ecma_ast::ScopeId,
    references: Vec<GeneratedHeritageTdzReference>,
}

struct ClassNameTdzWrapper {
    call: swc_next_ecma_ast::CallExpression,
    function: Function,
    params: swc_next_ecma_ast::FormalParameters,
    body: FunctionBody,
    return_statement: swc_next_ecma_ast::ReturnStatement,
    helper_reference: IdentifierReference,
}

#[derive(Clone, Copy)]
enum GeneratedHeritageTdzTarget {
    Helper(PrivateBinding),
    Sentinel,
}

#[derive(Clone, Copy)]
struct GeneratedHeritageTdzReference {
    reference: IdentifierReference,
    target: GeneratedHeritageTdzTarget,
}

struct DefinitionTdzBuilder<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    sentinel: crate::mutation::ReservedPrivateName,
    class_name: AstUtf8,
    helper: PrivateBinding,
    references: Vec<GeneratedHeritageTdzReference>,
}

impl DefinitionTdzBuilder<'_, '_> {
    fn temporal_reference(&mut self) -> Expr {
        let helper = self
            .ast
            .identifier_reference(swc_next_ecma_ast::DUMMY_SP, self.helper.text);
        self.ast
            .set_syntax_context(helper.node_id(), self.helper.context);
        self.references.push(GeneratedHeritageTdzReference {
            reference: helper,
            target: GeneratedHeritageTdzTarget::Helper(self.helper),
        });
        let sentinel = self
            .ast
            .identifier_reference(swc_next_ecma_ast::DUMMY_SP, self.sentinel.text);
        self.ast
            .set_syntax_context(sentinel.node_id(), self.sentinel.context);
        self.references.push(GeneratedHeritageTdzReference {
            reference: sentinel,
            target: GeneratedHeritageTdzTarget::Sentinel,
        });
        let class_name = self.ast.get_utf8(self.class_name).to_owned();
        let class_name = self.ast.add_wtf8((&*class_name).into());
        let raw = self.ast.empty_utf8();
        let class_name = self
            .ast
            .expr_string_literal(swc_next_ecma_ast::DUMMY_SP, class_name, raw);
        let arguments = self.ast.add_typed_sub_range([
            Argument::Expr(Expr::IdentifierReference(sentinel)),
            Argument::Expr(class_name),
        ]);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(helper),
            arguments,
            false,
            None,
        )
    }
}

fn prepare_definition_tdz_replacement(
    reference_node: NodeId,
    sentinel: crate::mutation::ReservedPrivateName,
    class_name: AstUtf8,
    helper: PrivateBinding,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) -> DefinitionTdzReplacement {
    let swc_next_ecma_ast::NodeKindData::IdentifierReference(original) =
        tx.ast().kind_data(reference_node)
    else {
        unreachable!("class TDZ fact points at an identifier reference")
    };
    let original_reference = tx
        .semantic()
        .reference_of(reference_node)
        .expect("class TDZ identifier has a semantic reference");
    let original_scope = tx.semantic().scope_of(reference_node);
    let (replacement, references) = tx.build_detached(|ast| {
        let mut builder = DefinitionTdzBuilder {
            ast,
            sentinel,
            class_name,
            helper,
            references: Vec::new(),
        };
        let replacement = builder.temporal_reference();
        (replacement, builder.references)
    });
    DefinitionTdzReplacement {
        original: Expr::IdentifierReference(original),
        replacement,
        original_reference,
        original_scope,
        references,
    }
}

fn capture_expression(
    expression: Expr,
    base: &str,
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
    allocation_scope: swc_next_ecma_ast::ScopeId,
    reservation_scope: ScopeHandle,
) -> ComputedKeyCapture {
    let name = tx.reserve_private_name_in(allocation_scope, reservation_scope, base);
    let (parameter, binding, reference) = tx.build_detached(|ast| {
        let pattern = ast.formal_parameter_pattern_binding_pattern_binding_identifier(
            ast.span(expression.node_id()),
            name.text,
            TypedSubRange::empty(),
            None,
            false,
        );
        let binding = pattern
            .as_binding_pattern(ast)
            .and_then(|pattern| pattern.as_binding_identifier(ast))
            .expect("expression capture is an identifier parameter");
        ast.set_syntax_context(binding.node_id(), name.context);
        let parameter =
            ast.formal_parameter_item_formal_parameter(ast.span(expression.node_id()), pattern);
        let reference = ast.identifier_reference(ast.span(expression.node_id()), name.text);
        ast.set_syntax_context(reference.node_id(), name.context);
        (parameter, binding, reference)
    });
    ComputedKeyCapture {
        expression,
        parameter,
        binding,
        reference,
        name,
    }
}

fn capture_computed_keys(
    methods: &mut [MethodGroupPlan],
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
    allocation_scope: swc_next_ecma_ast::ScopeId,
    reservation_scope: ScopeHandle,
) -> Vec<ComputedKeyCapture> {
    fn capture(
        method: &mut MethodPlan,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        allocation_scope: swc_next_ecma_ast::ScopeId,
        reservation_scope: ScopeHandle,
    ) -> Option<ComputedKeyCapture> {
        let MethodKeyPlan::Computed(expression) = method.key else {
            return None;
        };
        let capture =
            capture_expression(expression, "_key", tx, allocation_scope, reservation_scope);
        let reference = capture.reference;
        method.key = MethodKeyPlan::CapturedComputed(reference);
        Some(capture)
    }

    let mut captures = Vec::new();
    for method in methods {
        match method {
            MethodGroupPlan::Data(method) => {
                captures.extend(capture(method, tx, allocation_scope, reservation_scope));
            }
            MethodGroupPlan::Accessor { get, set } => {
                if let Some(method) = get {
                    captures.extend(capture(method, tx, allocation_scope, reservation_scope));
                }
                if let Some(method) = set {
                    captures.extend(capture(method, tx, allocation_scope, reservation_scope));
                }
            }
        }
    }
    captures
}

struct LoweredMethod {
    statement: Stmt,
    methods: Vec<PreservedMethod>,
    object_reference: IdentifierReference,
    class_reference: IdentifierReference,
}

struct PreservedMethod {
    function: Function,
    replacement_function: Function,
    body: swc_next_ecma_ast::FunctionBody,
    replacement_body: swc_next_ecma_ast::FunctionBody,
    strict_directive: swc_next_ecma_ast::Directive,
    definition: swc_next_ecma_ast::MethodDefinition,
    retired_key: Option<swc_next_ecma_ast::NodeId>,
    moved_key: Option<swc_next_ecma_ast::NodeId>,
    scope: swc_next_ecma_ast::ScopeId,
    r#static: bool,
    super_rewrite: Option<MethodSuperRewrite>,
}

struct MethodSuperRewrite {
    clone: crate::mutation::DetachedClone<FunctionBody>,
    references: Vec<GeneratedSuperReference>,
}

#[derive(Clone, Copy)]
enum GeneratedSuperReferenceTarget {
    Class,
    Helper(PrivateBinding),
}

#[derive(Clone, Copy)]
struct GeneratedSuperReference {
    reference: IdentifierReference,
    target: GeneratedSuperReferenceTarget,
    flags: ReferenceFlags,
}

#[derive(Clone, Copy, Default)]
struct SuperWriteTemps {
    base: Option<PrivateBinding>,
    property: Option<PrivateBinding>,
    value: Option<PrivateBinding>,
    old: Option<PrivateBinding>,
}

#[derive(Default)]
struct PreparedSuperWriteTemps {
    by_member: std::collections::HashMap<NodeId, SuperWriteTemps>,
    declarations: Vec<PrivateBinding>,
}

struct RepeatedSuperAccess {
    prefix: Vec<Expr>,
    get_base: Expr,
    get_property: Expr,
    set_base: Expr,
    set_property: Expr,
}

fn compound_binary_operator(operator: AssignmentOperator) -> Option<BinaryOperator> {
    Some(match operator {
        AssignmentOperator::AddAssign => BinaryOperator::Add,
        AssignmentOperator::SubtractAssign => BinaryOperator::Subtract,
        AssignmentOperator::MultiplyAssign => BinaryOperator::Multiply,
        AssignmentOperator::DivideAssign => BinaryOperator::Divide,
        AssignmentOperator::ModuloAssign => BinaryOperator::Modulo,
        AssignmentOperator::ExponentAssign => BinaryOperator::Exponent,
        AssignmentOperator::LeftShiftAssign => BinaryOperator::LeftShift,
        AssignmentOperator::RightShiftAssign => BinaryOperator::RightShift,
        AssignmentOperator::UnsignedRightShiftAssign => BinaryOperator::UnsignedRightShift,
        AssignmentOperator::BitwiseOrAssign => BinaryOperator::BitwiseOr,
        AssignmentOperator::BitwiseXorAssign => BinaryOperator::BitwiseXor,
        AssignmentOperator::BitwiseAndAssign => BinaryOperator::BitwiseAnd,
        _ => return None,
    })
}

fn compound_logical_operator(operator: AssignmentOperator) -> Option<LogicalOperator> {
    Some(match operator {
        AssignmentOperator::LogicalOrAssign => LogicalOperator::Or,
        AssignmentOperator::LogicalAndAssign => LogicalOperator::And,
        AssignmentOperator::NullishAssign => LogicalOperator::NullishCoalescing,
        _ => return None,
    })
}

fn prepare_super_write_temps(
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
    clone: &crate::mutation::DetachedClone<FunctionBody>,
    super_members: &[NodeId],
    scope: swc_next_ecma_ast::ScopeId,
) -> PreparedSuperWriteTemps {
    #[derive(Clone, Copy)]
    struct Needs {
        clone: NodeId,
        property: bool,
        value: bool,
        old: bool,
    }

    let needs: Vec<_> = super_members
        .iter()
        .filter_map(|member| {
            let cloned = clone.clone_of(*member)?;
            let parent = tx.semantic().parent_of(*member)?;
            let (value, old) = match tx.ast().kind_data(parent) {
                swc_next_ecma_ast::NodeKindData::AssignmentExpression(assignment)
                    if assignment.operator(tx.ast()) != AssignmentOperator::Assign =>
                {
                    (
                        matches!(
                            assignment.operator(tx.ast()),
                            AssignmentOperator::LogicalOrAssign
                                | AssignmentOperator::LogicalAndAssign
                                | AssignmentOperator::NullishAssign
                        ),
                        false,
                    )
                }
                swc_next_ecma_ast::NodeKindData::UpdateExpression(update) => {
                    (true, !update.prefix(tx.ast()))
                }
                _ => return None,
            };
            let member = match tx.ast().kind_data(*member) {
                swc_next_ecma_ast::NodeKindData::MemberExpression(member) => member,
                _ => return None,
            };
            Some(Needs {
                clone: cloned,
                property: member.computed(tx.ast()),
                value,
                old,
            })
        })
        .collect();

    let mut prepared = PreparedSuperWriteTemps::default();
    for needs in needs {
        let base = tx.declare_private(scope, "_super_base", SymbolFlags::FUNCTION_SCOPED_VAR);
        prepared.declarations.push(base);
        let property = needs.property.then(|| {
            let binding =
                tx.declare_private(scope, "_super_prop", SymbolFlags::FUNCTION_SCOPED_VAR);
            prepared.declarations.push(binding);
            binding
        });
        let value = needs.value.then(|| {
            let binding =
                tx.declare_private(scope, "_super_value", SymbolFlags::FUNCTION_SCOPED_VAR);
            prepared.declarations.push(binding);
            binding
        });
        let old = needs.old.then(|| {
            let binding = tx.declare_private(scope, "_super_old", SymbolFlags::FUNCTION_SCOPED_VAR);
            prepared.declarations.push(binding);
            binding
        });
        prepared.by_member.insert(
            needs.clone,
            SuperWriteTemps {
                base: Some(base),
                property,
                value,
                old,
            },
        );
    }
    prepared
}

fn prepend_private_declarations(
    ast: &mut Ast<'_>,
    body: FunctionBody,
    bindings: &[PrivateBinding],
) {
    if bindings.is_empty() {
        return;
    }
    let declarators: Vec<_> = bindings
        .iter()
        .map(|binding| {
            ast.variable_declarator(
                swc_next_ecma_ast::DUMMY_SP,
                BindingPattern::BindingIdentifier(binding.declaration),
                None,
                false,
            )
        })
        .collect();
    let declarators = ast.add_typed_sub_range(declarators);
    let declaration = ast.variable_declaration(
        swc_next_ecma_ast::DUMMY_SP,
        VariableKind::Var,
        declarators,
        false,
    );
    let statements: Vec<_> = body
        .body(ast)
        .iter()
        .map(|id| ast.get_node_in_sub_range(id))
        .collect();
    let statements = ast.add_typed_sub_range(
        std::iter::once(Stmt::Declaration(Decl::VariableDeclaration(declaration)))
            .chain(statements),
    );
    body.set_body(ast, statements);
}

struct SuperMemberRewriteCtx<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    members: HashSet<NodeId>,
    class_name: AstUtf8,
    class_context: SyntaxContext,
    r#static: bool,
    helpers: ClassHelpers,
    write_temps: std::collections::HashMap<NodeId, SuperWriteTemps>,
    references: Vec<GeneratedSuperReference>,
}

impl<'a, 'ast> SuperMemberRewriteCtx<'a, 'ast> {
    fn rewrite_expr(&mut self, expr: Expr) -> Expr {
        // Check if this expression itself is a super member rewrite target
        if let Some(assignment) = expr.as_assignment_expression(self.ast)
            && let Some(member) = assignment
                .left(self.ast)
                .as_simple_assignment_target(self.ast)
                .and_then(|target| target.as_member_expression(self.ast))
            && self.members.contains(&member.node_id())
        {
            return if assignment.operator(self.ast) == AssignmentOperator::Assign {
                self.lower_set(member, assignment.right(self.ast))
            } else {
                self.lower_compound_assignment(assignment, member)
            };
        }
        if let Some(update) = expr.as_update_expression(self.ast)
            && let Some(member) = update.argument(self.ast).as_member_expression(self.ast)
            && self.members.contains(&member.node_id())
        {
            return self.lower_update(update, member);
        }
        if let Some(call) = expr.as_call_expression(self.ast)
            && self.members.contains(&call.callee(self.ast).node_id())
        {
            return self.lower_call(call);
        }
        if self.members.contains(&expr.node_id()) {
            let member = expr
                .as_member_expression(self.ast)
                .expect("class fact super member must remain a member expression");
            return self.lower_get(member);
        }
        self.walk_expr_children(expr)
    }

    fn walk_expr_children(&mut self, expr: Expr) -> Expr {
        use swc_next_ecma_ast::{
            ArrowFunctionBodyData, ObjectPropertyKindData,
        };
        let kind = self.ast.kind_data(expr.node_id());
        match kind {
            NodeKindData::BinaryExpression(node) => {
                let left = self.rewrite_expr(node.left(self.ast));
                node.set_left(self.ast, left);
                let right = self.rewrite_expr(node.right(self.ast));
                node.set_right(self.ast, right);
                Expr::BinaryExpression(node)
            }
            NodeKindData::LogicalExpression(node) => {
                let left = self.rewrite_expr(node.left(self.ast));
                node.set_left(self.ast, left);
                let right = self.rewrite_expr(node.right(self.ast));
                node.set_right(self.ast, right);
                Expr::LogicalExpression(node)
            }
            NodeKindData::UnaryExpression(node) => {
                let argument = self.rewrite_expr(node.argument(self.ast));
                node.set_argument(self.ast, argument);
                Expr::UnaryExpression(node)
            }
            NodeKindData::UpdateExpression(node) => Expr::UpdateExpression(node),
            NodeKindData::AssignmentExpression(node) => {
                let left = self.rewrite_assignment_target(node.left(self.ast));
                node.set_left(self.ast, left);
                let right = self.rewrite_expr(node.right(self.ast));
                node.set_right(self.ast, right);
                Expr::AssignmentExpression(node)
            }
            NodeKindData::ConditionalExpression(node) => {
                let test = self.rewrite_expr(node.test(self.ast));
                node.set_test(self.ast, test);
                let consequent = self.rewrite_expr(node.consequent(self.ast));
                node.set_consequent(self.ast, consequent);
                let alternate = self.rewrite_expr(node.alternate(self.ast));
                node.set_alternate(self.ast, alternate);
                Expr::ConditionalExpression(node)
            }
            NodeKindData::SequenceExpression(node) => {
                let rewritten: Vec<_> = node
                    .expressions(self.ast)
                    .iter()
                    .map(|id| self.rewrite_expr(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_expressions(self.ast, range);
                Expr::SequenceExpression(node)
            }
            NodeKindData::MemberExpression(node) => {
                let object = self.rewrite_expr(node.object(self.ast));
                node.set_object(self.ast, object);
                Expr::MemberExpression(node)
            }
            NodeKindData::CallExpression(node) => {
                let callee = self.rewrite_expr(node.callee(self.ast));
                node.set_callee(self.ast, callee);
                let rewritten_args: Vec<_> = node
                    .arguments(self.ast)
                    .iter()
                    .map(|id| self.rewrite_argument(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten_args);
                node.set_arguments(self.ast, range);
                Expr::CallExpression(node)
            }
            NodeKindData::ChainExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::ChainExpression(node)
            }
            NodeKindData::NewExpression(node) => {
                let callee = self.rewrite_expr(node.callee(self.ast));
                node.set_callee(self.ast, callee);
                let rewritten_args: Vec<_> = node
                    .arguments(self.ast)
                    .iter()
                    .map(|id| self.rewrite_argument(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten_args);
                node.set_arguments(self.ast, range);
                Expr::NewExpression(node)
            }
            NodeKindData::TaggedTemplateExpression(node) => {
                let tag = self.rewrite_expr(node.tag(self.ast));
                node.set_tag(self.ast, tag);
                Expr::TaggedTemplateExpression(node)
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                let body = node.body(self.ast);
                match self.ast.arrow_function_body_data(body) {
                    ArrowFunctionBodyData::FunctionBody(body) => {
                        self.walk_body(body);
                    }
                    ArrowFunctionBodyData::Expr(_) => {
                        let rewritten = self.rewrite_expr(body.as_expr(self.ast).unwrap());
                        node.set_body(self.ast, ArrowFunctionBody::Expr(rewritten));
                    }
                }
                Expr::ArrowFunctionExpression(node)
            }
            NodeKindData::ArrayExpression(node) => {
                let rewritten: Vec<_> = node
                    .elements(self.ast)
                    .iter()
                    .map(|id| {
                        let elem = self.ast.get_node_in_sub_range(id);
                        elem.map(|e| self.rewrite_argument(e))
                    })
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_elements(self.ast, range);
                Expr::ArrayExpression(node)
            }
            NodeKindData::ObjectExpression(node) => {
                let rewritten: Vec<_> = node
                    .properties(self.ast)
                    .iter()
                    .map(|id| {
                        let prop = self.ast.get_node_in_sub_range(id);
                        match self.ast.object_property_kind_data(prop) {
                            ObjectPropertyKindData::ObjectProperty(p) => {
                                let value = self.rewrite_expr(p.value(self.ast));
                                p.set_value(self.ast, value);
                                prop
                            }
                            ObjectPropertyKindData::SpreadElement(spread) => {
                                let argument = self.rewrite_expr(spread.argument(self.ast));
                                spread.set_argument(self.ast, argument);
                                prop
                            }
                        }
                    })
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_properties(self.ast, range);
                Expr::ObjectExpression(node)
            }
            NodeKindData::ParenthesizedExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::ParenthesizedExpression(node)
            }
            NodeKindData::AwaitExpression(node) => {
                let argument = self.rewrite_expr(node.argument(self.ast));
                node.set_argument(self.ast, argument);
                Expr::AwaitExpression(node)
            }
            NodeKindData::YieldExpression(node) => {
                if let Some(argument) = node.argument(self.ast) {
                    let argument = self.rewrite_expr(argument);
                    node.set_argument(self.ast, Some(argument));
                }
                Expr::YieldExpression(node)
            }
            NodeKindData::TemplateLiteral(node) => {
                let rewritten: Vec<_> = node
                    .expressions(self.ast)
                    .iter()
                    .map(|id| self.rewrite_expr(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_expressions(self.ast, range);
                Expr::TemplateLiteral(node)
            }
            NodeKindData::ImportExpression(node) => {
                let source = self.rewrite_expr(node.source(self.ast));
                node.set_source(self.ast, source);
                Expr::ImportExpression(node)
            }
            // TS type expressions
            NodeKindData::TsAsExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsAsExpression(node)
            }
            NodeKindData::TsSatisfiesExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsSatisfiesExpression(node)
            }
            NodeKindData::TsTypeAssertion(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsTypeAssertion(node)
            }
            NodeKindData::TsNonNullExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsNonNullExpression(node)
            }
            NodeKindData::TsInstantiationExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsInstantiationExpression(node)
            }
            // JSX and leaf nodes
            NodeKindData::JsxElement(_) | NodeKindData::JsxFragment(_) => expr,
            _ => expr,
        }
    }

    fn rewrite_assignment_target(&mut self, target: AssignmentTarget) -> AssignmentTarget {
        if let Some(simple) = target.as_simple_assignment_target(self.ast) {
            use swc_next_ecma_ast::SimpleAssignmentTargetData;
            match self.ast.simple_assignment_target_data(simple) {
                SimpleAssignmentTargetData::MemberExpression(member) => {
                    let object = self.rewrite_expr(member.object(self.ast));
                    member.set_object(self.ast, object);
                }
                SimpleAssignmentTargetData::TsAsExpression(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                SimpleAssignmentTargetData::TsSatisfiesExpression(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                SimpleAssignmentTargetData::TsTypeAssertion(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                SimpleAssignmentTargetData::TsNonNullExpression(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                _ => {}
            }
            target
        } else {
            target
        }
    }

    fn rewrite_argument(&mut self, argument: Argument) -> Argument {
        use swc_next_ecma_ast::ArgumentData;
        match self.ast.argument_data(argument) {
            ArgumentData::Expr(expr) => Argument::Expr(self.rewrite_expr(expr)),
            ArgumentData::SpreadElement(spread) => {
                let argument = self.rewrite_expr(spread.argument(self.ast));
                spread.set_argument(self.ast, argument);
                Argument::SpreadElement(spread)
            }
        }
    }

    fn walk_body(&mut self, body: FunctionBody) {
        let statements: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        let rewritten: Vec<_> = statements
            .into_iter()
            .map(|stmt| self.rewrite_stmt(stmt))
            .collect();
        let body_range = self.ast.add_typed_sub_range(rewritten);
        body.set_body(self.ast, body_range);
    }

    fn walk_block_statement(&mut self, block: BlockStatement) {
        let statements: Vec<_> = block
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        let rewritten: Vec<_> = statements
            .into_iter()
            .map(|stmt| self.rewrite_stmt(stmt))
            .collect();
        let body_range = self.ast.add_typed_sub_range(rewritten);
        block.set_body(self.ast, body_range);
    }

    fn rewrite_stmt(&mut self, stmt: Stmt) -> Stmt {
        if let Some(node) = stmt.as_expression_statement(self.ast) {
            let expression = self.rewrite_expr(node.expression(self.ast));
            node.set_expression(self.ast, expression);
            return Stmt::ExpressionStatement(node);
        }
        if let Some(node) = stmt.as_return_statement(self.ast) {
            if let Some(argument) = node.argument(self.ast) {
                let argument = self.rewrite_expr(argument);
                node.set_argument(self.ast, Some(argument));
            }
            return Stmt::ReturnStatement(node);
        }
        if let Some(node) = stmt.as_if_statement(self.ast) {
            let test = self.rewrite_expr(node.test(self.ast));
            node.set_test(self.ast, test);
            let consequent = self.rewrite_stmt(node.consequent(self.ast));
            node.set_consequent(self.ast, consequent);
            if let Some(alt) = node.alternate(self.ast) {
                let rewritten = self.rewrite_stmt(alt);
                node.set_alternate(self.ast, Some(rewritten));
            }
            return Stmt::IfStatement(node);
        }
        if let Some(node) = stmt.as_switch_statement(self.ast) {
            let discriminant = self.rewrite_expr(node.discriminant(self.ast));
            node.set_discriminant(self.ast, discriminant);
            let rewritten_cases: Vec<_> = node
                .cases(self.ast)
                .iter()
                .map(|id| {
                    let case = self.ast.get_node_in_sub_range(id);
                    if let Some(test) = case.test(self.ast) {
                        let rewritten = self.rewrite_expr(test);
                        case.set_test(self.ast, Some(rewritten));
                    }
                    let rewritten: Vec<_> = case
                        .consequent(self.ast)
                        .iter()
                        .map(|cid| self.rewrite_stmt(self.ast.get_node_in_sub_range(cid)))
                        .collect();
                    let range = self.ast.add_typed_sub_range(rewritten);
                    case.set_consequent(self.ast, range);
                    case
                })
                .collect();
            let range = self.ast.add_typed_sub_range(rewritten_cases);
            node.set_cases(self.ast, range);
            return Stmt::SwitchStatement(node);
        }
        if let Some(node) = stmt.as_for_statement(self.ast) {
            use swc_next_ecma_ast::ForStatementInitData;
            if let Some(init) = node.init(self.ast) {
                match self.ast.for_statement_init_data(init) {
                    ForStatementInitData::Expr(expr) => {
                        self.walk_expr_children(expr);
                    }
                    ForStatementInitData::VariableDeclaration(decl) => {
                        self.rewrite_variable_declaration(decl);
                    }
                }
            }
            if let Some(test) = node.test(self.ast) {
                let rewritten = self.rewrite_expr(test);
                node.set_test(self.ast, Some(rewritten));
            }
            if let Some(update) = node.update(self.ast) {
                let rewritten = self.rewrite_expr(update);
                node.set_update(self.ast, Some(rewritten));
            }
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::ForStatement(node);
        }
        if let Some(node) = stmt.as_for_in_statement(self.ast) {
            use swc_next_ecma_ast::ForStatementLeftData;
            match self.ast.for_statement_left_data(node.left(self.ast)) {
                ForStatementLeftData::AssignmentTarget(target) => {
                    self.rewrite_assignment_target(target);
                }
                ForStatementLeftData::VariableDeclaration(decl) => {
                    self.rewrite_variable_declaration(decl);
                }
            }
            let right = self.rewrite_expr(node.right(self.ast));
            node.set_right(self.ast, right);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::ForInStatement(node);
        }
        if let Some(node) = stmt.as_for_of_statement(self.ast) {
            use swc_next_ecma_ast::ForStatementLeftData;
            match self.ast.for_statement_left_data(node.left(self.ast)) {
                ForStatementLeftData::AssignmentTarget(target) => {
                    self.rewrite_assignment_target(target);
                }
                ForStatementLeftData::VariableDeclaration(decl) => {
                    self.rewrite_variable_declaration(decl);
                }
            }
            let right = self.rewrite_expr(node.right(self.ast));
            node.set_right(self.ast, right);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::ForOfStatement(node);
        }
        if let Some(node) = stmt.as_while_statement(self.ast) {
            let test = self.rewrite_expr(node.test(self.ast));
            node.set_test(self.ast, test);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::WhileStatement(node);
        }
        if let Some(node) = stmt.as_do_while_statement(self.ast) {
            let test = self.rewrite_expr(node.test(self.ast));
            node.set_test(self.ast, test);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::DoWhileStatement(node);
        }
        if let Some(node) = stmt.as_labeled_statement(self.ast) {
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::LabeledStatement(node);
        }
        if let Some(node) = stmt.as_try_statement(self.ast) {
            self.walk_block_statement(node.block(self.ast));
            if let Some(handler) = node.handler(self.ast) {
                self.walk_block_statement(handler.body(self.ast));
            }
            if let Some(finalizer) = node.finalizer(self.ast) {
                self.walk_block_statement(finalizer);
            }
            return Stmt::TryStatement(node);
        }
        if let Some(node) = stmt.as_throw_statement(self.ast) {
            let argument = self.rewrite_expr(node.argument(self.ast));
            node.set_argument(self.ast, argument);
            return Stmt::ThrowStatement(node);
        }
        if let Some(node) = stmt.as_with_statement(self.ast) {
            let object = self.rewrite_expr(node.object(self.ast));
            node.set_object(self.ast, object);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::WithStatement(node);
        }
        if let Some(node) = stmt.as_block_statement(self.ast) {
            self.walk_block_statement(node);
            return Stmt::BlockStatement(node);
        }
        // VariableDeclaration
        if let Some(decl) = stmt.as_declaration(self.ast)
            && let Some(var_decl) = decl.as_variable_declaration(self.ast)
        {
            return Stmt::Declaration(Decl::VariableDeclaration(
                self.rewrite_variable_declaration(var_decl),
            ));
        }
        // BreakStatement, ContinueStatement, DebuggerStatement, EmptyStatement,
        // Declaration(Function/Class/TS): no expression children to rewrite
        stmt
    }

    fn rewrite_variable_declaration(
        &mut self,
        node: swc_next_ecma_ast::VariableDeclaration,
    ) -> swc_next_ecma_ast::VariableDeclaration {
        let rewritten: Vec<_> = node
            .declarators(self.ast)
            .iter()
            .map(|id| {
                let decl = self.ast.get_node_in_sub_range(id);
                if let Some(init) = decl.init(self.ast) {
                    let rewritten = self.rewrite_expr(init);
                    decl.set_init(self.ast, Some(rewritten));
                }
                decl
            })
            .collect();
        let range = self.ast.add_typed_sub_range(rewritten);
        node.set_declarators(self.ast, range);
        node
    }
}

impl<'a, 'ast> SuperMemberRewriteCtx<'a, 'ast> {
    fn private_reference_with_flags(
        &mut self,
        binding: PrivateBinding,
        flags: ReferenceFlags,
    ) -> IdentifierReference {
        let reference = self
            .ast
            .identifier_reference(swc_next_ecma_ast::DUMMY_SP, binding.text);
        self.ast
            .set_syntax_context(reference.node_id(), binding.context);
        self.references.push(GeneratedSuperReference {
            reference,
            target: GeneratedSuperReferenceTarget::Helper(binding),
            flags,
        });
        reference
    }

    fn helper_reference(&mut self, binding: PrivateBinding) -> IdentifierReference {
        self.private_reference_with_flags(binding, ReferenceFlags::default())
    }

    fn class_reference(&mut self) -> IdentifierReference {
        let reference = self
            .ast
            .identifier_reference(swc_next_ecma_ast::DUMMY_SP, self.class_name);
        self.ast
            .set_syntax_context(reference.node_id(), self.class_context);
        self.references.push(GeneratedSuperReference {
            reference,
            target: GeneratedSuperReferenceTarget::Class,
            flags: ReferenceFlags::default(),
        });
        reference
    }

    fn property_expression(&mut self, member: swc_next_ecma_ast::MemberExpression) -> Expr {
        let key = member.property(self.ast);
        if member.computed(self.ast) {
            let expression = key
                .as_expr(self.ast)
                .expect("computed super property must be an expression");
            self.rewrite_expr(expression)
        } else {
            let identifier = key
                .as_identifier_name(self.ast)
                .expect("non-computed super property must be an identifier");
            let name = self.ast.get_utf8(identifier.name(self.ast)).to_owned();
            let value = self.ast.add_wtf8((&*name).into());
            let raw = self.ast.empty_utf8();
            self.ast
                .expr_string_literal(swc_next_ecma_ast::DUMMY_SP, value, raw)
        }
    }

    fn prototype_expression(&mut self) -> Expr {
        let class = self.class_reference();
        let target = if self.r#static {
            Expr::IdentifierReference(class)
        } else {
            let prototype = self.ast.add_utf8("prototype");
            let property = self
                .ast
                .property_key_identifier_name(swc_next_ecma_ast::DUMMY_SP, prototype);
            self.ast.expr_member_expression(
                swc_next_ecma_ast::DUMMY_SP,
                Expr::IdentifierReference(class),
                property,
                false,
                false,
            )
        };
        let helper = self.helper_reference(self.helpers.get_prototype_of);
        let arguments = self.ast.add_typed_sub_range([Argument::Expr(target)]);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(helper),
            arguments,
            false,
            None,
        )
    }

    fn lower_get(&mut self, member: swc_next_ecma_ast::MemberExpression) -> Expr {
        let property = self.property_expression(member);
        let prototype = self.prototype_expression();
        self.build_get(prototype, property)
    }

    fn build_get(&mut self, prototype: Expr, property: Expr) -> Expr {
        let helper = self.helper_reference(
            self.helpers
                .get
                .expect("super get helper must be published"),
        );
        let receiver = self.ast.expr_this_expression(swc_next_ecma_ast::DUMMY_SP);
        let arguments = self.ast.add_typed_sub_range([
            Argument::Expr(prototype),
            Argument::Expr(property),
            Argument::Expr(receiver),
        ]);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(helper),
            arguments,
            false,
            None,
        )
    }

    fn lower_call(&mut self, call: swc_next_ecma_ast::CallExpression) -> Expr {
        let member = call
            .callee(self.ast)
            .as_member_expression(self.ast)
            .expect("super method call callee must be a member expression");
        let get = self.lower_get(member);
        let call_name = self.ast.add_utf8("call");
        let call_property = self
            .ast
            .property_key_identifier_name(swc_next_ecma_ast::DUMMY_SP, call_name);
        let callee = self.ast.expr_member_expression(
            swc_next_ecma_ast::DUMMY_SP,
            get,
            call_property,
            false,
            false,
        );
        let original_arguments: Vec<_> = call
            .arguments(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        let mut arguments = Vec::with_capacity(original_arguments.len() + 1);
        arguments.push(Argument::Expr(
            self.ast.expr_this_expression(swc_next_ecma_ast::DUMMY_SP),
        ));
        arguments.extend(
            original_arguments
                .into_iter()
                .map(|argument| self.rewrite_argument(argument)),
        );
        let arguments = self.ast.add_typed_sub_range(arguments);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            callee,
            arguments,
            false,
            call.type_arguments(self.ast),
        )
    }

    fn lower_set(&mut self, member: swc_next_ecma_ast::MemberExpression, value: Expr) -> Expr {
        let property = self.property_expression(member);
        let value = self.rewrite_expr(value);
        let prototype = self.prototype_expression();
        self.build_set(prototype, property, value)
    }

    fn build_set(&mut self, prototype: Expr, property: Expr, value: Expr) -> Expr {
        let helper = self.helper_reference(
            self.helpers
                .set
                .expect("super set helper must be published"),
        );
        let receiver = self.ast.expr_this_expression(swc_next_ecma_ast::DUMMY_SP);
        let strict = self
            .ast
            .expr_boolean_literal(swc_next_ecma_ast::DUMMY_SP, true);
        let arguments = self.ast.add_typed_sub_range([
            Argument::Expr(prototype),
            Argument::Expr(property),
            Argument::Expr(value),
            Argument::Expr(receiver),
            Argument::Expr(strict),
        ]);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(helper),
            arguments,
            false,
            None,
        )
    }

    fn private_assignment(&mut self, binding: PrivateBinding, value: Expr) -> Expr {
        let reference =
            self.private_reference_with_flags(binding, ReferenceFlags::default().write_only());
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(reference),
        );
        self.ast.expr_assignment_expression(
            swc_next_ecma_ast::DUMMY_SP,
            target,
            value,
            AssignmentOperator::Assign,
        )
    }

    fn private_read(&mut self, binding: PrivateBinding) -> Expr {
        Expr::IdentifierReference(
            self.private_reference_with_flags(binding, ReferenceFlags::default()),
        )
    }

    fn private_update_target(&mut self, binding: PrivateBinding) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::IdentifierReference(
            self.private_reference_with_flags(binding, ReferenceFlags::default().read_write()),
        )
    }

    fn repeated_access(
        &mut self,
        member: swc_next_ecma_ast::MemberExpression,
    ) -> RepeatedSuperAccess {
        let temps = self.write_temps[&member.node_id()];
        let base = temps.base.expect("repeated super access needs a base temp");
        let prototype = self.prototype_expression();
        let mut prefix = vec![self.private_assignment(base, prototype)];
        let (get_property, set_property) = if member.computed(self.ast) {
            let property = temps
                .property
                .expect("computed repeated super access needs a property temp");
            let expression = self.property_expression(member);
            prefix.push(self.private_assignment(property, expression));
            (self.private_read(property), self.private_read(property))
        } else {
            (
                self.property_expression(member),
                self.property_expression(member),
            )
        };
        RepeatedSuperAccess {
            prefix,
            get_base: self.private_read(base),
            get_property,
            set_base: self.private_read(base),
            set_property,
        }
    }

    fn sequence(&mut self, expressions: Vec<Expr>) -> Expr {
        if expressions.len() == 1 {
            expressions[0]
        } else {
            let expressions = self.ast.add_typed_sub_range(expressions);
            self.ast
                .expr_sequence_expression(swc_next_ecma_ast::DUMMY_SP, expressions)
        }
    }

    fn lower_compound_assignment(
        &mut self,
        assignment: swc_next_ecma_ast::AssignmentExpression,
        member: swc_next_ecma_ast::MemberExpression,
    ) -> Expr {
        let mut access = self.repeated_access(member);
        let get = self.build_get(access.get_base, access.get_property);
        let right = self.rewrite_expr(assignment.right(self.ast));
        let operator = assignment.operator(self.ast);
        if let Some(operator) = compound_binary_operator(operator) {
            let value =
                self.ast
                    .expr_binary_expression(swc_next_ecma_ast::DUMMY_SP, get, right, operator);
            let set = self.build_set(access.set_base, access.set_property, value);
            access.prefix.push(set);
        } else {
            let operator = compound_logical_operator(operator)
                .expect("supported compound super assignment operator");
            let value = self.write_temps[&member.node_id()]
                .value
                .expect("logical super assignment needs a value temp");
            access.prefix.push(self.private_assignment(value, get));
            let current = self.private_read(value);
            let set = self.build_set(access.set_base, access.set_property, right);
            access.prefix.push(self.ast.expr_logical_expression(
                swc_next_ecma_ast::DUMMY_SP,
                current,
                set,
                operator,
            ));
        }
        self.sequence(access.prefix)
    }

    fn lower_update(
        &mut self,
        update: swc_next_ecma_ast::UpdateExpression,
        member: swc_next_ecma_ast::MemberExpression,
    ) -> Expr {
        let mut access = self.repeated_access(member);
        let temps = self.write_temps[&member.node_id()];
        let value = temps.value.expect("super update needs a value temp");
        let get = self.build_get(access.get_base, access.get_property);
        access.prefix.push(self.private_assignment(value, get));
        let target = self.private_update_target(value);
        let operator = update.operator(self.ast);
        let prefix = update.prefix(self.ast);
        let update_value =
            self.ast
                .expr_update_expression(swc_next_ecma_ast::DUMMY_SP, target, operator, prefix);
        if prefix {
            access.prefix.push(update_value);
            let value = self.private_read(value);
            access
                .prefix
                .push(self.build_set(access.set_base, access.set_property, value));
        } else {
            let old = temps
                .old
                .expect("postfix super update needs an old-value temp");
            access
                .prefix
                .push(self.private_assignment(old, update_value));
            let value = self.private_read(value);
            access
                .prefix
                .push(self.build_set(access.set_base, access.set_property, value));
            access.prefix.push(self.private_read(old));
        }
        self.sequence(access.prefix)
    }
}

impl LoweredMethod {
    fn build(
        ast: &mut Ast<'_>,
        plan: MethodGroupPlan,
        class_name: AstUtf8,
        class_context: SyntaxContext,
    ) -> Self {
        let (plans, writable) = match plan {
            MethodGroupPlan::Data(method) => (vec![method], true),
            MethodGroupPlan::Accessor { get, set } => (get.into_iter().chain(set).collect(), false),
        };
        let first = plans[0];
        let span = ast.span(first.definition.node_id());

        let object_name = ast.add_utf8("Object");
        let object_reference = ast.identifier_reference(span, object_name);
        let define_property_name = ast.add_utf8("defineProperty");
        let define_property_key = ast.property_key_identifier_name(span, define_property_name);
        let callee = ast.expr_member_expression(
            span,
            Expr::IdentifierReference(object_reference),
            define_property_key,
            false,
            false,
        );

        let class_reference = ast.identifier_reference(span, class_name);
        ast.set_syntax_context(class_reference.node_id(), class_context);
        let target = if first.r#static {
            Expr::IdentifierReference(class_reference)
        } else {
            let prototype_name = ast.add_utf8("prototype");
            let prototype_key = ast.property_key_identifier_name(span, prototype_name);
            ast.expr_member_expression(
                span,
                Expr::IdentifierReference(class_reference),
                prototype_key,
                false,
                false,
            )
        };

        let key_expression = match first.key {
            MethodKeyPlan::Identifier(name) => {
                let value = ast.get_utf8(name).to_owned();
                let value = ast.add_wtf8((&*value).into());
                ast.expr_string_literal(span, value, ast.empty_utf8())
            }
            MethodKeyPlan::String { value, raw } => ast.expr_string_literal(span, value, raw),
            MethodKeyPlan::Numeric { kind, raw } => ast.expr_numeric_literal(span, kind, raw),
            MethodKeyPlan::Computed(expression) => expression,
            MethodKeyPlan::CapturedComputed(reference) => Expr::IdentifierReference(reference),
        };

        let mut properties = Vec::with_capacity(plans.len() + usize::from(writable) + 1);
        let mut methods = Vec::with_capacity(plans.len());
        for plan in plans {
            debug_assert_eq!(plan.r#static, first.r#static);
            let plan_key = plan.definition.key(ast);
            let (retired_key, moved_key) = match plan.key {
                MethodKeyPlan::Computed(expression) => (None, Some(expression.node_id())),
                MethodKeyPlan::CapturedComputed(_) => (None, None),
                _ => (Some(plan_key.node_id()), None),
            };
            let function = plan.function;
            let body = function.body(ast);
            let directives: Vec<_> = (0..body.directives(ast).len())
                .map(|index| body.directives(ast).get_node(ast, index).unwrap())
                .collect();
            let statements: Vec<_> = (0..body.body(ast).len())
                .map(|index| body.body(ast).get_node(ast, index).unwrap())
                .collect();
            let strict_directive = build_strict_directive(ast, span);
            let replacement_directives =
                ast.add_typed_sub_range(std::iter::once(strict_directive).chain(directives));
            let replacement_statements = ast.add_typed_sub_range(statements);
            let replacement_body = ast.function_body(
                ast.span(body.node_id()),
                replacement_directives,
                replacement_statements,
            );
            let replacement_function = ast.function(
                ast.span(function.node_id()),
                FunctionType::FunctionExpression,
                None,
                function.generator(ast),
                function.r#async(ast),
                function.params(ast),
                body,
                function.type_parameters(ast),
                function.return_type(ast),
            );
            let value_name = match plan.kind {
                MethodDefinitionKind::Method => "value",
                MethodDefinitionKind::Get => "get",
                MethodDefinitionKind::Set => "set",
                MethodDefinitionKind::Constructor => unreachable!(),
            };
            let value_name = ast.add_utf8(value_name);
            let value_key = ast.property_key_identifier_name(span, value_name);
            properties.push(ast.object_property_kind_object_property(
                span,
                value_key,
                Expr::Function(function),
                PropertyKind::Init,
                false,
                false,
                false,
            ));
            methods.push(PreservedMethod {
                function,
                replacement_function,
                body,
                replacement_body,
                strict_directive,
                definition: plan.definition,
                retired_key,
                moved_key,
                scope: plan.scope,
                r#static: plan.r#static,
                super_rewrite: None,
            });
        }
        let configurable_name = ast.add_utf8("configurable");
        let configurable_key = ast.property_key_identifier_name(span, configurable_name);
        let configurable_value = ast.expr_boolean_literal(span, true);
        let configurable = ast.object_property_kind_object_property(
            span,
            configurable_key,
            configurable_value,
            PropertyKind::Init,
            false,
            false,
            false,
        );
        if writable {
            let writable_name = ast.add_utf8("writable");
            let writable_key = ast.property_key_identifier_name(span, writable_name);
            let writable_value = ast.expr_boolean_literal(span, true);
            let writable = ast.object_property_kind_object_property(
                span,
                writable_key,
                writable_value,
                PropertyKind::Init,
                false,
                false,
                false,
            );
            properties.push(writable);
        }
        properties.push(configurable);
        let properties = ast.add_typed_sub_range::<ObjectPropertyKind, _>(properties);
        let descriptor = ast.expr_object_expression(span, properties);
        let arguments = ast.add_typed_sub_range([
            Argument::Expr(target),
            Argument::Expr(key_expression),
            Argument::Expr(descriptor),
        ]);
        let call = ast.expr_call_expression(span, callee, arguments, false, None);
        let statement = ast.stmt_expression_statement(span, call);

        Self {
            statement,
            methods,
            object_reference,
            class_reference,
        }
    }

    fn prepare_super_rewrites(
        &mut self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        super_members: &[NodeId],
        class_name: AstUtf8,
        class_context: SyntaxContext,
        helpers: ClassHelpers,
    ) {
        for method in &mut self.methods {
            let clone = tx.clone_detached(method.body);
            let members: HashSet<_> = super_members
                .iter()
                .filter_map(|member| clone.clone_of(*member))
                .collect();
            if members.is_empty() {
                continue;
            }
            let write_temps = prepare_super_write_temps(tx, &clone, super_members, method.scope);
            let temp_declarations = write_temps.declarations;
            let references = tx.build_detached(|ast| {
                let mut rewriter = SuperMemberRewriteCtx {
                    ast,
                    members,
                    class_name,
                    class_context,
                    r#static: method.r#static,
                    helpers,
                    write_temps: write_temps.by_member,
                    references: Vec::new(),
                };
                rewriter.walk_body(clone.root);
                let directives: Vec<_> = clone
                    .root
                    .directives(rewriter.ast)
                    .iter()
                    .map(|id| rewriter.ast.get_node_in_sub_range(id))
                    .collect();
                let directives = rewriter.ast.add_typed_sub_range(
                    std::iter::once(method.strict_directive).chain(directives),
                );
                clone.root.set_directives(rewriter.ast, directives);
                prepend_private_declarations(rewriter.ast, clone.root, &temp_declarations);
                rewriter.references
            });
            method.super_rewrite = Some(MethodSuperRewrite { clone, references });
        }
    }

    fn register(
        mut self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        parent: swc_next_ecma_ast::NodeId,
        scope: ScopeHandle,
        class_symbol: SymbolHandle,
    ) {
        for method in &mut self.methods {
            if let Some(rewrite) = method.super_rewrite.take() {
                let layout = tx.replace_from_transformed_clone(
                    method.body,
                    rewrite.clone,
                    method.scope,
                    &[],
                );
                for generated in rewrite.references {
                    let symbol = match generated.target {
                        GeneratedSuperReferenceTarget::Class => class_symbol,
                        GeneratedSuperReferenceTarget::Helper(helper) => helper.symbol.into(),
                    };
                    tx.reference(
                        layout.scope_of(generated.reference.node_id()),
                        generated.reference.node_id(),
                        tx.ast()
                            .canonicalize_utf8(generated.reference.name(tx.ast())),
                        generated.flags,
                        Some(symbol),
                    );
                }
            } else {
                tx.replace(
                    method.body.node_id(),
                    method.replacement_body.node_id(),
                    IdentityPolicy::Preserve,
                );
                tx.register_subtree(
                    method.strict_directive,
                    Some(method.body.node_id()),
                    method.scope,
                );
            }
            tx.replace(
                method.function.node_id(),
                method.replacement_function.node_id(),
                IdentityPolicy::Preserve,
            );
        }
        tx.adopt_generated_subtree(self.statement, Some(parent), scope);
        tx.reference(
            scope,
            self.object_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.object_reference.name(tx.ast())),
            ReferenceFlags::default(),
            None::<swc_next_ecma_ast::SymbolId>,
        );
        tx.reference(
            scope,
            self.class_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.class_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(class_symbol),
        );
        for method in self.methods {
            tx.reparent_scope(method.scope, scope);
            if let Some(key) = method.retired_key {
                tx.retire_node(key);
            }
            if let Some(key) = method.moved_key {
                tx.rescope_adopted_subtree(key, scope);
            }
            tx.retire_node(method.definition.node_id());
        }
    }
}

#[derive(Clone, Copy)]
struct ConstructorGuard {
    statement: IfStatement,
    class_reference: IdentifierReference,
    type_error_reference: IdentifierReference,
}

#[derive(Clone, Copy)]
struct ConstructorPrelude {
    directive: swc_next_ecma_ast::Directive,
    guard: ConstructorGuard,
}

#[derive(Clone, Copy)]
struct GeneratedConstructorReference {
    reference: IdentifierReference,
    symbol: SymbolHandle,
    flags: ReferenceFlags,
}

#[derive(Default)]
struct ConstructorFlowAudit {
    has_return: bool,
    labels: Vec<AstUtf8>,
}

/// Walks a function body looking for ReturnStatement and LabeledStatement
/// nodes using only typed AST accessors (no Visit/VisitMut trait).
fn audit_constructor_flow(ast: &Ast<'_>, body: FunctionBody) -> ConstructorFlowAudit {
    let mut result = ConstructorFlowAudit::default();
    for i in 0..body.body(ast).len() {
        let stmt = body.body(ast).get_node(ast, i).unwrap();
        walk_stmt_for_flow(ast, stmt, &mut result);
    }
    result
}

/// Recursively walks a statement subtree looking for return/label,
/// stopping at function/arrow boundaries.
fn walk_stmt_for_flow(ast: &Ast<'_>, stmt: Stmt, result: &mut ConstructorFlowAudit) {
    if stmt.is_return_statement(ast) {
        result.has_return = true;
        return;
    }
    if let Some(labeled) = stmt.as_labeled_statement(ast) {
        result.labels.push(labeled.label(ast).name(ast));
        walk_stmt_for_flow(ast, labeled.body(ast), result);
        return;
    }
    if let Some(block) = stmt.as_block_statement(ast) {
        for i in 0..block.body(ast).len() {
            walk_stmt_for_flow(ast, block.body(ast).get_node(ast, i).unwrap(), result);
        }
        return;
    }
    if let Some(if_stmt) = stmt.as_if_statement(ast) {
        walk_stmt_for_flow(ast, if_stmt.consequent(ast), result);
        if let Some(alt) = if_stmt.alternate(ast) {
            walk_stmt_for_flow(ast, alt, result);
        }
        return;
    }
    if let Some(switch) = stmt.as_switch_statement(ast) {
        for i in 0..switch.cases(ast).len() {
            let case = switch.cases(ast).get_node(ast, i).unwrap();
            for j in 0..case.consequent(ast).len() {
                walk_stmt_for_flow(ast, case.consequent(ast).get_node(ast, j).unwrap(), result);
            }
        }
        return;
    }
    if let Some(try_stmt) = stmt.as_try_statement(ast) {
        for i in 0..try_stmt.block(ast).body(ast).len() {
            walk_stmt_for_flow(ast, try_stmt.block(ast).body(ast).get_node(ast, i).unwrap(), result);
        }
        if let Some(handler) = try_stmt.handler(ast) {
            for i in 0..handler.body(ast).body(ast).len() {
                walk_stmt_for_flow(ast, handler.body(ast).body(ast).get_node(ast, i).unwrap(), result);
            }
        }
        if let Some(finalizer) = try_stmt.finalizer(ast) {
            for i in 0..finalizer.body(ast).len() {
                walk_stmt_for_flow(ast, finalizer.body(ast).get_node(ast, i).unwrap(), result);
            }
        }
        return;
    }
    // Loop bodies
    if let Some(while_stmt) = stmt.as_while_statement(ast) {
        walk_stmt_for_flow(ast, while_stmt.body(ast), result);
        return;
    }
    if let Some(do_while) = stmt.as_do_while_statement(ast) {
        walk_stmt_for_flow(ast, do_while.body(ast), result);
        return;
    }
    if let Some(for_stmt) = stmt.as_for_statement(ast) {
        walk_stmt_for_flow(ast, for_stmt.body(ast), result);
        return;
    }
    if let Some(for_in) = stmt.as_for_in_statement(ast) {
        walk_stmt_for_flow(ast, for_in.body(ast), result);
        return;
    }
    if let Some(for_of) = stmt.as_for_of_statement(ast) {
        walk_stmt_for_flow(ast, for_of.body(ast), result);
        return;
    }
    if let Some(with_stmt) = stmt.as_with_statement(ast) {
        walk_stmt_for_flow(ast, with_stmt.body(ast), result);
    }
}

#[derive(Clone, Copy)]
struct DerivedConstructorCompletion {
    value: PrivateBinding,
    pending: PrivateBinding,
    label: ReservedPrivateName,
}

#[derive(Clone, Copy)]
struct ConstructorControlTarget {
    label: Option<AstUtf8>,
    allows_unlabeled_break: bool,
    allows_unlabeled_continue: bool,
    finalizer_depth: usize,
}

struct DerivedConstructorRewriteCtx<'a, 'ast> {
    ast: &'a mut Ast<'ast>,
    super_calls: HashSet<NodeId>,
    super_members: HashSet<NodeId>,
    this_nodes: HashSet<NodeId>,
    class_name: AstUtf8,
    class_context: SyntaxContext,
    class_symbol: SymbolHandle,
    this_binding: PrivateBinding,
    helpers: ClassHelpers,
    write_temps: std::collections::HashMap<NodeId, SuperWriteTemps>,
    arrow_depth: usize,
    completion: Option<DerivedConstructorCompletion>,
    completion_boundary_active: bool,
    finalizer_depth: usize,
    control_targets: Vec<ConstructorControlTarget>,
    references: Vec<GeneratedConstructorReference>,
}

impl<'a, 'ast> DerivedConstructorRewriteCtx<'a, 'ast> {
    fn rewrite_stmt(&mut self, stmt: Stmt) -> Stmt {
        if self.arrow_depth == 0 && self.completion_boundary_active {
            if let Some(statement) = stmt.as_return_statement(self.ast) {
                let argument = statement.argument(self.ast);
                return self.lower_constructor_return(argument);
            }
            if let Some(statement) = stmt.as_break_statement(self.ast)
                && self.abrupt_control_crosses_finalizer(statement.label(self.ast), false)
            {
                return self.clear_pending_before(stmt);
            }
            if let Some(statement) = stmt.as_continue_statement(self.ast)
                && self.abrupt_control_crosses_finalizer(statement.label(self.ast), true)
            {
                return self.clear_pending_before(stmt);
            }
        }
        self.walk_stmt_children(stmt)
    }

    fn rewrite_expr(&mut self, expr: Expr) -> Expr {
        if self.this_nodes.contains(&expr.node_id()) {
            return self.assert_this_initialized();
        }

        if let Some(assignment) = expr.as_assignment_expression(self.ast)
            && let Some(member) = assignment
                .left(self.ast)
                .as_simple_assignment_target(self.ast)
                .and_then(|target| target.as_member_expression(self.ast))
            && self.super_members.contains(&member.node_id())
        {
            return if assignment.operator(self.ast) == AssignmentOperator::Assign {
                self.lower_super_member_set(member, assignment.right(self.ast))
            } else {
                self.lower_super_member_compound_assignment(assignment, member)
            };
        }
        if let Some(update) = expr.as_update_expression(self.ast)
            && let Some(member) = update.argument(self.ast).as_member_expression(self.ast)
            && self.super_members.contains(&member.node_id())
        {
            return self.lower_super_member_update(update, member);
        }

        if let Some(call) = expr.as_call_expression(self.ast)
            && self
                .super_members
                .contains(&call.callee(self.ast).node_id())
        {
            return self.lower_super_member_call(call);
        }

        let result = self.walk_expr_children(expr);
        if self.super_calls.contains(&result.node_id()) {
            let call = result
                .as_call_expression(self.ast)
                .expect("class fact super call must remain a call expression");
            return self.lower_super_call(call);
        }
        if self.super_members.contains(&result.node_id()) {
            let member = result
                .as_member_expression(self.ast)
                .expect("class fact super member must remain a member expression");
            return self.lower_super_member_get(member);
        }
        result
    }

    fn walk_stmt_children(&mut self, stmt: Stmt) -> Stmt {
        if let Some(node) = stmt.as_expression_statement(self.ast) {
            let expression = self.rewrite_expr(node.expression(self.ast));
            node.set_expression(self.ast, expression);
            return Stmt::ExpressionStatement(node);
        }
        if let Some(node) = stmt.as_return_statement(self.ast) {
            if let Some(argument) = node.argument(self.ast) {
                let argument = self.rewrite_expr(argument);
                node.set_argument(self.ast, Some(argument));
            }
            return Stmt::ReturnStatement(node);
        }
        if let Some(node) = stmt.as_if_statement(self.ast) {
            let test = self.rewrite_expr(node.test(self.ast));
            node.set_test(self.ast, test);
            let consequent = self.rewrite_stmt(node.consequent(self.ast));
            node.set_consequent(self.ast, consequent);
            if let Some(alt) = node.alternate(self.ast) {
                let rewritten = self.rewrite_stmt(alt);
                node.set_alternate(self.ast, Some(rewritten));
            }
            return Stmt::IfStatement(node);
        }
        if let Some(node) = stmt.as_switch_statement(self.ast) {
            self.push_control_target(None, true, false);
            let discriminant = self.rewrite_expr(node.discriminant(self.ast));
            node.set_discriminant(self.ast, discriminant);
            let rewritten_cases: Vec<_> = node
                .cases(self.ast)
                .iter()
                .map(|id| {
                    let case = self.ast.get_node_in_sub_range(id);
                    if let Some(test) = case.test(self.ast) {
                        let rewritten = self.rewrite_expr(test);
                        case.set_test(self.ast, Some(rewritten));
                    }
                    let rewritten: Vec<_> = case
                        .consequent(self.ast)
                        .iter()
                        .map(|cid| self.rewrite_stmt(self.ast.get_node_in_sub_range(cid)))
                        .collect();
                    let range = self.ast.add_typed_sub_range(rewritten);
                    case.set_consequent(self.ast, range);
                    case
                })
                .collect();
            let range = self.ast.add_typed_sub_range(rewritten_cases);
            node.set_cases(self.ast, range);
            self.control_targets.pop();
            return Stmt::SwitchStatement(node);
        }
        if let Some(node) = stmt.as_for_statement(self.ast) {
            self.push_control_target(None, true, true);
            use swc_next_ecma_ast::ForStatementInitData;
            if let Some(init) = node.init(self.ast) {
                match self.ast.for_statement_init_data(init) {
                    ForStatementInitData::Expr(expr) => {
                        self.rewrite_expr(expr);
                    }
                    ForStatementInitData::VariableDeclaration(decl) => {
                        self.rewrite_variable_declaration(decl);
                    }
                }
            }
            if let Some(test) = node.test(self.ast) {
                let rewritten = self.rewrite_expr(test);
                node.set_test(self.ast, Some(rewritten));
            }
            if let Some(update) = node.update(self.ast) {
                let rewritten = self.rewrite_expr(update);
                node.set_update(self.ast, Some(rewritten));
            }
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            self.control_targets.pop();
            return Stmt::ForStatement(node);
        }
        if let Some(node) = stmt.as_for_in_statement(self.ast) {
            self.push_control_target(None, true, true);
            use swc_next_ecma_ast::ForStatementLeftData;
            match self.ast.for_statement_left_data(node.left(self.ast)) {
                ForStatementLeftData::AssignmentTarget(target) => {
                    self.rewrite_assignment_target(target);
                }
                ForStatementLeftData::VariableDeclaration(decl) => {
                    self.rewrite_variable_declaration(decl);
                }
            }
            let right = self.rewrite_expr(node.right(self.ast));
            node.set_right(self.ast, right);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            self.control_targets.pop();
            return Stmt::ForInStatement(node);
        }
        if let Some(node) = stmt.as_for_of_statement(self.ast) {
            self.push_control_target(None, true, true);
            use swc_next_ecma_ast::ForStatementLeftData;
            match self.ast.for_statement_left_data(node.left(self.ast)) {
                ForStatementLeftData::AssignmentTarget(target) => {
                    self.rewrite_assignment_target(target);
                }
                ForStatementLeftData::VariableDeclaration(decl) => {
                    self.rewrite_variable_declaration(decl);
                }
            }
            let right = self.rewrite_expr(node.right(self.ast));
            node.set_right(self.ast, right);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            self.control_targets.pop();
            return Stmt::ForOfStatement(node);
        }
        if let Some(node) = stmt.as_while_statement(self.ast) {
            self.push_control_target(None, true, true);
            let test = self.rewrite_expr(node.test(self.ast));
            node.set_test(self.ast, test);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            self.control_targets.pop();
            return Stmt::WhileStatement(node);
        }
        if let Some(node) = stmt.as_do_while_statement(self.ast) {
            self.push_control_target(None, true, true);
            let test = self.rewrite_expr(node.test(self.ast));
            node.set_test(self.ast, test);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            self.control_targets.pop();
            return Stmt::DoWhileStatement(node);
        }
        if let Some(node) = stmt.as_labeled_statement(self.ast) {
            let label = node.label(self.ast).name(self.ast);
            let allows_continue = self.labeled_statement_targets_iteration(node);
            self.push_control_target(Some(label), false, allows_continue);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            self.control_targets.pop();
            return Stmt::LabeledStatement(node);
        }
        if let Some(node) = stmt.as_try_statement(self.ast) {
            self.walk_block_statement(node.block(self.ast));
            if let Some(handler) = node.handler(self.ast) {
                self.walk_block_statement(handler.body(self.ast));
            }
            if let Some(finalizer) = node.finalizer(self.ast) {
                self.finalizer_depth += 1;
                self.walk_block_statement(finalizer);
                self.finalizer_depth -= 1;
            }
            return Stmt::TryStatement(node);
        }
        if let Some(node) = stmt.as_throw_statement(self.ast) {
            let argument = self.rewrite_expr(node.argument(self.ast));
            node.set_argument(self.ast, argument);
            return Stmt::ThrowStatement(node);
        }
        if let Some(node) = stmt.as_with_statement(self.ast) {
            let object = self.rewrite_expr(node.object(self.ast));
            node.set_object(self.ast, object);
            let body = self.rewrite_stmt(node.body(self.ast));
            node.set_body(self.ast, body);
            return Stmt::WithStatement(node);
        }
        if let Some(node) = stmt.as_block_statement(self.ast) {
            self.walk_block_statement(node);
            return Stmt::BlockStatement(node);
        }
        // VariableDeclaration
        if let Some(decl) = stmt.as_declaration(self.ast)
            && let Some(var_decl) = decl.as_variable_declaration(self.ast)
        {
            return Stmt::Declaration(Decl::VariableDeclaration(
                self.rewrite_variable_declaration(var_decl),
            ));
        }
        // BreakStatement, ContinueStatement, DebuggerStatement, EmptyStatement,
        // Declaration(Function/Class/TS): no expression children to rewrite
        stmt
    }

    fn walk_expr_children(&mut self, expr: Expr) -> Expr {
        use swc_next_ecma_ast::{ArrowFunctionBodyData, ObjectPropertyKindData};
        let kind = self.ast.kind_data(expr.node_id());
        match kind {
            NodeKindData::BinaryExpression(node) => {
                let left = self.rewrite_expr(node.left(self.ast));
                node.set_left(self.ast, left);
                let right = self.rewrite_expr(node.right(self.ast));
                node.set_right(self.ast, right);
                Expr::BinaryExpression(node)
            }
            NodeKindData::LogicalExpression(node) => {
                let left = self.rewrite_expr(node.left(self.ast));
                node.set_left(self.ast, left);
                let right = self.rewrite_expr(node.right(self.ast));
                node.set_right(self.ast, right);
                Expr::LogicalExpression(node)
            }
            NodeKindData::UnaryExpression(node) => {
                let argument = self.rewrite_expr(node.argument(self.ast));
                node.set_argument(self.ast, argument);
                Expr::UnaryExpression(node)
            }
            NodeKindData::UpdateExpression(node) => Expr::UpdateExpression(node),
            NodeKindData::AssignmentExpression(node) => {
                let left = self.rewrite_assignment_target(node.left(self.ast));
                node.set_left(self.ast, left);
                let right = self.rewrite_expr(node.right(self.ast));
                node.set_right(self.ast, right);
                Expr::AssignmentExpression(node)
            }
            NodeKindData::ConditionalExpression(node) => {
                let test = self.rewrite_expr(node.test(self.ast));
                node.set_test(self.ast, test);
                let consequent = self.rewrite_expr(node.consequent(self.ast));
                node.set_consequent(self.ast, consequent);
                let alternate = self.rewrite_expr(node.alternate(self.ast));
                node.set_alternate(self.ast, alternate);
                Expr::ConditionalExpression(node)
            }
            NodeKindData::SequenceExpression(node) => {
                let rewritten: Vec<_> = node
                    .expressions(self.ast)
                    .iter()
                    .map(|id| self.rewrite_expr(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_expressions(self.ast, range);
                Expr::SequenceExpression(node)
            }
            NodeKindData::MemberExpression(node) => {
                let object = self.rewrite_expr(node.object(self.ast));
                node.set_object(self.ast, object);
                Expr::MemberExpression(node)
            }
            NodeKindData::CallExpression(node) => {
                let callee = self.rewrite_expr(node.callee(self.ast));
                node.set_callee(self.ast, callee);
                let rewritten_args: Vec<_> = node
                    .arguments(self.ast)
                    .iter()
                    .map(|id| self.rewrite_argument(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten_args);
                node.set_arguments(self.ast, range);
                Expr::CallExpression(node)
            }
            NodeKindData::ChainExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::ChainExpression(node)
            }
            NodeKindData::NewExpression(node) => {
                let callee = self.rewrite_expr(node.callee(self.ast));
                node.set_callee(self.ast, callee);
                let rewritten_args: Vec<_> = node
                    .arguments(self.ast)
                    .iter()
                    .map(|id| self.rewrite_argument(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten_args);
                node.set_arguments(self.ast, range);
                Expr::NewExpression(node)
            }
            NodeKindData::TaggedTemplateExpression(node) => {
                let tag = self.rewrite_expr(node.tag(self.ast));
                node.set_tag(self.ast, tag);
                Expr::TaggedTemplateExpression(node)
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                self.arrow_depth += 1;
                let body = node.body(self.ast);
                match self.ast.arrow_function_body_data(body) {
                    ArrowFunctionBodyData::FunctionBody(body) => {
                        self.walk_body(body);
                    }
                    ArrowFunctionBodyData::Expr(_) => {
                        let rewritten = self.rewrite_expr(body.as_expr(self.ast).unwrap());
                        node.set_body(self.ast, ArrowFunctionBody::Expr(rewritten));
                    }
                }
                self.arrow_depth -= 1;
                Expr::ArrowFunctionExpression(node)
            }
            NodeKindData::Function(_) => expr,
            NodeKindData::ArrayExpression(node) => {
                let rewritten: Vec<_> = node
                    .elements(self.ast)
                    .iter()
                    .map(|id| {
                        let elem = self.ast.get_node_in_sub_range(id);
                        elem.map(|e| self.rewrite_argument(e))
                    })
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_elements(self.ast, range);
                Expr::ArrayExpression(node)
            }
            NodeKindData::ObjectExpression(node) => {
                let rewritten: Vec<_> = node
                    .properties(self.ast)
                    .iter()
                    .map(|id| {
                        let prop = self.ast.get_node_in_sub_range(id);
                        match self.ast.object_property_kind_data(prop) {
                            ObjectPropertyKindData::ObjectProperty(p) => {
                                let value = self.rewrite_expr(p.value(self.ast));
                                p.set_value(self.ast, value);
                                prop
                            }
                            ObjectPropertyKindData::SpreadElement(spread) => {
                                let argument = self.rewrite_expr(spread.argument(self.ast));
                                spread.set_argument(self.ast, argument);
                                prop
                            }
                        }
                    })
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_properties(self.ast, range);
                Expr::ObjectExpression(node)
            }
            NodeKindData::ParenthesizedExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::ParenthesizedExpression(node)
            }
            NodeKindData::AwaitExpression(node) => {
                let argument = self.rewrite_expr(node.argument(self.ast));
                node.set_argument(self.ast, argument);
                Expr::AwaitExpression(node)
            }
            NodeKindData::YieldExpression(node) => {
                if let Some(argument) = node.argument(self.ast) {
                    let argument = self.rewrite_expr(argument);
                    node.set_argument(self.ast, Some(argument));
                }
                Expr::YieldExpression(node)
            }
            NodeKindData::TemplateLiteral(node) => {
                let rewritten: Vec<_> = node
                    .expressions(self.ast)
                    .iter()
                    .map(|id| self.rewrite_expr(self.ast.get_node_in_sub_range(id)))
                    .collect();
                let range = self.ast.add_typed_sub_range(rewritten);
                node.set_expressions(self.ast, range);
                Expr::TemplateLiteral(node)
            }
            NodeKindData::ImportExpression(node) => {
                let source = self.rewrite_expr(node.source(self.ast));
                node.set_source(self.ast, source);
                Expr::ImportExpression(node)
            }
            // TS type expressions
            NodeKindData::TsAsExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsAsExpression(node)
            }
            NodeKindData::TsSatisfiesExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsSatisfiesExpression(node)
            }
            NodeKindData::TsTypeAssertion(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsTypeAssertion(node)
            }
            NodeKindData::TsNonNullExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsNonNullExpression(node)
            }
            NodeKindData::TsInstantiationExpression(node) => {
                let expression = self.rewrite_expr(node.expression(self.ast));
                node.set_expression(self.ast, expression);
                Expr::TsInstantiationExpression(node)
            }
            // JSX and leaf nodes
            NodeKindData::JsxElement(_) | NodeKindData::JsxFragment(_) => expr,
            _ => expr,
        }
    }

    fn walk_body(&mut self, body: FunctionBody) {
        let statements: Vec<_> = body
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        let rewritten: Vec<_> = statements
            .into_iter()
            .map(|stmt| self.rewrite_stmt(stmt))
            .collect();
        let body_range = self.ast.add_typed_sub_range(rewritten);
        body.set_body(self.ast, body_range);
    }

    fn walk_block_statement(&mut self, block: BlockStatement) {
        let statements: Vec<_> = block
            .body(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        let rewritten: Vec<_> = statements
            .into_iter()
            .map(|stmt| self.rewrite_stmt(stmt))
            .collect();
        let body_range = self.ast.add_typed_sub_range(rewritten);
        block.set_body(self.ast, body_range);
    }

    fn rewrite_assignment_target(&mut self, target: AssignmentTarget) -> AssignmentTarget {
        if let Some(simple) = target.as_simple_assignment_target(self.ast) {
            use swc_next_ecma_ast::SimpleAssignmentTargetData;
            match self.ast.simple_assignment_target_data(simple) {
                SimpleAssignmentTargetData::MemberExpression(member) => {
                    let object = self.rewrite_expr(member.object(self.ast));
                    member.set_object(self.ast, object);
                }
                SimpleAssignmentTargetData::TsAsExpression(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                SimpleAssignmentTargetData::TsSatisfiesExpression(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                SimpleAssignmentTargetData::TsTypeAssertion(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                SimpleAssignmentTargetData::TsNonNullExpression(node) => {
                    let expression = self.rewrite_expr(node.expression(self.ast));
                    node.set_expression(self.ast, expression);
                }
                _ => {}
            }
            target
        } else {
            target
        }
    }

    fn rewrite_argument(&mut self, argument: Argument) -> Argument {
        use swc_next_ecma_ast::ArgumentData;
        match self.ast.argument_data(argument) {
            ArgumentData::Expr(expr) => Argument::Expr(self.rewrite_expr(expr)),
            ArgumentData::SpreadElement(spread) => {
                let argument = self.rewrite_expr(spread.argument(self.ast));
                spread.set_argument(self.ast, argument);
                Argument::SpreadElement(spread)
            }
        }
    }

    fn rewrite_variable_declaration(
        &mut self,
        node: swc_next_ecma_ast::VariableDeclaration,
    ) -> swc_next_ecma_ast::VariableDeclaration {
        let rewritten: Vec<_> = node
            .declarators(self.ast)
            .iter()
            .map(|id| {
                let decl = self.ast.get_node_in_sub_range(id);
                if let Some(init) = decl.init(self.ast) {
                    let rewritten = self.rewrite_expr(init);
                    decl.set_init(self.ast, Some(rewritten));
                }
                decl
            })
            .collect();
        let range = self.ast.add_typed_sub_range(rewritten);
        node.set_declarators(self.ast, range);
        node
    }
}

impl<'a, 'ast> DerivedConstructorRewriteCtx<'a, 'ast> {
    fn push_control_target(
        &mut self,
        label: Option<AstUtf8>,
        allows_unlabeled_break: bool,
        allows_unlabeled_continue: bool,
    ) {
        self.control_targets.push(ConstructorControlTarget {
            label,
            allows_unlabeled_break,
            allows_unlabeled_continue,
            finalizer_depth: self.finalizer_depth,
        });
    }

    fn labeled_statement_targets_iteration(
        &self,
        statement: swc_next_ecma_ast::LabeledStatement,
    ) -> bool {
        let mut body = statement.body(self.ast);
        loop {
            if body.is_for_statement(self.ast)
                || body.is_for_in_statement(self.ast)
                || body.is_for_of_statement(self.ast)
                || body.is_while_statement(self.ast)
                || body.is_do_while_statement(self.ast)
            {
                return true;
            }
            let Some(label) = body.as_labeled_statement(self.ast) else {
                return false;
            };
            body = label.body(self.ast);
        }
    }

    fn abrupt_control_crosses_finalizer(
        &self,
        label: Option<swc_next_ecma_ast::LabelIdentifier>,
        is_continue: bool,
    ) -> bool {
        if self.finalizer_depth == 0 {
            return false;
        }
        let target = if let Some(label) = label {
            let name = label.name(self.ast);
            self.control_targets.iter().rev().find(|target| {
                target.label.is_some_and(|target_name| {
                    self.ast.get_utf8(target_name) == self.ast.get_utf8(name)
                }) && (!is_continue || target.allows_unlabeled_continue)
            })
        } else {
            self.control_targets.iter().rev().find(|target| {
                if is_continue {
                    target.allows_unlabeled_continue
                } else {
                    target.allows_unlabeled_break
                }
            })
        };
        target.is_some_and(|target| target.finalizer_depth < self.finalizer_depth)
    }

    fn completion_label_identifier(&mut self) -> swc_next_ecma_ast::LabelIdentifier {
        let completion = self
            .completion
            .expect("completion label requires constructor return state");
        let label = self
            .ast
            .label_identifier(swc_next_ecma_ast::DUMMY_SP, completion.label.text);
        self.ast
            .set_syntax_context(label.node_id(), completion.label.context);
        label
    }

    fn lower_constructor_return(&mut self, value: Option<Expr>) -> Stmt {
        let completion = self
            .completion
            .expect("constructor return lowering requires completion state");
        let value = value.unwrap_or_else(|| {
            let raw = self.ast.add_utf8("0");
            let zero = self.ast.expr_numeric_literal(
                swc_next_ecma_ast::DUMMY_SP,
                swc_next_ecma_ast::NumericLiteralKind::Decimal,
                raw,
            );
            self.ast
                .expr_unary_expression(swc_next_ecma_ast::DUMMY_SP, zero, UnaryOperator::Void)
        });
        let write_value = self.constructor_private_assignment(completion.value, value);
        let pending = self
            .ast
            .expr_boolean_literal(swc_next_ecma_ast::DUMMY_SP, true);
        let write_pending = self.constructor_private_assignment(completion.pending, pending);
        let label = self.completion_label_identifier();
        let break_statement = self
            .ast
            .stmt_break_statement(swc_next_ecma_ast::DUMMY_SP, Some(label));
        let test = self.ast.add_typed_sub_range([write_value, write_pending]);
        let test = self
            .ast
            .expr_sequence_expression(swc_next_ecma_ast::DUMMY_SP, test);
        self.ast
            .stmt_if_statement(swc_next_ecma_ast::DUMMY_SP, test, break_statement, None)
    }

    fn clear_pending_before(&mut self, abrupt: Stmt) -> Stmt {
        let completion = self
            .completion
            .expect("completion clearing requires constructor return state");
        let pending = self
            .ast
            .expr_boolean_literal(swc_next_ecma_ast::DUMMY_SP, false);
        let clear = self.constructor_private_assignment(completion.pending, pending);
        let always = self
            .ast
            .expr_boolean_literal(swc_next_ecma_ast::DUMMY_SP, true);
        let test = self.ast.add_typed_sub_range([clear, always]);
        let test = self
            .ast
            .expr_sequence_expression(swc_next_ecma_ast::DUMMY_SP, test);
        self.ast
            .stmt_if_statement(swc_next_ecma_ast::DUMMY_SP, test, abrupt, None)
    }

    fn completion_boundary(&mut self, statement: Stmt) -> [Stmt; 2] {
        let completion = self
            .completion
            .expect("completion boundary requires constructor return state");
        let label = self.completion_label_identifier();
        let body = self
            .ast
            .stmt_labeled_statement(swc_next_ecma_ast::DUMMY_SP, label, statement);
        let pending = self.private_reference(completion.pending);
        let value = self.private_reference(completion.value);
        let returned = self.possible_constructor_return(Some(Expr::IdentifierReference(value)));
        let returned = self
            .ast
            .stmt_return_statement(swc_next_ecma_ast::DUMMY_SP, Some(returned));
        let check = self.ast.stmt_if_statement(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(pending),
            returned,
            None,
        );
        [body, check]
    }

    fn private_reference(&mut self, binding: PrivateBinding) -> IdentifierReference {
        self.private_reference_with_flags(binding, ReferenceFlags::default())
    }

    fn private_reference_with_flags(
        &mut self,
        binding: PrivateBinding,
        flags: ReferenceFlags,
    ) -> IdentifierReference {
        let reference = self
            .ast
            .identifier_reference(swc_next_ecma_ast::DUMMY_SP, binding.text);
        self.ast
            .set_syntax_context(reference.node_id(), binding.context);
        self.references.push(GeneratedConstructorReference {
            reference,
            symbol: binding.symbol.into(),
            flags,
        });
        reference
    }

    fn class_reference(&mut self) -> IdentifierReference {
        let reference = self
            .ast
            .identifier_reference(swc_next_ecma_ast::DUMMY_SP, self.class_name);
        self.ast
            .set_syntax_context(reference.node_id(), self.class_context);
        self.references.push(GeneratedConstructorReference {
            reference,
            symbol: self.class_symbol,
            flags: ReferenceFlags::default(),
        });
        reference
    }

    fn call_private_helper(
        &mut self,
        helper: PrivateBinding,
        arguments: impl IntoIterator<Item = Argument>,
    ) -> Expr {
        let helper = self.private_reference(helper);
        let arguments = self.ast.add_typed_sub_range(arguments);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(helper),
            arguments,
            false,
            None,
        )
    }

    fn assert_this_initialized(&mut self) -> Expr {
        let this = self.private_reference(self.this_binding);
        self.call_private_helper(
            self.helpers.assert_this_initialized,
            [Argument::Expr(Expr::IdentifierReference(this))],
        )
    }

    fn possible_constructor_return(&mut self, value: Option<Expr>) -> Expr {
        let this = self.private_reference(self.this_binding);
        let mut arguments = vec![Argument::Expr(Expr::IdentifierReference(this))];
        arguments.extend(value.map(Argument::Expr));
        self.call_private_helper(self.helpers.possible_constructor_return, arguments)
    }

    fn lower_super_call(&mut self, call: swc_next_ecma_ast::CallExpression) -> Expr {
        let original_arguments: Vec<_> = call
            .arguments(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        let helper = self.private_reference(self.helpers.call_super);
        let class = self.class_reference();
        let mut arguments = vec![
            Argument::Expr(self.ast.expr_this_expression(swc_next_ecma_ast::DUMMY_SP)),
            Argument::Expr(Expr::IdentifierReference(class)),
        ];
        if !original_arguments.is_empty() {
            let elements = self
                .ast
                .add_typed_sub_range(original_arguments.into_iter().map(Some));
            arguments.push(Argument::Expr(
                self.ast
                    .expr_array_expression(swc_next_ecma_ast::DUMMY_SP, elements),
            ));
        }
        let arguments = self.ast.add_typed_sub_range(arguments);
        let call = self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(helper),
            arguments,
            false,
            None,
        );
        let target_reference = self.private_reference_with_flags(
            self.this_binding,
            ReferenceFlags::default().write_only(),
        );
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(target_reference),
        );
        self.ast.expr_assignment_expression(
            swc_next_ecma_ast::DUMMY_SP,
            target,
            call,
            AssignmentOperator::Assign,
        )
    }

    fn super_property_expression(&mut self, member: swc_next_ecma_ast::MemberExpression) -> Expr {
        let key = member.property(self.ast);
        if member.computed(self.ast) {
            let expression = key
                .as_expr(self.ast)
                .expect("computed super property must be an expression");
            self.rewrite_expr(expression)
        } else {
            let identifier = key
                .as_identifier_name(self.ast)
                .expect("non-computed super property must be an identifier");
            let name = self.ast.get_utf8(identifier.name(self.ast)).to_owned();
            let value = self.ast.add_wtf8((&*name).into());
            let raw = self.ast.empty_utf8();
            self.ast
                .expr_string_literal(swc_next_ecma_ast::DUMMY_SP, value, raw)
        }
    }

    fn lower_super_member_get(&mut self, member: swc_next_ecma_ast::MemberExpression) -> Expr {
        let property = self.super_property_expression(member);
        let prototype = self.super_prototype_expression();
        self.build_super_member_get(prototype, property)
    }

    fn super_prototype_expression(&mut self) -> Expr {
        let class = self.class_reference();
        let prototype_helper = self.private_reference(self.helpers.get_prototype_of);
        let prototype_arguments = self
            .ast
            .add_typed_sub_range([Argument::Expr(Expr::IdentifierReference(class))]);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(prototype_helper),
            prototype_arguments,
            false,
            None,
        )
    }

    fn build_super_member_get(&mut self, prototype: Expr, property: Expr) -> Expr {
        let get_helper = self.private_reference(
            self.helpers
                .get
                .expect("super get helper must be published"),
        );
        let receiver = self.assert_this_initialized();
        let arguments = self.ast.add_typed_sub_range([
            Argument::Expr(prototype),
            Argument::Expr(property),
            Argument::Expr(receiver),
        ]);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(get_helper),
            arguments,
            false,
            None,
        )
    }

    fn lower_super_member_call(&mut self, call: swc_next_ecma_ast::CallExpression) -> Expr {
        let member = call
            .callee(self.ast)
            .as_member_expression(self.ast)
            .expect("super method call callee must be a member expression");
        let get = self.lower_super_member_get(member);
        let call_name = self.ast.add_utf8("call");
        let call_property = self
            .ast
            .property_key_identifier_name(swc_next_ecma_ast::DUMMY_SP, call_name);
        let callee = self.ast.expr_member_expression(
            swc_next_ecma_ast::DUMMY_SP,
            get,
            call_property,
            false,
            false,
        );
        let original_arguments: Vec<_> = call
            .arguments(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect();
        let mut arguments = Vec::with_capacity(original_arguments.len() + 1);
        arguments.push(Argument::Expr(self.assert_this_initialized()));
        arguments.extend(
            original_arguments
                .into_iter()
                .map(|argument| self.rewrite_argument(argument)),
        );
        let arguments = self.ast.add_typed_sub_range(arguments);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            callee,
            arguments,
            false,
            call.type_arguments(self.ast),
        )
    }

    fn lower_super_member_set(
        &mut self,
        member: swc_next_ecma_ast::MemberExpression,
        value: Expr,
    ) -> Expr {
        let property = self.super_property_expression(member);
        let value = self.rewrite_expr(value);
        let prototype = self.super_prototype_expression();
        self.build_super_member_set(prototype, property, value)
    }

    fn build_super_member_set(&mut self, prototype: Expr, property: Expr, value: Expr) -> Expr {
        let set_helper = self.private_reference(
            self.helpers
                .set
                .expect("super set helper must be published"),
        );
        let receiver = self.assert_this_initialized();
        let strict = self
            .ast
            .expr_boolean_literal(swc_next_ecma_ast::DUMMY_SP, true);
        let arguments = self.ast.add_typed_sub_range([
            Argument::Expr(prototype),
            Argument::Expr(property),
            Argument::Expr(value),
            Argument::Expr(receiver),
            Argument::Expr(strict),
        ]);
        self.ast.expr_call_expression(
            swc_next_ecma_ast::DUMMY_SP,
            Expr::IdentifierReference(set_helper),
            arguments,
            false,
            None,
        )
    }

    fn constructor_private_assignment(&mut self, binding: PrivateBinding, value: Expr) -> Expr {
        let reference =
            self.private_reference_with_flags(binding, ReferenceFlags::default().write_only());
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(reference),
        );
        self.ast.expr_assignment_expression(
            swc_next_ecma_ast::DUMMY_SP,
            target,
            value,
            AssignmentOperator::Assign,
        )
    }

    fn constructor_private_read(&mut self, binding: PrivateBinding) -> Expr {
        Expr::IdentifierReference(
            self.private_reference_with_flags(binding, ReferenceFlags::default()),
        )
    }

    fn constructor_private_update_target(
        &mut self,
        binding: PrivateBinding,
    ) -> SimpleAssignmentTarget {
        SimpleAssignmentTarget::IdentifierReference(
            self.private_reference_with_flags(binding, ReferenceFlags::default().read_write()),
        )
    }

    fn repeated_constructor_super_access(
        &mut self,
        member: swc_next_ecma_ast::MemberExpression,
    ) -> RepeatedSuperAccess {
        let temps = self.write_temps[&member.node_id()];
        let base = temps.base.expect("repeated super access needs a base temp");
        let prototype = self.super_prototype_expression();
        let mut prefix = vec![self.constructor_private_assignment(base, prototype)];
        let (get_property, set_property) = if member.computed(self.ast) {
            let property = temps
                .property
                .expect("computed repeated super access needs a property temp");
            let expression = self.super_property_expression(member);
            prefix.push(self.constructor_private_assignment(property, expression));
            (
                self.constructor_private_read(property),
                self.constructor_private_read(property),
            )
        } else {
            (
                self.super_property_expression(member),
                self.super_property_expression(member),
            )
        };
        RepeatedSuperAccess {
            prefix,
            get_base: self.constructor_private_read(base),
            get_property,
            set_base: self.constructor_private_read(base),
            set_property,
        }
    }

    fn constructor_sequence(&mut self, expressions: Vec<Expr>) -> Expr {
        if expressions.len() == 1 {
            expressions[0]
        } else {
            let expressions = self.ast.add_typed_sub_range(expressions);
            self.ast
                .expr_sequence_expression(swc_next_ecma_ast::DUMMY_SP, expressions)
        }
    }

    fn lower_super_member_compound_assignment(
        &mut self,
        assignment: swc_next_ecma_ast::AssignmentExpression,
        member: swc_next_ecma_ast::MemberExpression,
    ) -> Expr {
        let mut access = self.repeated_constructor_super_access(member);
        let get = self.build_super_member_get(access.get_base, access.get_property);
        let right = self.rewrite_expr(assignment.right(self.ast));
        let operator = assignment.operator(self.ast);
        if let Some(operator) = compound_binary_operator(operator) {
            let value =
                self.ast
                    .expr_binary_expression(swc_next_ecma_ast::DUMMY_SP, get, right, operator);
            let set = self.build_super_member_set(access.set_base, access.set_property, value);
            access.prefix.push(set);
        } else {
            let operator = compound_logical_operator(operator)
                .expect("supported compound super assignment operator");
            let value = self.write_temps[&member.node_id()]
                .value
                .expect("logical super assignment needs a value temp");
            access
                .prefix
                .push(self.constructor_private_assignment(value, get));
            let current = self.constructor_private_read(value);
            let set = self.build_super_member_set(access.set_base, access.set_property, right);
            access.prefix.push(self.ast.expr_logical_expression(
                swc_next_ecma_ast::DUMMY_SP,
                current,
                set,
                operator,
            ));
        }
        self.constructor_sequence(access.prefix)
    }

    fn lower_super_member_update(
        &mut self,
        update: swc_next_ecma_ast::UpdateExpression,
        member: swc_next_ecma_ast::MemberExpression,
    ) -> Expr {
        let mut access = self.repeated_constructor_super_access(member);
        let temps = self.write_temps[&member.node_id()];
        let value = temps.value.expect("super update needs a value temp");
        let get = self.build_super_member_get(access.get_base, access.get_property);
        access
            .prefix
            .push(self.constructor_private_assignment(value, get));
        let target = self.constructor_private_update_target(value);
        let update_value = self.ast.expr_update_expression(
            swc_next_ecma_ast::DUMMY_SP,
            target,
            update.operator(self.ast),
            update.prefix(self.ast),
        );
        if update.prefix(self.ast) {
            access.prefix.push(update_value);
            let value = self.constructor_private_read(value);
            let set = self.build_super_member_set(access.set_base, access.set_property, value);
            access.prefix.push(set);
        } else {
            let old = temps
                .old
                .expect("postfix super update needs an old-value temp");
            access
                .prefix
                .push(self.constructor_private_assignment(old, update_value));
            let value = self.constructor_private_read(value);
            let set = self.build_super_member_set(access.set_base, access.set_property, value);
            access.prefix.push(set);
            let old = self.constructor_private_read(old);
            access.prefix.push(old);
        }
        self.constructor_sequence(access.prefix)
    }
}

#[derive(Clone, Copy)]
struct ConstructorLoweringContext {
    scope: swc_next_ecma_ast::ScopeId,
    class_name: AstUtf8,
    class_context: SyntaxContext,
    class_symbol: SymbolHandle,
    helpers: ClassHelpers,
}

fn lower_base_constructor_body(
    body: FunctionBody,
    context: ConstructorLoweringContext,
    super_members: &[NodeId],
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) -> bool {
    let ConstructorLoweringContext {
        scope,
        class_name,
        class_context,
        class_symbol,
        helpers,
    } = context;
    let clone = tx.clone_detached(body);
    let members: HashSet<_> = super_members
        .iter()
        .filter_map(|member| clone.clone_of(*member))
        .collect();
    if members.is_empty() {
        return false;
    }
    let write_temps = prepare_super_write_temps(tx, &clone, super_members, scope);
    let temp_declarations = write_temps.declarations;
    let (prelude, references) = tx.build_detached(|ast| {
        let mut rewriter = SuperMemberRewriteCtx {
            ast,
            members,
            class_name,
            class_context,
            r#static: true,
            helpers,
            write_temps: write_temps.by_member,
            references: Vec::new(),
        };
        rewriter.walk_body(clone.root);
        let prelude = ConstructorPrelude::build(
            rewriter.ast,
            swc_next_ecma_ast::DUMMY_SP,
            class_name,
            class_context,
        );
        let directives: Vec<_> = clone
            .root
            .directives(rewriter.ast)
            .iter()
            .map(|id| rewriter.ast.get_node_in_sub_range(id))
            .collect();
        let statements: Vec<_> = clone
            .root
            .body(rewriter.ast)
            .iter()
            .map(|id| rewriter.ast.get_node_in_sub_range(id))
            .collect();
        let directives = rewriter
            .ast
            .add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
        let statements = rewriter.ast.add_typed_sub_range(
            std::iter::once(Stmt::IfStatement(prelude.guard.statement)).chain(statements),
        );
        clone.root.set_directives(rewriter.ast, directives);
        clone.root.set_body(rewriter.ast, statements);
        prepend_private_declarations(rewriter.ast, clone.root, &temp_declarations);
        (prelude, rewriter.references)
    });
    let layout = tx.replace_from_transformed_clone(body, clone, scope, &[]);
    prelude.register_references(
        tx,
        layout.scope_of(prelude.guard.statement.node_id()),
        class_symbol,
    );
    for generated in references {
        let symbol = match generated.target {
            GeneratedSuperReferenceTarget::Class => class_symbol,
            GeneratedSuperReferenceTarget::Helper(helper) => helper.symbol.into(),
        };
        tx.reference(
            layout.scope_of(generated.reference.node_id()),
            generated.reference.node_id(),
            tx.ast()
                .canonicalize_utf8(generated.reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(symbol),
        );
    }
    true
}

fn lower_explicit_derived_constructor_body(
    body: FunctionBody,
    context: ConstructorLoweringContext,
    super_calls: &[NodeId],
    super_members: &[NodeId],
    this_nodes: &[NodeId],
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
) {
    let ConstructorLoweringContext {
        scope,
        class_name,
        class_context,
        class_symbol,
        helpers,
    } = context;
    let this_binding = tx.declare_private(scope, "_this", SymbolFlags::FUNCTION_SCOPED_VAR);
    let flow = audit_constructor_flow(tx.ast(), body);
    let completion = flow.has_return.then(|| {
        let value = tx.declare_private(
            scope,
            "_constructor_return_value",
            SymbolFlags::FUNCTION_SCOPED_VAR,
        );
        let pending = tx.declare_private(
            scope,
            "_constructor_return_pending",
            SymbolFlags::FUNCTION_SCOPED_VAR,
        );
        let labels: Vec<_> = flow
            .labels
            .iter()
            .map(|label| tx.ast().get_utf8(*label).to_owned())
            .collect();
        let mut label_base = "_constructor_completion".to_string();
        while labels.iter().any(|label| label.starts_with(&label_base)) {
            label_base.push('_');
        }
        let label = tx.reserve_private_name(scope, &label_base);
        DerivedConstructorCompletion {
            value,
            pending,
            label,
        }
    });
    let clone = tx.clone_detached(body);
    let write_temps = prepare_super_write_temps(tx, &clone, super_members, scope);
    let temp_declarations = write_temps.declarations;
    let cloned_super_calls = super_calls
        .iter()
        .filter_map(|node| clone.clone_of(*node))
        .collect();
    let cloned_super_members = super_members
        .iter()
        .filter_map(|node| clone.clone_of(*node))
        .collect();
    let cloned_this_nodes = this_nodes
        .iter()
        .filter_map(|node| clone.clone_of(*node))
        .collect();
    let (prelude, references) = tx.build_detached(|ast| {
        let directives: Vec<_> = clone
            .root
            .directives(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect();
        let statements: Vec<_> = clone
            .root
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect();
        let statement_returns: Vec<_> = statements
            .iter()
            .map(|statement| {
                let mut audit = ConstructorFlowAudit::default();
                walk_stmt_for_flow(ast, *statement, &mut audit);
                audit.has_return
            })
            .collect();
        let mut rewriter = DerivedConstructorRewriteCtx {
            ast,
            super_calls: cloned_super_calls,
            super_members: cloned_super_members,
            this_nodes: cloned_this_nodes,
            class_name,
            class_context,
            class_symbol,
            this_binding,
            helpers,
            write_temps: write_temps.by_member,
            arrow_depth: 0,
            completion,
            completion_boundary_active: false,
            finalizer_depth: 0,
            control_targets: Vec::new(),
            references: Vec::new(),
        };
        let mut rewritten_statements = Vec::with_capacity(statements.len() * 2);
        for (statement, has_return) in statements.into_iter().zip(statement_returns) {
            rewriter.completion_boundary_active = has_return;
            let statement = rewriter.rewrite_stmt(statement);
            rewriter.completion_boundary_active = false;
            debug_assert_eq!(rewriter.finalizer_depth, 0);
            debug_assert!(rewriter.control_targets.is_empty());
            if has_return {
                rewritten_statements.extend(rewriter.completion_boundary(statement));
            } else {
                rewritten_statements.push(statement);
            }
        }

        let prelude = ConstructorPrelude::build(
            rewriter.ast,
            swc_next_ecma_ast::DUMMY_SP,
            class_name,
            class_context,
        );
        let declarator = rewriter.ast.variable_declarator(
            swc_next_ecma_ast::DUMMY_SP,
            BindingPattern::BindingIdentifier(this_binding.declaration),
            None,
            false,
        );
        let mut declarators = vec![declarator];
        if let Some(completion) = completion {
            declarators.push(rewriter.ast.variable_declarator(
                swc_next_ecma_ast::DUMMY_SP,
                BindingPattern::BindingIdentifier(completion.value.declaration),
                None,
                false,
            ));
            let pending = rewriter
                .ast
                .expr_boolean_literal(swc_next_ecma_ast::DUMMY_SP, false);
            declarators.push(rewriter.ast.variable_declarator(
                swc_next_ecma_ast::DUMMY_SP,
                BindingPattern::BindingIdentifier(completion.pending.declaration),
                Some(pending),
                false,
            ));
        }
        declarators.extend(temp_declarations.iter().map(|binding| {
            rewriter.ast.variable_declarator(
                swc_next_ecma_ast::DUMMY_SP,
                BindingPattern::BindingIdentifier(binding.declaration),
                None,
                false,
            )
        }));
        let declarators = rewriter.ast.add_typed_sub_range(declarators);
        let declaration = rewriter.ast.variable_declaration(
            swc_next_ecma_ast::DUMMY_SP,
            VariableKind::Var,
            declarators,
            false,
        );
        let fallthrough = rewriter.assert_this_initialized();
        let fallthrough = rewriter
            .ast
            .return_statement(swc_next_ecma_ast::DUMMY_SP, Some(fallthrough));
        let directives = rewriter
            .ast
            .add_typed_sub_range(std::iter::once(prelude.directive).chain(directives));
        let statements = rewriter.ast.add_typed_sub_range(
            std::iter::once(Stmt::Declaration(Decl::VariableDeclaration(declaration)))
                .chain(std::iter::once(Stmt::IfStatement(prelude.guard.statement)))
                .chain(rewritten_statements)
                .chain(std::iter::once(Stmt::ReturnStatement(fallthrough))),
        );
        clone.root.set_directives(rewriter.ast, directives);
        clone.root.set_body(rewriter.ast, statements);
        (prelude, rewriter.references)
    });

    let layout = tx.replace_from_transformed_clone(body, clone, scope, &[]);
    prelude.register_references(
        tx,
        layout.scope_of(prelude.guard.statement.node_id()),
        class_symbol,
    );
    for generated in references {
        let reference = generated.reference;
        let reference_scope = layout.scope_of(reference.node_id());
        tx.reference(
            reference_scope,
            reference.node_id(),
            tx.ast().canonicalize_utf8(reference.name(tx.ast())),
            generated.flags,
            Some(generated.symbol),
        );
    }
}

#[derive(Clone, Copy)]
struct DerivedConstructorReturn {
    statement: swc_next_ecma_ast::ReturnStatement,
    helper_reference: IdentifierReference,
    class_reference: IdentifierReference,
    arguments_reference: IdentifierReference,
}

impl DerivedConstructorReturn {
    fn build(
        ast: &mut Ast<'_>,
        span: swc_next_ecma_ast::Span,
        binding_name: AstUtf8,
        binding_context: SyntaxContext,
        helper: PrivateBinding,
    ) -> Self {
        let helper_reference = ast.identifier_reference(span, helper.text);
        ast.set_syntax_context(helper_reference.node_id(), helper.context);
        let this_expression = ast.expr_this_expression(span);
        let class_reference = ast.identifier_reference(span, binding_name);
        ast.set_syntax_context(class_reference.node_id(), binding_context);
        let arguments_name = ast.add_utf8("arguments");
        let arguments_reference = ast.identifier_reference(span, arguments_name);
        let arguments = ast.add_typed_sub_range([
            Argument::Expr(this_expression),
            Argument::Expr(Expr::IdentifierReference(class_reference)),
            Argument::Expr(Expr::IdentifierReference(arguments_reference)),
        ]);
        let call = ast.expr_call_expression(
            span,
            Expr::IdentifierReference(helper_reference),
            arguments,
            false,
            None,
        );
        let statement = ast.return_statement(span, Some(call));
        Self {
            statement,
            helper_reference,
            class_reference,
            arguments_reference,
        }
    }

    fn register(
        self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        body: swc_next_ecma_ast::NodeId,
        scope: ScopeHandle,
        class_symbol: SymbolHandle,
        helper_symbol: SymbolHandle,
    ) {
        tx.register_subtree(Stmt::ReturnStatement(self.statement), Some(body), scope);
        tx.reference(
            scope,
            self.helper_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.helper_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(helper_symbol),
        );
        tx.reference(
            scope,
            self.class_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.class_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(class_symbol),
        );
        tx.reference(
            scope,
            self.arguments_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.arguments_reference.name(tx.ast())),
            ReferenceFlags::default(),
            None::<swc_next_ecma_ast::SymbolId>,
        );
    }
}

impl ConstructorPrelude {
    fn build(
        ast: &mut Ast<'_>,
        span: swc_next_ecma_ast::Span,
        binding_name: AstUtf8,
        binding_context: SyntaxContext,
    ) -> Self {
        let directive = build_strict_directive(ast, span);
        let this_expression = ast.this_expression(span);
        let class_reference = ast.identifier_reference(span, binding_name);
        ast.set_syntax_context(class_reference.node_id(), binding_context);
        let instance_of = ast.binary_expression(
            span,
            Expr::ThisExpression(this_expression),
            Expr::IdentifierReference(class_reference),
            BinaryOperator::Instanceof,
        );
        let not_instance = ast.unary_expression(
            span,
            Expr::BinaryExpression(instance_of),
            UnaryOperator::LogicalNot,
        );
        let type_error_name = ast.add_utf8("TypeError");
        let type_error_reference = ast.identifier_reference(span, type_error_name);
        let type_error = ast.new_expression(
            span,
            Expr::IdentifierReference(type_error_reference),
            TypedSubRange::empty(),
            None,
        );
        let throw = ast.throw_statement(span, Expr::NewExpression(type_error));
        let guard = ast.if_statement(
            span,
            Expr::UnaryExpression(not_instance),
            Stmt::ThrowStatement(throw),
            None,
        );
        Self {
            directive,
            guard: ConstructorGuard {
                statement: guard,
                class_reference,
                type_error_reference,
            },
        }
    }

    fn register(
        self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        body: swc_next_ecma_ast::NodeId,
        scope: ScopeHandle,
        class_symbol: SymbolHandle,
    ) {
        tx.register_subtree(self.directive, Some(body), scope);
        self.guard.register(tx, body, scope, class_symbol);
    }

    fn register_references(
        self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        scope: ScopeHandle,
        class_symbol: SymbolHandle,
    ) {
        self.guard.register_references(tx, scope, class_symbol);
    }
}

fn build_strict_directive(
    ast: &mut Ast<'_>,
    span: swc_next_ecma_ast::Span,
) -> swc_next_ecma_ast::Directive {
    let directive_value = ast.add_utf8("use strict");
    let directive_raw = ast.add_utf8("\"use strict\"");
    let literal_value = ast.add_wtf8("use strict".into());
    let literal = ast.string_literal(span, literal_value, directive_raw);
    ast.directive(span, literal, directive_value)
}

impl ConstructorGuard {
    fn register(
        self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        body: swc_next_ecma_ast::NodeId,
        scope: ScopeHandle,
        class_symbol: SymbolHandle,
    ) {
        tx.register_subtree(Stmt::IfStatement(self.statement), Some(body), scope);
        self.register_references(tx, scope, class_symbol);
    }

    fn register_references(
        self,
        tx: &mut crate::mutation::MutationTransaction<'_, '_>,
        scope: ScopeHandle,
        class_symbol: SymbolHandle,
    ) {
        tx.reference(
            scope,
            self.class_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.class_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(class_symbol),
        );
        tx.reference(
            scope,
            self.type_error_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(self.type_error_reference.name(tx.ast())),
            ReferenceFlags::default(),
            None::<swc_next_ecma_ast::SymbolId>,
        );
    }
}
