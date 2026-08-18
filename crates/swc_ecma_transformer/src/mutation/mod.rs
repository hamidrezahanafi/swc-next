pub(crate) mod builder;
mod commit;
mod journal;
mod verify;

#[cfg(test)]
pub use verify::assert_semantic_matches_reanalysis;
pub use verify::{InvariantViolation, verify_program};

use std::{collections::HashMap, error::Error, fmt};

use swc_next_ecma_ast::{
    Ast, AstStr, AstUtf8, BindingIdentifier, BlockStatement, FunctionBody, NodeId, Program,
    ReferenceId, ScopeId, StaticBlock, SwitchCase, SymbolId, SyntaxContext, TsModuleBlock,
};
use swc_next_ecma_semantic::Semantic;

use self::journal::MutationJournal;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IdentityPolicy {
    Preserve,
    Replace,
    Move,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum StatementListOwner {
    Program(Program),
    Block(BlockStatement),
    FunctionBody(FunctionBody),
    StaticBlock(StaticBlock),
    SwitchCase(SwitchCase),
    TsModuleBlock(TsModuleBlock),
}

impl StatementListOwner {
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Program(node) => node.node_id(),
            Self::Block(node) => node.node_id(),
            Self::FunctionBody(node) => node.node_id(),
            Self::StaticBlock(node) => node.node_id(),
            Self::SwitchCase(node) => node.node_id(),
            Self::TsModuleBlock(node) => node.node_id(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct PendingScopeId(usize);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct PendingSymbolId(usize);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct PendingReferenceId(usize);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum ScopeHandle {
    Existing(ScopeId),
    Pending(PendingScopeId),
}

impl From<ScopeId> for ScopeHandle {
    fn from(value: ScopeId) -> Self {
        Self::Existing(value)
    }
}

impl From<PendingScopeId> for ScopeHandle {
    fn from(value: PendingScopeId) -> Self {
        Self::Pending(value)
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SymbolHandle {
    Existing(SymbolId),
    Pending(PendingSymbolId),
}

impl From<SymbolId> for SymbolHandle {
    fn from(value: SymbolId) -> Self {
        Self::Existing(value)
    }
}

impl From<PendingSymbolId> for SymbolHandle {
    fn from(value: PendingSymbolId) -> Self {
        Self::Pending(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PrivateBinding {
    pub text: AstUtf8,
    pub context: SyntaxContext,
    pub declaration: BindingIdentifier,
    pub symbol: PendingSymbolId,
}

#[derive(Debug, Clone, Copy)]
pub struct ReservedPrivateName {
    pub text: AstUtf8,
    pub context: SyntaxContext,
}

/// A deep-cloned AST subtree that is not attached to the program yet.
///
/// The origin map lets a lowering pass mutate the clone while continuing to
/// query facts collected for the corresponding nodes in the attached tree.
/// This is particularly useful for transforms such as generator lowering,
/// where effect information belongs to the old tree until commit.
#[derive(Debug)]
pub struct DetachedClone<N> {
    pub root: N,
    origins: HashMap<NodeId, NodeId>,
    clones: HashMap<NodeId, NodeId>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DetachedScopeRegion {
    pub root: NodeId,
    pub scope: ScopeHandle,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DetachedScopeCreator {
    /// Node recorded as the semantic scope owner.
    pub owner: NodeId,
    /// Subtree roots evaluated inside the scope.
    pub entries: Vec<NodeId>,
    /// Whether the owner node itself is recorded in the created scope.
    pub owner_in_scope: bool,
    pub kind: swc_next_ecma_semantic::scope::ScopeKind,
    pub flags: swc_next_ecma_semantic::scope::ScopeFlags,
}

#[derive(Debug)]
pub struct DetachedScopeLayout {
    regions: Vec<DetachedScopeRegion>,
    node_scopes: HashMap<NodeId, ScopeHandle>,
}

/// Pending identity mapping produced when a separately analyzed generated
/// program is imported into a mutation transaction.
#[derive(Debug)]
pub struct GeneratedProgramImport {
    pub statements: Vec<swc_next_ecma_ast::Stmt>,
    scopes: HashMap<ScopeId, ScopeHandle>,
    symbols: HashMap<SymbolId, SymbolHandle>,
}

impl GeneratedProgramImport {
    pub fn scope(&self, source: ScopeId) -> ScopeHandle {
        self.scopes[&source]
    }

    pub fn symbol(&self, source: SymbolId) -> SymbolHandle {
        self.symbols[&source]
    }
}

impl DetachedScopeLayout {
    pub fn regions(&self) -> &[DetachedScopeRegion] {
        &self.regions
    }

    pub fn scope_of(&self, node: NodeId) -> ScopeHandle {
        self.node_scopes[&node]
    }
}

impl<N> DetachedClone<N> {
    pub fn origin_of(&self, clone: NodeId) -> Option<NodeId> {
        self.origins.get(&clone).copied()
    }

    pub fn clone_of(&self, origin: NodeId) -> Option<NodeId> {
        self.clones.get(&origin).copied()
    }

    pub fn len(&self) -> usize {
        self.origins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.origins.is_empty()
    }

    /// Changes the detached root variant while keeping its origin identity.
    /// Descendants retained by `new_root` keep their existing origin mapping;
    /// unreachable cloned descendants are reconciled at publication time.
    pub fn replace_root(&mut self, new_root: N)
    where
        N: Copy + swc_next_ecma_ast::AstNode,
    {
        let old_root = self.root.node_id();
        let new_root_id = new_root.node_id();
        let origin = self
            .origins
            .remove(&old_root)
            .expect("detached root has an origin");
        assert!(
            self.origins.insert(new_root_id, origin).is_none(),
            "replacement root must be newly generated"
        );
        self.clones.insert(origin, new_root_id);
        self.root = new_root;
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MutationError {
    EpochMismatch {
        ast: u64,
        semantic: u64,
    },
    InactiveRoot(NodeId),
    InactiveNode(NodeId),
    InactiveScope(ScopeId),
    RootScopeMutation,
    ScopeCycle(ScopeId),
    NodeCycle(NodeId),
    ScopeNotEmpty(ScopeId),
    ConflictingScopeEdit(ScopeId),
    InactiveReference(ReferenceId),
    InactiveDeclaration(swc_next_ecma_semantic::DeclarationId),
    InvalidNodeId(NodeId),
    AttachedReplacement(NodeId),
    ReplacementIsNotDetached(NodeId),
    NodeAlreadyRegistered(NodeId),
    NodeHasSemanticIdentity(NodeId),
    KindMismatch {
        old: NodeId,
        new: NodeId,
    },
    ConflictingNodeEdit(NodeId),
    ConflictingSymbolEdit(SymbolId),
    ConflictingReferenceEdit(ReferenceId),
    UnsupportedIdentityPolicy(IdentityPolicy),
    InvalidPendingScope(PendingScopeId),
    InvalidPendingSymbol(PendingSymbolId),
    InactiveSymbol(SymbolId),
    SymbolStillInUse(SymbolId),
    ExternalSymbolUse {
        symbol: SymbolId,
        reference: ReferenceId,
    },
    BindingConflict {
        scope: ScopeHandle,
        name: AstStr,
    },
    MissingModuleExport(NodeId),
    ConflictingModuleEdit(NodeId),
}

impl fmt::Display for MutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mutation preflight failed: {self:?}")
    }
}

impl Error for MutationError {}

#[derive(Debug)]
pub struct MutationCommit {
    pub epoch: u64,
    pub dirty_roots: Vec<NodeId>,
    scopes: Vec<ScopeId>,
    symbols: Vec<SymbolId>,
    references: Vec<ReferenceId>,
}

impl MutationCommit {
    pub fn scope(&self, id: PendingScopeId) -> ScopeId {
        self.scopes[id.0]
    }

    pub fn symbol(&self, id: PendingSymbolId) -> SymbolId {
        self.symbols[id.0]
    }

    pub fn reference(&self, id: PendingReferenceId) -> ReferenceId {
        self.references[id.0]
    }
}

#[must_use = "a mutation transaction must be committed or explicitly discarded"]
pub struct MutationTransaction<'tx, 'ast> {
    ast: &'tx mut Ast<'ast>,
    semantic: &'tx mut Semantic<'ast>,
    root: NodeId,
    ast_node_checkpoint: usize,
    ast_extra_checkpoint: usize,
    journal: MutationJournal,
    dirty_roots: Vec<NodeId>,
    committed: bool,
}

impl<'tx, 'ast> MutationTransaction<'tx, 'ast> {
    pub fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    pub fn semantic(&self) -> &Semantic<'ast> {
        self.semantic
    }

    pub fn begin(
        ast: &'tx mut Ast<'ast>,
        semantic: &'tx mut Semantic<'ast>,
        root: NodeId,
    ) -> Result<Self, MutationError> {
        if ast.mutation_epoch() != semantic.mutation_epoch() {
            return Err(MutationError::EpochMismatch {
                ast: ast.mutation_epoch(),
                semantic: semantic.mutation_epoch(),
            });
        }
        if !semantic.is_node_active(root) {
            return Err(MutationError::InactiveRoot(root));
        }
        let ast_node_checkpoint = ast.node_count();
        let ast_extra_checkpoint = ast.extra_data_count();
        Ok(Self {
            ast,
            semantic,
            root,
            ast_node_checkpoint,
            ast_extra_checkpoint,
            journal: MutationJournal::default(),
            dirty_roots: Vec::new(),
            committed: false,
        })
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn discard(self) {}

    fn mark_dirty(&mut self, node: NodeId) {
        if !self.dirty_roots.contains(&node) {
            self.dirty_roots.push(node);
        }
    }
}

impl Drop for MutationTransaction<'_, '_> {
    fn drop(&mut self) {
        if !self.committed {
            self.ast
                .truncate(self.ast_node_checkpoint, self.ast_extra_checkpoint);
        }
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{
        Ast, DUMMY_SP, Decl, FormalParameterKind, FunctionType, Mark, NodeId, NodeKind,
        NodeKindData, Stmt, SyntaxContext, TypedSubRange,
    };
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{
        AnalyzeOptions, ReferenceFlags, Semantic, SemanticFeatureFlags, SubtreeEffect, SymbolFlags,
        analyze,
    };

    use super::{
        DetachedScopeCreator, IdentityPolicy, MutationError, MutationTransaction,
        StatementListOwner, assert_semantic_matches_reanalysis, verify_program,
    };

    #[test]
    fn detached_clone_preserves_origin_and_hygiene_without_touching_attached_tree() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("let value = source + 1;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let source = semantic
            .iter_references()
            .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"source")
            .unwrap()
            .reference
            .node;
        let mark = ast.fresh_mark(Mark::root());
        let context = ast.apply_mark(SyntaxContext::empty(), mark);
        ast.set_syntax_context(source, context);
        let statement = parse.program.body(&ast).get_node(&ast, 0).unwrap();
        let node_count = ast.node_count();

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let detached = tx.clone_detached(statement);
        let cloned_source = detached.clone_of(source).unwrap();

        assert!(detached.len() > 1);
        assert_eq!(detached.origin_of(cloned_source), Some(source));
        assert_eq!(tx.ast().syntax_context(cloned_source), context);
        assert_eq!(
            tx.ast().node_kind(cloned_source),
            NodeKind::IdentifierReference
        );
        assert_ne!(detached.root.node_id(), statement.node_id());

        let replacement_name = tx.build_detached(|ast| ast.add_utf8("replacement"));
        tx.build_detached(|ast| {
            let NodeKindData::IdentifierReference(reference) = ast.kind_data(cloned_source) else {
                unreachable!();
            };
            reference.set_name(ast, replacement_name);
        });
        let NodeKindData::IdentifierReference(original) = tx.ast().kind_data(source) else {
            unreachable!();
        };
        assert_eq!(tx.ast().get_utf8(original.name(tx.ast())), "source");

        tx.discard();
        assert_eq!(ast.node_count(), node_count);
        assert_eq!(ast.syntax_context(source), context);
        verify_program(parse.program, &ast, &semantic).unwrap();
    }

    #[test]
    fn cloned_subtree_can_preserve_declaration_and_reference_ids() {
        fn descendants_postorder(semantic: &Semantic<'_>, root: NodeId, output: &mut Vec<NodeId>) {
            let children: Vec<_> = semantic.node_children(root).collect();
            for child in children {
                descendants_postorder(semantic, child, output);
            }
            output.push(root);
        }

        let allocator = Allocator::new();
        let mut ast = Ast::new("let value; value;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let symbol = semantic.iter_symbols().next().unwrap().id;
        let declaration = semantic
            .declaration_of(semantic.declarations(symbol)[0].node)
            .unwrap();
        let declaration_node = semantic.declaration(declaration).node;
        let reference = semantic.uses(symbol)[0];
        let reference_node = semantic.reference(reference).node;
        let scope = semantic.scope_of(parse.program.node_id());
        let old_statements: Vec<_> = (0..parse.program.body(&ast).len())
            .map(|index| parse.program.body(&ast).get_node(&ast, index).unwrap())
            .collect();
        let mut retired = Vec::new();
        for statement in &old_statements {
            descendants_postorder(&semantic, statement.node_id(), &mut retired);
        }

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let declaration_clone = tx.clone_detached(old_statements[0]);
        let reference_clone = tx.clone_detached(old_statements[1]);
        let cloned_declaration_node = declaration_clone.clone_of(declaration_node).unwrap();
        let cloned_reference_node = reference_clone.clone_of(reference_node).unwrap();
        tx.register_subtree(declaration_clone.root, Some(parse.program.node_id()), scope);
        tx.register_subtree(reference_clone.root, Some(parse.program.node_id()), scope);
        tx.retarget_declaration_node(declaration, cloned_declaration_node);
        tx.retarget_reference_node(reference, cloned_reference_node);
        for node in retired {
            tx.retire_node(node);
        }
        tx.replace_statements(
            StatementListOwner::Program(parse.program),
            [declaration_clone.root, reference_clone.root],
        );
        tx.commit().unwrap();

        assert_eq!(
            semantic.declaration_of(cloned_declaration_node),
            Some(declaration)
        );
        assert_eq!(semantic.declaration(declaration).symbol, symbol);
        assert_eq!(
            semantic.reference_of(cloned_reference_node),
            Some(reference)
        );
        assert_eq!(semantic.reference(reference).symbol, Some(symbol));
        assert_eq!(semantic.uses(symbol), &[reference]);
        assert!(!semantic.is_node_active(declaration_node));
        assert!(!semantic.is_node_active(reference_node));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn detached_clone_replacement_reowners_nested_scopes_and_preserves_ids() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function outer() { let value = 1; function inner() { return value; } return inner(); }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let outer = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_function(&ast))
            .unwrap();
        let body = outer.body(&ast);
        let inner_scope = semantic
            .iter_scopes()
            .find(|entry| {
                entry.scope.node != outer.node_id()
                    && semantic
                        .ancestors(Some(entry.scope.node))
                        .any(|ancestor| ancestor == body.node_id())
                    && entry.scope.kind == swc_next_ecma_semantic::scope::ScopeKind::Function
            })
            .unwrap();
        let inner_scope_id = inner_scope.id;
        let inner_node = inner_scope.scope.node;
        let value_symbol = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"value")
            .unwrap();
        let value_symbol_id = value_symbol.id;
        let value_declaration = semantic
            .declaration_of(semantic.declarations(value_symbol_id)[0].node)
            .unwrap();
        let value_declaration_node = semantic.declaration(value_declaration).node;
        let value_reference = semantic.uses(value_symbol_id)[0];
        let value_reference_node = semantic.reference(value_reference).node;

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let clone = tx.clone_detached(body);
        let cloned_inner = clone.clone_of(inner_node).unwrap();
        let cloned_declaration = clone.clone_of(value_declaration_node).unwrap();
        let cloned_reference = clone.clone_of(value_reference_node).unwrap();
        tx.replace_from_detached_clone(body, clone);
        tx.commit().unwrap();

        assert_eq!(semantic.scope(inner_scope_id).node, cloned_inner);
        assert_eq!(
            semantic.declaration_of(cloned_declaration),
            Some(value_declaration)
        );
        assert_eq!(
            semantic.declaration(value_declaration).symbol,
            value_symbol_id
        );
        assert_eq!(
            semantic.reference_of(cloned_reference),
            Some(value_reference)
        );
        assert_eq!(
            semantic.reference(value_reference).symbol,
            Some(value_symbol_id)
        );
        assert!(!semantic.is_node_active(inner_node));
        assert!(!semantic.is_node_active(value_declaration_node));
        assert!(!semantic.is_node_active(value_reference_node));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn transformed_clone_reconciles_removed_and_generated_scope_regions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function outer() { { let value = 1; consume(value); } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let outer = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_function(&ast))
            .unwrap();
        let body = outer.body(&ast);
        let outer_scope = semantic.scope_of(body.node_id());
        let outer_flags = semantic.scope(outer_scope).flags;
        let block = body
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_block_statement(&ast)
            .unwrap();
        let block_scope = semantic.scope_of(block.node_id());
        let value_symbol = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"value")
            .unwrap()
            .id;
        let value_declaration = semantic
            .declaration_of(semantic.declarations(value_symbol)[0].node)
            .unwrap();
        let value_reference = semantic.uses(value_symbol)[0];

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let clone = tx.clone_detached(body);
        let cloned_block = clone.clone_of(block.node_id()).unwrap();
        let NodeKindData::BlockStatement(cloned_block) = tx.ast().kind_data(cloned_block) else {
            unreachable!();
        };
        let flattened: Vec<_> = (0..cloned_block.body(tx.ast()).len())
            .map(|index| {
                cloned_block
                    .body(tx.ast())
                    .get_node(tx.ast(), index)
                    .unwrap()
            })
            .collect();
        let (generated_function, generated_statement) = tx.build_detached(|ast| {
            let raw = ast.add_utf8("0");
            let value = ast.expr_numeric_literal(
                DUMMY_SP,
                swc_next_ecma_ast::NumericLiteralKind::Decimal,
                raw,
            );
            let return_statement = ast.stmt_return_statement(DUMMY_SP, Some(value));
            let statements = ast.add_typed_sub_range([return_statement]);
            let generated_body = ast.function_body(DUMMY_SP, TypedSubRange::empty(), statements);
            let params = ast.formal_parameters(
                DUMMY_SP,
                TypedSubRange::empty(),
                None,
                swc_next_ecma_ast::FormalParameterKind::FormalParameters,
            );
            let function = ast.function(
                DUMMY_SP,
                swc_next_ecma_ast::FunctionType::FunctionExpression,
                None,
                false,
                false,
                params,
                generated_body,
                None,
                None,
            );
            let statement = ast
                .stmt_expression_statement(DUMMY_SP, swc_next_ecma_ast::Expr::Function(function));
            (function, statement)
        });
        tx.build_detached(|ast| {
            let statements =
                ast.add_typed_sub_range(flattened.into_iter().chain([generated_statement]));
            clone.root.set_body(ast, statements);
        });
        let generated_scope = tx.create_scope(
            outer_scope,
            generated_function.node_id(),
            swc_next_ecma_semantic::scope::ScopeKind::Function,
            outer_flags,
        );
        tx.replace_from_transformed_clone(
            body,
            clone,
            outer_scope,
            &[super::DetachedScopeRegion {
                root: generated_function.node_id(),
                scope: generated_scope.into(),
            }],
        );
        let commit = tx.commit().unwrap();

        assert!(!semantic.is_scope_active(block_scope));
        assert_eq!(semantic.symbol(value_symbol).scope, outer_scope);
        assert_eq!(semantic.reference(value_reference).scope, outer_scope);
        assert_eq!(semantic.declaration(value_declaration).symbol, value_symbol);
        let generated_scope = commit.scope(generated_scope);
        assert_eq!(
            semantic.scope(generated_scope).node,
            generated_function.node_id()
        );
        assert_eq!(semantic.scope(generated_scope).parent, Some(outer_scope));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn preserve_replacement_commits_ast_and_semantic_epochs_together() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("value;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let reference = semantic.iter_references().next().unwrap();
        let old = reference.reference.node;
        let old_reference = reference.id;
        let name = ast.add_utf8("value");

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let new_id = tx
            .build_detached(|ast| ast.identifier_reference(DUMMY_SP, name))
            .node_id();
        tx.replace(old, new_id, IdentityPolicy::Preserve);
        let commit = tx.commit().unwrap();

        assert_eq!(commit.epoch, 1);
        assert_eq!(ast.mutation_epoch(), semantic.mutation_epoch());
        assert_eq!(ast.span(old), DUMMY_SP);
        assert_eq!(semantic.reference_of(old), Some(old_reference));
        assert!(!semantic.is_node_active(new_id));
        verify_program(parse.program, &ast, &semantic).unwrap();
    }

    #[test]
    fn expression_variant_replacement_retires_old_reference_and_registers_descendants() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("source;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let statement = parse.program.body(&ast).get_node(&ast, 0).unwrap();
        let old = statement
            .as_expression_statement(&ast)
            .unwrap()
            .expression(&ast);
        let old_reference = semantic.reference_of(old.node_id()).unwrap();
        let scope = semantic.scope_of(old.node_id());

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let (replacement, generated_reference) = tx.build_detached(|ast| {
            let name = ast.add_utf8("replacement");
            let reference = ast.identifier_reference(DUMMY_SP, name);
            let call = ast.call_expression(
                DUMMY_SP,
                swc_next_ecma_ast::Expr::IdentifierReference(reference),
                TypedSubRange::empty(),
                false,
                None,
            );
            (swc_next_ecma_ast::Expr::CallExpression(call), reference)
        });
        tx.rescope_reference(old_reference, scope);
        tx.retire_reference(old_reference);
        tx.rescope_reference(old_reference, scope);
        tx.replace_expression(old, replacement, IdentityPolicy::Preserve);
        tx.register_subtree_children(replacement, old.node_id(), scope);
        tx.reference(
            scope,
            generated_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(generated_reference.name(tx.ast())),
            ReferenceFlags::default(),
            None::<swc_next_ecma_ast::SymbolId>,
        );
        tx.commit().unwrap();

        assert_eq!(ast.node_kind(old.node_id()), NodeKind::CallExpression);
        assert!(!semantic.is_reference_active(old_reference));
        let generated = semantic
            .reference_of(generated_reference.node_id())
            .unwrap();
        assert_eq!(semantic.reference(generated).scope, scope);
        assert_eq!(
            semantic.parent_of(generated_reference.node_id()),
            Some(old.node_id())
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn failed_preflight_does_not_partially_mutate_attached_state() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("value;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let old = semantic.iter_references().next().unwrap().reference.node;
        let old_span = ast.span(old);
        let node_count = ast.node_count();
        let name = ast.add_utf8("value");

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let new = tx
            .build_detached(|ast| ast.identifier_name(DUMMY_SP, name))
            .node_id();
        tx.replace(old, new, IdentityPolicy::Preserve);
        assert!(matches!(
            tx.commit(),
            Err(MutationError::KindMismatch { old: error_old, new: error_new })
                if error_old == old && error_new == new
        ));

        assert_eq!(ast.mutation_epoch(), 0);
        assert_eq!(semantic.mutation_epoch(), 0);
        assert_eq!(ast.node_count(), node_count);
        assert_eq!(ast.node_kind(old), NodeKind::IdentifierReference);
        assert_eq!(ast.span(old), old_span);
        assert!(semantic.reference_of(old).is_some());
        verify_program(parse.program, &ast, &semantic).unwrap();
    }

    #[test]
    fn reference_rewrite_retires_and_rebinds_atomically() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("let target; old;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let target = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"target")
            .unwrap();
        let target_symbol = target.id;
        let target_name = target.symbol.name;
        let old = semantic
            .iter_references()
            .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"old")
            .unwrap();
        let old_reference = old.id;
        let old_node = old.reference.node;
        let old_scope = old.reference.scope;
        let target_text = ast.add_utf8("target");

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let replacement = tx
            .build_detached(|ast| ast.identifier_reference(DUMMY_SP, target_text))
            .node_id();
        tx.replace(old_node, replacement, IdentityPolicy::Preserve);
        tx.retire_reference(old_reference);
        let pending = tx.reference(
            old_scope,
            old_node,
            target_name,
            Default::default(),
            Some(target_symbol),
        );
        let commit = tx.commit().unwrap();
        let new_reference = commit.reference(pending);

        assert!(!semantic.is_reference_active(old_reference));
        assert_eq!(semantic.reference_of(old_node), Some(new_reference));
        assert_eq!(
            semantic.reference(new_reference).symbol,
            Some(target_symbol)
        );
        assert!(semantic.uses(target_symbol).contains(&new_reference));
        let NodeKindData::IdentifierReference(identifier) = ast.kind_data(old_node) else {
            panic!("replacement did not preserve identifier-reference kind");
        };
        assert_eq!(ast.get_utf8(identifier.name(&ast)), "target");
        verify_program(parse.program, &ast, &semantic).unwrap();
    }

    #[test]
    fn symbol_and_reference_rename_preserve_semantic_ids() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("let a; a;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::GENERATED_NAMES,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let symbol = semantic.iter_symbols().next().unwrap().id;
        let declaration = semantic.declarations(symbol)[0].node;
        let reference = semantic.uses(symbol)[0];
        let reference_node = semantic.reference(reference).node;
        let scope = semantic.symbol(symbol).scope;
        let b = ast.add_utf8("b");
        let b_name = ast.canonicalize_utf8(b);

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let new_declaration = tx.build_detached(|ast| {
            ast.binding_identifier(DUMMY_SP, b, TypedSubRange::empty(), None, false)
        });
        let new_reference = tx.build_detached(|ast| ast.identifier_reference(DUMMY_SP, b));
        tx.replace(
            declaration,
            new_declaration.node_id(),
            IdentityPolicy::Preserve,
        );
        tx.replace(
            reference_node,
            new_reference.node_id(),
            IdentityPolicy::Preserve,
        );
        tx.rename_symbol(symbol, b_name);
        tx.rename_reference(reference, b_name);
        tx.commit().unwrap();

        assert_eq!(semantic.binding(scope, b_name), Some(symbol));
        assert_eq!(semantic.reference(reference).symbol, Some(symbol));
        assert_eq!(semantic.reference(reference).node, reference_node);
        assert_eq!(semantic.uses(symbol), &[reference]);
        verify_program(parse.program, &ast, &semantic).unwrap();
    }

    #[test]
    fn dirty_effects_drop_retired_yield_subtrees() {
        fn descendants_postorder(semantic: &Semantic<'_>, node: NodeId, out: &mut Vec<NodeId>) {
            let children: Vec<_> = semantic.node_children(node).collect();
            for child in children {
                descendants_postorder(semantic, child, out);
                out.push(child);
            }
        }

        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function* f() { arguments; new.target; yield 1; }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::YIELD
                    | SemanticFeatureFlags::GENERATOR_FUNCTIONS
                    | SemanticFeatureFlags::FUNCTION_ENV_CAPTURE,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let body = semantic
            .iter_nodes()
            .find(|node| ast.node_kind(*node) == NodeKind::FunctionBody)
            .unwrap();
        let function = semantic
            .iter_nodes()
            .find(|node| ast.node_kind(*node) == NodeKind::Function)
            .unwrap();
        assert!(
            semantic
                .contains_effect(body, SubtreeEffect::Yield)
                .unwrap()
        );
        let captures = semantic.function_captures(function).unwrap();
        assert_eq!(captures.arguments.len(), 1);
        assert_eq!(captures.new_targets.len(), 1);
        let mut retired = Vec::new();
        descendants_postorder(&semantic, body, &mut retired);
        let body_scope = semantic.scope_of(body);
        let retired_references: Vec<_> = retired
            .iter()
            .filter_map(|node| semantic.reference_of(*node))
            .collect();

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let replacement = tx.build_detached(|ast| {
            let empty = ast.empty_statement(DUMMY_SP);
            let statements = ast.add_typed_sub_range([Stmt::EmptyStatement(empty)]);
            ast.function_body(DUMMY_SP, TypedSubRange::empty(), statements)
        });
        tx.register_subtree_children(replacement, body, body_scope);
        tx.replace(body, replacement.node_id(), IdentityPolicy::Preserve);
        for reference in retired_references {
            tx.retire_reference(reference);
        }
        for node in retired {
            tx.retire_node(node);
        }
        tx.commit().unwrap();

        assert!(
            !semantic
                .contains_effect(body, SubtreeEffect::Yield)
                .unwrap()
        );
        assert!(
            !semantic
                .contains_effect(function, SubtreeEffect::Yield)
                .unwrap()
        );
        let captures = semantic.function_captures(function).unwrap();
        assert!(captures.arguments.is_empty());
        assert!(captures.new_targets.is_empty());
        assert_eq!(semantic.generator_functions_post_order().unwrap().len(), 1);
        verify_program(parse.program, &ast, &semantic).unwrap();
    }

    #[test]
    fn remove_subtree_updates_the_list_and_retires_semantics_atomically() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("first(); second;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let removed = parse.program.body(&ast).get_node(&ast, 0).unwrap();

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        tx.remove_subtree(
            StatementListOwner::Program(parse.program),
            removed.node_id(),
        );
        tx.commit().unwrap();

        assert_eq!(parse.program.body(&ast).len(), 1);
        assert!(!semantic.is_node_active(removed.node_id()));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn move_subtree_updates_parent_scope_and_reference_scope() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("{ first(); } { second(); }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let program_body = parse.program.body(&ast);
        let from = program_body
            .get_node(&ast, 0)
            .unwrap()
            .as_block_statement(&ast)
            .unwrap();
        let to = program_body
            .get_node(&ast, 1)
            .unwrap()
            .as_block_statement(&ast)
            .unwrap();
        let moved = from.body(&ast).get_node(&ast, 0).unwrap();
        let anchor = to.body(&ast).get_node(&ast, 0).unwrap();
        let moved_reference = semantic
            .node_children(moved.node_id())
            .flat_map(|child| semantic.node_children(child))
            .find_map(|node| semantic.reference_of(node))
            .unwrap();
        let new_scope = semantic.scope_of(to.node_id());

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        tx.move_statement_after(
            StatementListOwner::Block(from),
            StatementListOwner::Block(to),
            moved.node_id(),
            anchor.node_id(),
            new_scope,
        );
        tx.commit().unwrap();

        assert!(from.body(&ast).is_empty());
        assert_eq!(to.body(&ast).len(), 2);
        assert_eq!(semantic.parent_of(moved.node_id()), Some(to.node_id()));
        assert_eq!(semantic.scope_of(moved.node_id()), new_scope);
        assert_eq!(semantic.reference(moved_reference).scope, new_scope);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn move_subtree_rejects_a_binding_with_external_uses_atomically() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("{ let x = 1; use(x); } { anchor; }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let program_body = parse.program.body(&ast);
        let from = program_body
            .get_node(&ast, 0)
            .unwrap()
            .as_block_statement(&ast)
            .unwrap();
        let to = program_body
            .get_node(&ast, 1)
            .unwrap()
            .as_block_statement(&ast)
            .unwrap();
        let moved = from.body(&ast).get_node(&ast, 0).unwrap();
        let anchor = to.body(&ast).get_node(&ast, 0).unwrap();
        let new_scope = semantic.scope_of(to.node_id());
        let node_count = ast.node_count();

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        tx.move_statement_before(
            StatementListOwner::Block(from),
            StatementListOwner::Block(to),
            moved.node_id(),
            anchor.node_id(),
            new_scope,
        );
        assert!(matches!(
            tx.commit(),
            Err(MutationError::ExternalSymbolUse { .. })
        ));

        assert_eq!(ast.node_count(), node_count);
        assert_eq!(from.body(&ast).len(), 2);
        assert_eq!(to.body(&ast).len(), 1);
        verify_program(parse.program, &ast, &semantic).unwrap();
    }

    #[test]
    fn generated_helper_subtree_uses_precomputed_scope_layout() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("use();", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let outer_scope = semantic.scope_of(parse.program.node_id());
        let original = parse.program.body(&ast).get_node(&ast, 0).unwrap();

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let (function, function_binding, parameter_binding, parameter_reference) = tx
            .build_detached(|ast| {
                let helper_name = ast.add_utf8("_helper");
                let function_binding = ast.binding_identifier(
                    DUMMY_SP,
                    helper_name,
                    TypedSubRange::empty(),
                    None,
                    false,
                );
                let parameter_name = ast.add_utf8("value");
                let parameter = ast.formal_parameter_pattern_binding_pattern_binding_identifier(
                    DUMMY_SP,
                    parameter_name,
                    TypedSubRange::empty(),
                    None,
                    false,
                );
                let parameter_binding = parameter
                    .as_binding_pattern(ast)
                    .and_then(|pattern| pattern.as_binding_identifier(ast))
                    .unwrap();
                let item = ast.formal_parameter_item_formal_parameter(DUMMY_SP, parameter);
                let items = ast.add_typed_sub_range([item]);
                let params = ast.formal_parameters(
                    DUMMY_SP,
                    items,
                    None,
                    FormalParameterKind::FormalParameters,
                );
                let parameter_reference = ast.identifier_reference(DUMMY_SP, parameter_name);
                let return_statement = ast.return_statement(
                    DUMMY_SP,
                    Some(swc_next_ecma_ast::Expr::IdentifierReference(
                        parameter_reference,
                    )),
                );
                let statements = ast.add_typed_sub_range([Stmt::ReturnStatement(return_statement)]);
                let body = ast.function_body(DUMMY_SP, TypedSubRange::empty(), statements);
                let function = ast.function(
                    DUMMY_SP,
                    FunctionType::FunctionDeclaration,
                    Some(function_binding),
                    false,
                    false,
                    params,
                    body,
                    None,
                    None,
                );
                (
                    function,
                    function_binding,
                    parameter_binding,
                    parameter_reference,
                )
            });
        let creator = DetachedScopeCreator {
            owner: function.node_id(),
            entries: vec![function.node_id()],
            owner_in_scope: true,
            kind: swc_next_ecma_semantic::scope::ScopeKind::Function,
            flags: swc_next_ecma_semantic::scope::ScopeFlags {
                strict: tx.semantic().scope(outer_scope).flags.strict,
                ..Default::default()
            },
        };
        let statement = Stmt::Declaration(Decl::Function(function));
        let layout = tx.create_generated_scope_regions(statement, outer_scope, &[], &[creator]);
        tx.register_generated_subtree_with_layout(
            statement,
            Some(parse.program.node_id()),
            &layout,
        );
        let function_symbol = tx.declare(
            outer_scope,
            function_binding.node_id(),
            tx.ast().canonicalize_utf8(function_binding.name(tx.ast())),
            SymbolFlags::FUNCTION | SymbolFlags::ANNEX_B_FUNCTION,
        );
        let function_scope = layout.scope_of(function.node_id());
        let parameter_symbol = tx.declare(
            function_scope,
            parameter_binding.node_id(),
            tx.ast().canonicalize_utf8(parameter_binding.name(tx.ast())),
            SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::PARAMETER,
        );
        tx.reference(
            function_scope,
            parameter_reference.node_id(),
            tx.ast()
                .canonicalize_utf8(parameter_reference.name(tx.ast())),
            ReferenceFlags::default(),
            Some(parameter_symbol),
        );
        tx.replace_statements(
            StatementListOwner::Program(parse.program),
            [statement, original],
        );
        let commit = tx.commit().unwrap();

        assert_eq!(
            semantic.symbol(commit.symbol(function_symbol)).scope,
            outer_scope
        );
        assert_eq!(
            semantic
                .scope(commit.scope(match function_scope {
                    super::ScopeHandle::Pending(scope) => scope,
                    super::ScopeHandle::Existing(_) => unreachable!(),
                }))
                .parent,
            Some(outer_scope)
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn shared_private_helper_binding_deduplicates_and_avoids_user_names() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("let _helper; use();", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::GENERATED_NAMES,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let outer_scope = semantic.scope_of(parse.program.node_id());
        let originals: Vec<_> = parse
            .program
            .body(&ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .collect();

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let flags = SymbolFlags::FUNCTION | SymbolFlags::ANNEX_B_FUNCTION;
        let first = tx.declare_shared_private(outer_scope, "runtime.helper", "_helper", flags);
        let second = tx.declare_shared_private(outer_scope, "runtime.helper", "_helper", flags);
        assert_eq!(first.symbol, second.symbol);
        assert_eq!(first.declaration, second.declaration);
        assert_eq!(tx.ast().get_utf8(first.text), "_helper1");

        let function = tx.build_detached(|ast| {
            let params = ast.formal_parameters(
                DUMMY_SP,
                TypedSubRange::empty(),
                None,
                FormalParameterKind::FormalParameters,
            );
            let body = ast.function_body(DUMMY_SP, TypedSubRange::empty(), TypedSubRange::empty());
            ast.function(
                DUMMY_SP,
                FunctionType::FunctionDeclaration,
                Some(first.declaration),
                false,
                false,
                params,
                body,
                None,
                None,
            )
        });
        let statement = Stmt::Declaration(Decl::Function(function));
        let creator = DetachedScopeCreator {
            owner: function.node_id(),
            entries: vec![function.node_id()],
            owner_in_scope: true,
            kind: swc_next_ecma_semantic::scope::ScopeKind::Function,
            flags: swc_next_ecma_semantic::scope::ScopeFlags {
                strict: tx.semantic().scope(outer_scope).flags.strict,
                ..Default::default()
            },
        };
        let layout = tx.create_generated_scope_regions(statement, outer_scope, &[], &[creator]);
        tx.register_generated_subtree_with_layout(
            statement,
            Some(parse.program.node_id()),
            &layout,
        );
        tx.replace_statements(
            StatementListOwner::Program(parse.program),
            std::iter::once(statement).chain(originals),
        );
        let commit = tx.commit().unwrap();

        assert_eq!(
            semantic.symbol(commit.symbol(first.symbol)).scope,
            outer_scope
        );
        assert_eq!(
            semantic
                .binding_text(&ast, outer_scope, "_helper1")
                .unwrap(),
            commit.symbol(first.symbol)
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn generated_program_import_recreates_scopes_symbols_and_references() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("use();", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::GENERATED_NAMES,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let outer_scope = semantic.scope_of(parse.program.node_id());
        let original = parse.program.body(&ast).get_node(&ast, 0).unwrap();

        let mut tx =
            MutationTransaction::begin(&mut ast, &mut semantic, parse.program.node_id()).unwrap();
        let generated = tx.build_detached(|ast| {
            Parser::init_generated(
                ast,
                "function _helper(value) { return value; }",
                Options::default(),
                NoTokenParserConfig,
            )
            .parse()
        });
        assert!(
            generated.diagnostics.is_empty(),
            "{:#?}",
            generated.diagnostics
        );
        let imported =
            tx.import_generated_program(generated.program, parse.program.node_id(), outer_scope);
        assert_eq!(imported.statements.len(), 1);
        let helper = imported.statements[0];
        tx.replace_statements(
            StatementListOwner::Program(parse.program),
            [helper, original],
        );
        tx.commit().unwrap();

        assert_eq!(ast.span(helper.node_id()), DUMMY_SP);
        let function = helper
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_function(&ast))
            .unwrap();
        let binding = function.id(&ast).unwrap();
        let symbol = semantic.declaration_of(binding.node_id()).unwrap();
        let symbol = semantic.declaration(symbol).symbol;
        assert_eq!(semantic.symbol(symbol).scope, outer_scope);
        let function_scope = semantic.scope_of(function.node_id());
        assert_eq!(semantic.scope(function_scope).parent, Some(outer_scope));
        let parameter = semantic
            .bindings(function_scope)
            .find(|symbol| ast.get_ast_str_bytes(semantic.symbol(*symbol).name) == b"value")
            .unwrap();
        assert_eq!(semantic.uses(parameter).len(), 1);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
