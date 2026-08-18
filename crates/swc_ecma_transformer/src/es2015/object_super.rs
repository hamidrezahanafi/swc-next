//! ES2015 object-literal `super` lowering.

use std::collections::HashMap;

use swc_next_ecma_ast::{
    Ast, BindingPattern, DUMMY_SP, Expr, NodeId, NodeKindData, Program, Stmt, VariableKind, Visit,
    VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace, SymbolFlags};

use crate::{
    mutation::{PrivateBinding, ScopeHandle, SymbolHandle},
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct ObjectSuper;

impl ObjectSuper {
    pub fn new() -> Self {
        Self
    }

    pub fn transform(&mut self, program: Program, ctx: &mut TransformerCtx<'_, '_, '_>) {
        let targets = collect_targets(program, ctx.ast, ctx.semantic());
        if targets.is_empty() {
            return;
        }
        let root_scope = ctx.semantic().scope_of(program.node_id());
        ctx.transaction(program.node_id(), |tx| {
            let clone = tx.clone_detached(program);
            let targets = targets
                .iter()
                .map(|target| {
                    let binding = tx.declare_private(
                        target.hoist_scope,
                        "_obj",
                        SymbolFlags::FUNCTION_SCOPED_VAR,
                    );
                    TargetBinding {
                        object: clone.clone_of(target.object).unwrap(),
                        owner: clone
                            .clone_of(tx.semantic().scope(target.hoist_scope).node)
                            .unwrap_or(clone.root.node_id()),
                        binding,
                    }
                })
                .collect::<Vec<_>>();
            let references = tx.build_detached(|ast| {
                inject_declarations(ast, &targets);
                let bindings = targets
                    .iter()
                    .map(|target| (target.object, target.binding))
                    .collect();
                let mut rewriter = ObjectSuperRewriter {
                    ast,
                    bindings,
                    references: Vec::new(),
                };
                clone.root.visit_mut_children_with(&mut rewriter);
                rewriter.references
            });
            let layout = tx.replace_from_transformed_clone(program, clone, root_scope, &[]);
            for reference in references {
                let reference_scope = layout.scope_of(reference.node.node_id());
                let name = tx.ast().canonicalize_utf8(reference.node.name(tx.ast()));
                let symbol: Option<SymbolHandle> = match reference.target {
                    ReferenceTarget::Capture(binding) => Some(binding.symbol.into()),
                    ReferenceTarget::Helper => {
                        let ScopeHandle::Existing(scope) = reference_scope else {
                            unreachable!("object-super creates no scopes")
                        };
                        tx.semantic()
                            .lookup(scope, name, ReferenceSpace::Value)
                            .map(Into::into)
                    }
                };
                tx.reference(
                    reference_scope,
                    reference.node.node_id(),
                    name,
                    reference.flags,
                    symbol,
                );
            }
        })
        .expect("object-super mutation must pass preflight");
    }
}

#[derive(Clone, Copy)]
struct ObjectTarget {
    object: NodeId,
    hoist_scope: swc_next_ecma_ast::ScopeId,
}

#[derive(Clone, Copy)]
struct TargetBinding {
    object: NodeId,
    owner: NodeId,
    binding: PrivateBinding,
}

fn collect_targets(
    program: Program,
    ast: &Ast<'_>,
    semantic: &swc_next_ecma_semantic::Semantic<'_>,
) -> Vec<ObjectTarget> {
    struct Collector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        objects: Vec<NodeId>,
    }
    impl<'ast> Visit<'ast> for Collector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_object_expression(&mut self, object: swc_next_ecma_ast::ObjectExpression) {
            object.visit_children_with(self);
            if object_has_direct_method_super(object, self.ast) {
                self.objects.push(object.node_id());
            }
        }
    }

    let mut collector = Collector {
        ast,
        objects: Vec::new(),
    };
    program.visit_children_with(&mut collector);
    collector
        .objects
        .into_iter()
        .map(|object| ObjectTarget {
            object,
            hoist_scope: semantic.scope(semantic.scope_of(object)).hoist_target,
        })
        .collect()
}

fn object_has_direct_method_super(
    object: swc_next_ecma_ast::ObjectExpression,
    ast: &Ast<'_>,
) -> bool {
    object.properties(ast).iter().any(|property| {
        let property = ast.get_node_in_sub_range(property);
        let Some(property) = property.as_object_property(ast) else {
            return false;
        };
        if !property.method(ast) {
            return false;
        }
        let Some(function) = property.value(ast).as_function(ast) else {
            return false;
        };
        let mut finder = DirectSuperFinder { ast, found: false };
        function.visit_children_with(&mut finder);
        finder.found
    })
}

struct DirectSuperFinder<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    found: bool,
}

impl<'ast> Visit<'ast> for DirectSuperFinder<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_super(&mut self, _node: swc_next_ecma_ast::Super) {
        self.found = true;
    }

    fn visit_function(&mut self, _node: swc_next_ecma_ast::Function) {}
    fn visit_class(&mut self, _node: swc_next_ecma_ast::Class) {}
    fn visit_object_expression(&mut self, _node: swc_next_ecma_ast::ObjectExpression) {}
}

