use super::{TypeScriptOptions, analysis::TypeScriptAnalysis};
use crate::{context::CommentState, used_names::UsedNameState};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{Ast, Program, SourceType, SymbolId, VisitMut};
use swc_next_ecma_semantic::Semantic;

mod bindings;
mod enums;
mod erase;
mod lexical_env;
mod modules;
mod names;
mod namespaces;
mod parameters;
mod runtime;
mod statements;
mod visit;

use namespaces::NamespaceState;
use statements::StatementExpansion;

pub(super) struct TypeScriptLowerer<'visit, 'ast, 'id> {
    ast: &'visit mut Ast<'ast, 'id>,
    options: &'visit TypeScriptOptions,
    analysis: TypeScriptAnalysis<'ast, 'id>,
    semantic: &'visit Semantic<'ast, 'id>,
    semantic_node_count: usize,
    comments: CommentState,
    runtime_binding_scopes: Vec<FxHashSet<SymbolId>>,
    enum_member_symbols: FxHashMap<SymbolId, FxHashSet<SymbolId>>,
    namespaces: NamespaceState<'id>,
    // Lazy index; None as a map value means unresolved or conflicting owners.
    source_name_owners: Option<FxHashMap<&'ast [u8], Option<SymbolId>>>,
    used_names: Option<UsedNameState<'ast>>,
}

impl<'id, 'visit, 'ast> TypeScriptLowerer<'visit, 'ast, 'id> {
    pub(super) fn new(
        options: &'visit TypeScriptOptions,
        analysis: TypeScriptAnalysis<'ast, 'id>,
        ast: &'visit mut Ast<'ast, 'id>,
        semantic: &'visit Semantic<'ast, 'id>,
    ) -> Self {
        let semantic_node_count = ast.node_count();
        Self {
            ast,
            options,
            analysis,
            semantic,
            semantic_node_count,
            comments: CommentState::default(),
            runtime_binding_scopes: Vec::new(),
            enum_member_symbols: FxHashMap::default(),
            namespaces: NamespaceState::default(),
            source_name_owners: None,
            used_names: None,
        }
    }

    pub(super) fn lower_program(mut self, program: Program<'id>) {
        let body = program.body(self.ast());
        if let Some(body) = self.lower_statement_list(body) {
            program.set_body(self.ast(), body);
        }
        if self.ast_ref().source_type() == SourceType::Module
            && !self.has_module_indicator(program.body(self.ast_ref()))
        {
            self.append_esm_marker(program);
        }

        self.comments.flush(self.ast);
    }

    pub(super) fn ast_ref(&self) -> &Ast<'ast, 'id> {
        self.ast
    }
}
