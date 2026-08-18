use swc_next_ecma_ast::{
    Accessibility, Ast, ClassType, MethodDefinitionKind, NodeKindData, Program, PropertyKeyData,
};
use swc_next_ecma_semantic::{Semantic, SymbolFlags, scope::ScopeKind};

use crate::mutation::StatementListOwner;

use super::plan::{
    AnonymousClassExpressionPlan, AnonymousDefaultClassPlan, ClassExport, ClassExpressionPlan,
    ClassLoweringPlan, ConstructorPlan, MethodGroupPlan, MethodKeyPlan, MethodPlan,
};

pub(super) fn analyze_anonymous_default_exports(
    program: Program,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> Vec<AnonymousDefaultClassPlan> {
    let Ok(facts) = semantic.classes_post_order() else {
        return Vec::new();
    };
    program
        .body(ast)
        .iter()
        .filter_map(|statement| {
            let statement = ast.get_node_in_sub_range(statement);
            let export = statement.as_export_default_declaration(ast)?;
            let class = export.declaration(ast).as_class(ast)?;
            if class.id(ast).is_some()
                || class.class_type(ast) != ClassType::ClassDeclaration
                || !class.decorators(ast).is_empty()
                || class.type_parameters(ast).is_some()
                || class.super_type_arguments(ast).is_some()
                || !class.implements(ast).is_empty()
                || class.r#abstract(ast)
                || class.declare(ast)
            {
                return None;
            }
            let fact = facts
                .iter()
                .find(|fact| fact.class.node_id() == class.node_id())?;
            if fact.name_symbol.is_some()
                || fact.has_properties
                || fact.has_private_elements
                || fact.has_static_blocks
                || fact.has_decorators
                || !supported_super_members(fact, ast, semantic)
                || semantic.scope(fact.class_scope).kind != ScopeKind::Class
                || semantic.parent_of(class.node_id()) != Some(export.node_id())
                || semantic.parent_of(export.node_id()) != Some(program.node_id())
            {
                return None;
            }
            let (constructor, methods) = supported_expression_members(fact, ast, semantic)?;
            let (constructor_super_calls, constructor_this_nodes) =
                constructor_nodes(fact, constructor, ast, semantic);
            Some(AnonymousDefaultClassPlan {
                statement,
                export,
                class,
                body: class.body(ast),
                class_scope: fact.class_scope,
                outer_scope: semantic.scope(fact.class_scope).parent?,
                heritage: class.super_class(ast),
                super_members: fact.super_members.iter().copied().collect(),
                has_super_writes: has_super_writes(fact, ast, semantic),
                constructor_super_calls,
                constructor_this_nodes,
                constructor,
                methods,
            })
        })
        .collect()
}

pub(super) fn analyze_anonymous_expressions(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> Vec<AnonymousClassExpressionPlan> {
    let Ok(facts) = semantic.classes_post_order() else {
        return Vec::new();
    };
    facts
        .iter()
        .filter_map(|fact| {
            let class = fact.class;
            if class.class_type(ast) != ClassType::ClassExpression
                || class.id(ast).is_some()
                || !class.decorators(ast).is_empty()
                || class.type_parameters(ast).is_some()
                || class.super_type_arguments(ast).is_some()
                || !class.implements(ast).is_empty()
                || class.r#abstract(ast)
                || class.declare(ast)
                || fact.has_properties
                || fact.has_private_elements
                || fact.has_static_blocks
                || fact.has_decorators
                || !supported_super_members(fact, ast, semantic)
                || semantic.scope(fact.class_scope).kind != ScopeKind::Class
            {
                return None;
            }
            let (constructor, methods) = supported_expression_members(fact, ast, semantic)?;
            let (constructor_super_calls, constructor_this_nodes) =
                constructor_nodes(fact, constructor, ast, semantic);
            Some(AnonymousClassExpressionPlan {
                class,
                body: class.body(ast),
                class_scope: fact.class_scope,
                outer_scope: semantic.scope(fact.class_scope).parent?,
                heritage: class.super_class(ast),
                super_members: fact.super_members.iter().copied().collect(),
                has_super_writes: has_super_writes(fact, ast, semantic),
                constructor_super_calls,
                constructor_this_nodes,
                constructor,
                methods,
            })
        })
        .collect()
}

pub(super) fn analyze_named_expressions(
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> Vec<ClassExpressionPlan> {
    let Ok(facts) = semantic.classes_post_order() else {
        return Vec::new();
    };
    facts
        .iter()
        .filter_map(|fact| {
            let class = fact.class;
            if class.class_type(ast) != ClassType::ClassExpression
                || class.id(ast).is_none()
                || !class.decorators(ast).is_empty()
                || class.type_parameters(ast).is_some()
                || class.super_type_arguments(ast).is_some()
                || !class.implements(ast).is_empty()
                || class.r#abstract(ast)
                || class.declare(ast)
                || fact.has_properties
                || fact.has_private_elements
                || fact.has_static_blocks
                || fact.has_decorators
                || !supported_super_members(fact, ast, semantic)
            {
                return None;
            }
            let binding = class.id(ast)?;
            let declaration = semantic.declaration_of(binding.node_id())?;
            let symbol = semantic.declaration(declaration).symbol;
            let name_scope = semantic.scope(fact.class_scope).parent?;
            let outer_scope = semantic.scope(name_scope).parent?;
            if fact.name_symbol != Some(symbol)
                || semantic.scope(fact.class_scope).kind != ScopeKind::Class
                || semantic.scope(name_scope).kind != ScopeKind::ExpressionName
            {
                return None;
            }
            let heritage = class.super_class(ast);
            let definition_tdz_self_references = references_to_symbol_in_subtrees(
                heritage
                    .map(|heritage| heritage.node_id())
                    .into_iter()
                    .chain(fact.computed_keys.iter().copied()),
                symbol,
                semantic,
            );
            if definition_tdz_self_references
                .iter()
                .any(|reference| !semantic.reference(*reference).flags.is_read_only())
            {
                return None;
            }
            let (constructor, methods) = supported_expression_members(fact, ast, semantic)?;
            let (constructor_super_calls, constructor_this_nodes) =
                constructor_nodes(fact, constructor, ast, semantic);
            let mut output_flags = semantic.symbol(symbol).flags;
            output_flags.remove(SymbolFlags::CLASS);
            output_flags.insert(SymbolFlags::FUNCTION);
            Some(ClassExpressionPlan {
                class,
                body: class.body(ast),
                binding,
                declaration,
                symbol,
                output_flags,
                class_scope: fact.class_scope,
                name_scope,
                outer_scope,
                heritage,
                definition_tdz_self_references: definition_tdz_self_references
                    .into_iter()
                    .map(|reference| semantic.reference(reference).node)
                    .collect(),
                super_members: fact.super_members.iter().copied().collect(),
                has_super_writes: has_super_writes(fact, ast, semantic),
                constructor_super_calls,
                constructor_this_nodes,
                constructor,
                methods,
            })
        })
        .collect()
}

fn references_to_symbol_in_subtrees(
    roots: impl IntoIterator<Item = swc_next_ecma_ast::NodeId>,
    symbol: swc_next_ecma_ast::SymbolId,
    semantic: &Semantic<'_>,
) -> Vec<swc_next_ecma_ast::ReferenceId> {
    let mut references = Vec::new();
    let mut visited = HashSet::new();
    let mut pending: Vec<_> = roots.into_iter().collect();
    while let Some(node) = pending.pop() {
        if !visited.insert(node) {
            continue;
        }
        if let Some(reference) = semantic.reference_of(node)
            && semantic.reference(reference).symbol == Some(symbol)
        {
            references.push(reference);
        }
        pending.extend(semantic.node_children(node));
    }
    references
}

fn supported_super_members(
    fact: &swc_next_ecma_semantic::ClassLoweringFact<'_>,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> bool {
    fact.super_members.iter().all(|member| {
        let NodeKindData::MemberExpression(expression) = ast.kind_data(*member) else {
            return false;
        };
        if expression.optional(ast) {
            return false;
        }
        let Some(parent) = semantic.parent_of(*member) else {
            return false;
        };
        match ast.kind_data(parent) {
            NodeKindData::CallExpression(call) if call.callee(ast).node_id() == *member => {
                !call.optional(ast)
            }
            NodeKindData::AssignmentExpression(_) | NodeKindData::UpdateExpression(_) => true,
            NodeKindData::TaggedTemplateExpression(_) => false,
            _ => true,
        }
    })
}

fn has_super_writes(
    fact: &swc_next_ecma_semantic::ClassLoweringFact<'_>,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> bool {
    fact.super_members.iter().any(|member| {
        semantic.parent_of(*member).is_some_and(|parent| {
            matches!(
                ast.kind_data(parent),
                NodeKindData::AssignmentExpression(_) | NodeKindData::UpdateExpression(_)
            )
        })
    })
}

fn supported_expression_members(
    fact: &swc_next_ecma_semantic::ClassLoweringFact<'_>,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> Option<(ConstructorPlan, Vec<MethodGroupPlan>)> {
    if fact.class.body(ast).body(ast).len() != fact.methods.len() {
        return None;
    }
    let mut constructor = ConstructorPlan::Synthetic;
    let mut has_constructor = false;
    let mut methods = Vec::new();
    let mut accessor_groups = HashMap::new();
    for method in &fact.methods {
        let definition = method.definition;
        let function = method.function;
        if !definition.decorators(ast).is_empty()
            || definition.r#override(ast)
            || definition.optional(ast)
            || definition.r#abstract(ast)
            || definition.accessibility(ast) != Accessibility::None
            || function.generator(ast)
            || function.r#async(ast)
            || function.id(ast).is_some()
            || function.type_parameters(ast).is_some()
            || function.return_type(ast).is_some()
            || semantic
                .scope(semantic.scope_of(function.body(ast).node_id()))
                .kind
                == ScopeKind::FunctionBody
        {
            return None;
        }
        match method.kind {
            MethodDefinitionKind::Constructor if !has_constructor && !method.r#static => {
                constructor = ConstructorPlan::Existing {
                    definition,
                    function,
                    scope: method.function_scope,
                };
                has_constructor = true;
            }
            kind @ (MethodDefinitionKind::Method
            | MethodDefinitionKind::Get
            | MethodDefinitionKind::Set) => {
                let (key, group_name) = if method.computed {
                    if semantic.scope(fact.class_scope).flags.direct_eval {
                        return None;
                    }
                    let PropertyKeyData::Expr(expression) =
                        ast.property_key_data(definition.key(ast))
                    else {
                        return None;
                    };
                    (MethodKeyPlan::Computed(expression), None)
                } else {
                    let (key, group_name) = method_key(definition.key(ast), ast)?;
                    (key, Some(group_name))
                };
                let method = MethodPlan {
                    definition,
                    function,
                    scope: method.function_scope,
                    r#static: method.r#static,
                    kind,
                    key,
                };
                match kind {
                    MethodDefinitionKind::Method => {
                        if let Some(group_name) = group_name {
                            accessor_groups.remove(&(method.r#static, group_name));
                        }
                        methods.push(MethodGroupPlan::Data(method));
                    }
                    MethodDefinitionKind::Get | MethodDefinitionKind::Set => {
                        let Some(group_name) = group_name else {
                            methods.push(MethodGroupPlan::Accessor {
                                get: (kind == MethodDefinitionKind::Get).then_some(method),
                                set: (kind == MethodDefinitionKind::Set).then_some(method),
                            });
                            continue;
                        };
                        let group_key = (method.r#static, group_name);
                        let existing = accessor_groups.get(&group_key).copied();
                        let merged = existing.is_some_and(|index| {
                            let MethodGroupPlan::Accessor { get, set } = &mut methods[index] else {
                                unreachable!("accessor index points at an accessor group")
                            };
                            let slot = if kind == MethodDefinitionKind::Get {
                                get
                            } else {
                                set
                            };
                            if slot.is_none() {
                                *slot = Some(method);
                                true
                            } else {
                                false
                            }
                        });
                        if !merged {
                            let index = methods.len();
                            methods.push(MethodGroupPlan::Accessor {
                                get: (kind == MethodDefinitionKind::Get).then_some(method),
                                set: (kind == MethodDefinitionKind::Set).then_some(method),
                            });
                            accessor_groups.insert(group_key, index);
                        }
                    }
                    MethodDefinitionKind::Constructor => unreachable!(),
                }
            }
            _ => return None,
        }
    }
    Some((constructor, methods))
}

pub(super) fn analyze_program(
    program: Program,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> Vec<ClassLoweringPlan> {
    let Ok(facts) = semantic.classes_post_order() else {
        return Vec::new();
    };
    let body = program.body(ast);
    let mut plans = Vec::new();
    for index in 0..body.len() {
        let statement = body.get_node(ast, index).unwrap();
        let (class, export) = if let Some(class) = statement
            .as_declaration(ast)
            .and_then(|declaration| declaration.as_class(ast))
        {
            (class, None)
        } else if let Some(export) = statement.as_export_named_declaration(ast)
            && let Some(class) = export
                .declaration(ast)
                .and_then(|declaration| declaration.as_class(ast))
        {
            (class, Some(ClassExport::Named(export)))
        } else if let Some(export) = statement.as_export_default_declaration(ast)
            && let Some(class) = export.declaration(ast).as_class(ast)
        {
            (class, Some(ClassExport::Default(export)))
        } else {
            continue;
        };
        let Some(fact) = facts
            .iter()
            .find(|fact| fact.class.node_id() == class.node_id())
        else {
            continue;
        };

        // This first lowering slice deliberately accepts only the shape for
        // which every semantic transition is already modeled. Additional
        // class shapes are enabled by extending ClassLoweringPlan, never by
        // bypassing the transaction.
        if class.class_type(ast) != ClassType::ClassDeclaration
            || !class.decorators(ast).is_empty()
            || class.type_parameters(ast).is_some()
            || class.super_type_arguments(ast).is_some()
            || !class.implements(ast).is_empty()
            || class.r#abstract(ast)
            || class.declare(ast)
            || fact.has_properties
            || fact.has_private_elements
            || fact.has_static_blocks
            || fact.has_decorators
        {
            continue;
        }
        let Some((constructor, methods)) = supported_expression_members(fact, ast, semantic) else {
            continue;
        };
        let heritage = class.super_class(ast);
        if !supported_super_members(fact, ast, semantic) {
            continue;
        }
        let Some(binding) = class.id(ast) else {
            continue;
        };
        let Some(declaration) = semantic.declaration_of(binding.node_id()) else {
            continue;
        };
        let symbol = semantic.declaration(declaration).symbol;
        if fact.name_symbol != Some(symbol) {
            continue;
        }
        let Some(outer_scope) = semantic.scope(fact.class_scope).parent else {
            continue;
        };
        let expected_parent = export.map_or(program.node_id(), ClassExport::node_id);
        if semantic.parent_of(class.node_id()) != Some(expected_parent)
            || export.is_some_and(|export| {
                semantic.parent_of(export.node_id()) != Some(program.node_id())
            })
        {
            continue;
        }

        let mut output_flags = semantic.symbol(symbol).flags;
        output_flags.remove(SymbolFlags::CLASS);
        output_flags.insert(SymbolFlags::FUNCTION | SymbolFlags::ANNEX_B_FUNCTION);
        let (constructor_super_calls, constructor_this_nodes) =
            constructor_nodes(fact, constructor, ast, semantic);
        plans.push(ClassLoweringPlan {
            statement,
            owner: StatementListOwner::Program(program),
            export,
            class,
            body: class.body(ast),
            binding,
            declaration,
            symbol,
            output_flags,
            class_scope: fact.class_scope,
            outer_scope,
            heritage,
            super_members: fact.super_members.iter().copied().collect(),
            has_super_writes: has_super_writes(fact, ast, semantic),
            constructor_super_calls,
            constructor_this_nodes,
            constructor,
            methods,
        });
    }

    for fact in facts {
        let class = fact.class;
        if plans.iter().any(|plan| plan.class == class)
            || class.class_type(ast) != ClassType::ClassDeclaration
            || !class.decorators(ast).is_empty()
            || class.type_parameters(ast).is_some()
            || class.super_type_arguments(ast).is_some()
            || !class.implements(ast).is_empty()
            || class.r#abstract(ast)
            || class.declare(ast)
            || fact.has_properties
            || fact.has_private_elements
            || fact.has_static_blocks
            || fact.has_decorators
            || semantic.scope(fact.class_scope).kind != ScopeKind::Class
        {
            continue;
        }
        let Some(parent) = semantic.parent_of(class.node_id()) else {
            continue;
        };
        let Some(owner) = statement_list_owner(parent, ast) else {
            continue;
        };
        if matches!(owner, StatementListOwner::Program(_)) {
            continue;
        }
        let Some(statement) = statement_list(owner, ast)
            .into_iter()
            .find(|statement| statement.node_id() == class.node_id())
        else {
            continue;
        };
        let Some(binding) = class.id(ast) else {
            continue;
        };
        let Some(declaration) = semantic.declaration_of(binding.node_id()) else {
            continue;
        };
        let symbol = semantic.declaration(declaration).symbol;
        if fact.name_symbol != Some(symbol) {
            continue;
        }
        let Some(outer_scope) = semantic.scope(fact.class_scope).parent else {
            continue;
        };
        let Some((constructor, methods)) = supported_expression_members(fact, ast, semantic) else {
            continue;
        };
        let heritage = class.super_class(ast);
        if !supported_super_members(fact, ast, semantic) {
            continue;
        }
        let mut output_flags = semantic.symbol(symbol).flags;
        output_flags.remove(SymbolFlags::CLASS);
        output_flags.insert(SymbolFlags::FUNCTION | SymbolFlags::ANNEX_B_FUNCTION);
        let (constructor_super_calls, constructor_this_nodes) =
            constructor_nodes(fact, constructor, ast, semantic);
        plans.push(ClassLoweringPlan {
            statement,
            owner,
            export: None,
            class,
            body: class.body(ast),
            binding,
            declaration,
            symbol,
            output_flags,
            class_scope: fact.class_scope,
            outer_scope,
            heritage,
            super_members: fact.super_members.iter().copied().collect(),
            has_super_writes: has_super_writes(fact, ast, semantic),
            constructor_super_calls,
            constructor_this_nodes,
            constructor,
            methods,
        });
    }
    plans
}

fn constructor_nodes(
    fact: &swc_next_ecma_semantic::ClassLoweringFact<'_>,
    constructor: ConstructorPlan,
    ast: &Ast<'_>,
    semantic: &Semantic<'_>,
) -> (
    Vec<swc_next_ecma_ast::NodeId>,
    Vec<swc_next_ecma_ast::NodeId>,
) {
    let ConstructorPlan::Existing { function, .. } = constructor else {
        return (Vec::new(), Vec::new());
    };
    let body = function.body(ast);
    let body = body.node_id();
    let belongs_to_constructor = |node| {
        semantic
            .ancestors(Some(node))
            .any(|ancestor| ancestor == body)
    };
    (
        fact.super_calls
            .iter()
            .copied()
            .filter(|node| belongs_to_constructor(*node))
            .collect(),
        fact.this_nodes
            .iter()
            .copied()
            .filter(|node| belongs_to_constructor(*node))
            .collect(),
    )
}

fn statement_list_owner(
    node: swc_next_ecma_ast::NodeId,
    ast: &Ast<'_>,
) -> Option<StatementListOwner> {
    match ast.kind_data(node) {
        NodeKindData::Program(node) => Some(StatementListOwner::Program(node)),
        NodeKindData::BlockStatement(node) => Some(StatementListOwner::Block(node)),
        NodeKindData::FunctionBody(node) => Some(StatementListOwner::FunctionBody(node)),
        NodeKindData::StaticBlock(node) => Some(StatementListOwner::StaticBlock(node)),
        NodeKindData::SwitchCase(node) => Some(StatementListOwner::SwitchCase(node)),
        NodeKindData::TsModuleBlock(node) => Some(StatementListOwner::TsModuleBlock(node)),
        _ => None,
    }
}

fn statement_list(owner: StatementListOwner, ast: &Ast<'_>) -> Vec<swc_next_ecma_ast::Stmt> {
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

fn method_key(
    key: swc_next_ecma_ast::PropertyKey,
    ast: &Ast<'_>,
) -> Option<(MethodKeyPlan, Vec<u8>)> {
    match ast.property_key_data(key) {
        PropertyKeyData::IdentifierName(identifier) => {
            let name = identifier.name(ast);
            Some((
                MethodKeyPlan::Identifier(name),
                ast.get_utf8(name).as_bytes().to_vec(),
            ))
        }
        PropertyKeyData::StringLiteral(literal) => Some((
            MethodKeyPlan::String {
                value: literal.value(ast),
                raw: literal.raw(ast),
            },
            ast.get_wtf8(literal.value(ast)).as_bytes().to_vec(),
        )),
        PropertyKeyData::NumericLiteral(literal) => Some((
            MethodKeyPlan::Numeric {
                kind: literal.kind(ast),
                raw: literal.raw(ast),
            },
            ast.get_utf8(literal.raw(ast)).as_bytes().to_vec(),
        )),
        _ => None,
    }
}
use std::collections::{HashMap, HashSet};