fn inject_declarations(ast: &mut Ast<'_>, targets: &[TargetBinding]) {
    let mut groups: Vec<(NodeId, Vec<PrivateBinding>)> = Vec::new();
    for target in targets {
        if let Some((_, bindings)) = groups.iter_mut().find(|(owner, _)| *owner == target.owner) {
            bindings.push(target.binding);
        } else {
            groups.push((target.owner, vec![target.binding]));
        }
    }
    for (owner, bindings) in groups {
        let declarators = bindings
            .into_iter()
            .map(|binding| {
                ast.variable_declarator(
                    DUMMY_SP,
                    BindingPattern::BindingIdentifier(binding.declaration),
                    None,
                    false,
                )
            })
            .collect::<Vec<_>>();
        let declarators = ast.add_typed_sub_range(declarators);
        let declaration = ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
        let statement =
            Stmt::Declaration(swc_next_ecma_ast::Decl::VariableDeclaration(declaration));
        match ast.kind_data(owner) {
            NodeKindData::Program(program) => {
                let body = std::iter::once(statement)
                    .chain(
                        program
                            .body(ast)
                            .iter()
                            .map(|id| ast.get_node_in_sub_range(id)),
                    )
                    .collect::<Vec<_>>();
                let body = ast.add_typed_sub_range(body);
                program.set_body(ast, body);
            }
            NodeKindData::Function(function) => {
                prepend_to_function_body(function.body(ast), statement, ast)
            }
            NodeKindData::StaticBlock(block) => {
                let body = std::iter::once(statement)
                    .chain(
                        block
                            .body(ast)
                            .iter()
                            .map(|id| ast.get_node_in_sub_range(id)),
                    )
                    .collect::<Vec<_>>();
                let body = ast.add_typed_sub_range(body);
                block.set_body(ast, body);
            }
            kind => panic!("unsupported object-super hoist owner: {kind:?}"),
        }
    }
}

fn prepend_to_function_body(
    body: swc_next_ecma_ast::FunctionBody,
    statement: Stmt,
    ast: &mut Ast<'_>,
) {
    let statements = std::iter::once(statement)
        .chain(
            body.body(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id)),
        )
        .collect::<Vec<_>>();
    let statements = ast.add_typed_sub_range(statements);
    body.set_body(ast, statements);
}

#[derive(Clone, Copy)]
enum ReferenceTarget {
    Capture(PrivateBinding),
    Helper,
}

struct GeneratedReference {
    node: swc_next_ecma_ast::IdentifierReference,
    flags: ReferenceFlags,
    target: ReferenceTarget,
}

struct ObjectSuperRewriter<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    bindings: HashMap<NodeId, PrivateBinding>,
    references: Vec<GeneratedReference>,
}

impl<'ast> VisitMut<'ast> for ObjectSuperRewriter<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        node.visit_mut_children_with(self);
        let object = node.as_object_expression(self.ast)?;
        let binding = self.bindings.get(&object.node_id()).copied()?;
        for property in object.properties(self.ast).iter() {
            let property = self.ast.get_node_in_sub_range(property);
            let Some(property) = property.as_object_property(self.ast) else {
                continue;
            };
            if !property.method(self.ast) {
                continue;
            }
            let Some(function) = property.value(self.ast).as_function(self.ast) else {
                continue;
            };
            let mut replacer = SuperReplacer {
                ast: self.ast,
                capture: binding,
                references: &mut self.references,
            };
            function.visit_mut_children_with(&mut replacer);
        }
        let target = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(
                DUMMY_SP,
                binding.text,
            );
        let reference = target
            .as_simple_assignment_target(self.ast)
            .and_then(|target| target.as_identifier_reference(self.ast))
            .unwrap();
        self.ast
            .set_syntax_context(reference.node_id(), binding.context);
        self.references.push(GeneratedReference {
            node: reference,
            flags: ReferenceFlags::default().write_only(),
            target: ReferenceTarget::Capture(binding),
        });
        let properties = object.properties(self.ast);
        let object = self.ast.expr_object_expression(DUMMY_SP, properties);
        let assignment = self.ast.assignment_expression(
            DUMMY_SP,
            target,
            object,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        );
        unsafe { self.ast.replace_data(node, assignment) };
        None
    }
}

struct SuperReplacer<'visit, 'ast, 'refs> {
    ast: &'visit mut Ast<'ast>,
    capture: PrivateBinding,
    references: &'refs mut Vec<GeneratedReference>,
}

impl SuperReplacer<'_, '_, '_> {
    fn capture(&mut self) -> Expr {
        let reference = self.ast.identifier_reference(DUMMY_SP, self.capture.text);
        self.ast
            .set_syntax_context(reference.node_id(), self.capture.context);
        self.references.push(GeneratedReference {
            node: reference,
            flags: ReferenceFlags::default(),
            target: ReferenceTarget::Capture(self.capture),
        });
        Expr::IdentifierReference(reference)
    }

    fn helper(&mut self, name: &str) -> Expr {
        let name = self.ast.add_utf8(name);
        let reference = self.ast.identifier_reference(DUMMY_SP, name);
        self.references.push(GeneratedReference {
            node: reference,
            flags: ReferenceFlags::default(),
            target: ReferenceTarget::Helper,
        });
        Expr::IdentifierReference(reference)
    }

    fn property(&mut self, key: swc_next_ecma_ast::PropertyKey) -> Expr {
        if let Some(identifier) = key.as_identifier_name(self.ast) {
            let text = self.ast.get_utf8(identifier.name(self.ast)).to_owned();
            let value = self.ast.add_wtf8((&*text).into());
            let raw = self.ast.add_utf8(&format!("\"{text}\""));
            self.ast.expr_string_literal(DUMMY_SP, value, raw)
        } else if let Some(string) = key.as_string_literal(self.ast) {
            Expr::StringLiteral(string)
        } else if let Some(private) = key.as_private_identifier(self.ast) {
            let text = self.ast.get_utf8(private.name(self.ast)).to_owned();
            let value = self.ast.add_wtf8((&*text).into());
            let raw = self.ast.add_utf8(&format!("\"{text}\""));
            self.ast.expr_string_literal(DUMMY_SP, value, raw)
        } else {
            key.as_expr(self.ast)
                .expect("computed super key is an expression")
        }
    }

    fn prototype(&mut self) -> Expr {
        let capture = self.capture();
        let helper = self.helper("_get_prototype_of");
        let arguments = self
            .ast
            .add_typed_sub_range([swc_next_ecma_ast::Argument::Expr(capture)]);
        self.ast
            .expr_call_expression(DUMMY_SP, helper, arguments, false, None)
    }

    fn get(&mut self, property: Expr) -> Expr {
        let prototype = self.prototype();
        let helper = self.helper("_get");
        let this = self.ast.expr_this_expression(DUMMY_SP);
        let arguments = self.ast.add_typed_sub_range([
            swc_next_ecma_ast::Argument::Expr(prototype),
            swc_next_ecma_ast::Argument::Expr(property),
            swc_next_ecma_ast::Argument::Expr(this),
        ]);
        self.ast
            .expr_call_expression(DUMMY_SP, helper, arguments, false, None)
    }

    fn set(&mut self, property: Expr, value: Expr) -> Expr {
        let prototype = self.prototype();
        let helper = self.helper("_set");
        let this = self.ast.expr_this_expression(DUMMY_SP);
        let strict = self.ast.expr_boolean_literal(DUMMY_SP, true);
        let arguments = self.ast.add_typed_sub_range([
            swc_next_ecma_ast::Argument::Expr(prototype),
            swc_next_ecma_ast::Argument::Expr(property),
            swc_next_ecma_ast::Argument::Expr(value),
            swc_next_ecma_ast::Argument::Expr(this),
            swc_next_ecma_ast::Argument::Expr(strict),
        ]);
        self.ast
            .expr_call_expression(DUMMY_SP, helper, arguments, false, None)
    }
}

impl<'ast> VisitMut<'ast> for SuperReplacer<'_, 'ast, '_> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        if node.is_function(self.ast)
            || node.is_class(self.ast)
            || node.is_object_expression(self.ast)
        {
            return None;
        }
        if let Some(call) = node.as_call_expression(self.ast) {
            let callee = call.callee(self.ast);
            if let Some(member) = callee.as_member_expression(self.ast)
                && member.object(self.ast).is_super(self.ast)
            {
                let property = self.property(member.property(self.ast));
                let get = self.get(property);
                let call_name = self.ast.add_utf8("call");
                let call_key = self.ast.property_key_identifier_name(DUMMY_SP, call_name);
                let callee = self
                    .ast
                    .expr_member_expression(DUMMY_SP, get, call_key, false, false);
                let this = self.ast.expr_this_expression(DUMMY_SP);
                let arguments = std::iter::once(swc_next_ecma_ast::Argument::Expr(this))
                    .chain(
                        call.arguments(self.ast)
                            .iter()
                            .map(|id| self.ast.get_node_in_sub_range(id)),
                    )
                    .collect::<Vec<_>>();
                let arguments = self.ast.add_typed_sub_range(arguments);
                return Some(
                    self.ast
                        .expr_call_expression(DUMMY_SP, callee, arguments, false, None),
                );
            }
        }
        if let Some(assignment) = node.as_assignment_expression(self.ast)
            && let Some(member) = assignment
                .left(self.ast)
                .as_simple_assignment_target(self.ast)
                .and_then(|target| target.as_member_expression(self.ast))
            && member.object(self.ast).is_super(self.ast)
        {
            let property = self.property(member.property(self.ast));
            return Some(self.set(property, assignment.right(self.ast)));
        }
        if let Some(member) = node.as_member_expression(self.ast)
            && member.object(self.ast).is_super(self.ast)
        {
            let property = self.property(member.property(self.ast));
            return Some(self.get(property));
        }
        node.visit_mut_children_with(self)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn object_capture_and_super_helpers_are_transactional() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function make(value) { return { method(arg) { super.value = arg; return super.value + super.call(arg); } }; }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.object_super = true;
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
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
