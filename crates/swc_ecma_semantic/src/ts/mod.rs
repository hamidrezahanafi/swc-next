use oxc_index::Idx;
use swc_next_ecma_ast::{Ast, ReferenceId, ScopeId, SymbolId};

use crate::{NameResolution, Semantic as JsSemantic, ecmascript, ts::binder::TsMemberIndex};

pub(crate) mod binder;
pub(crate) mod builder;
pub(crate) mod check;

pub struct TsSemantic<'a> {
    base: JsSemantic<'a>,
    ts_members: TsMemberIndex<'a>,
}

impl<'a> TsSemantic<'a> {
    /// The lexical binding of `name` at `scope` alone, including a hoisting
    /// `var` passing through on its way to its hoist target. This does not walk
    /// the scope chain; see [`Semantic::lookup`]. TypeScript container members
    /// are owned by their merged container environment and are therefore
    /// queried through [`Semantic::lookup`] or the container APIs instead.
    /// Source-file named exports are returned when no separate local binding
    /// owns the name; use [`Semantic::lookup`] to select a declaration space.
    /// Source-file default declarations likewise expose their local names
    /// through [`Semantic::lookup`], with separate visibility in each space.
    ///
    /// ## Example
    /// ```ts
    /// function f() { { var a; } }
    /// //             ^ binding here finds the occurrence owned by
    /// //               the function scope
    /// ```
    pub fn binding(&self, scope_id: ScopeId, name: &[u8]) -> Option<SymbolId> {
        self.base.scope_maps[scope_id.index()]
            .get(name)
            .copied()
            .or_else(|| {
                self.base.hoisting_variables[scope_id.index()]
                    .as_deref()
                    .and_then(|bindings| bindings.get(name).copied())
            })
            .or_else(|| {
                (scope_id == ScopeId::MODULE)
                    .then(|| self.ts_members.source_binding(name))
                    .flatten()
            })
    }

    /// Value bindings associated with a reference's selected bindings.
    /// Includes the alias contributions of a selected public member and follows
    /// local TypeScript ExportValue links without changing lexical identity.
    /// Alias space information determines whether each target is a definite
    /// value or only a possible one. Results are never Unresolved.
    ///
    /// This queries value meaning, not runtime usage: a `typeof` or explicit
    /// type-only export can still name a value. Access flags and emit options
    /// determine whether it is needed at runtime. Import aliases remain bindings;
    /// this does not replace them with their final property or declaration target.
    /// `ast` must be the tree analyzed to produce this model.
    pub fn reference_value_targets<'s>(
        &'s self,
        ast: &'s Ast<'_>,
        id: ReferenceId,
    ) -> impl Iterator<Item = NameResolution> + 's {
        let reference = self.reference(id);
        let name = ast.get_utf8(reference.name).as_bytes();
        let member_values = self.ts_members.reference_values(id);
        let lexical_values = member_values
            .is_none()
            .then_some(())
            .into_iter()
            .flat_map(move |_| self.reference_candidates(id))
            .filter_map(move |symbol| {
                let resolution = if reference.flags.is_uncertain() {
                    NameResolution::Uncertain { candidate: symbol }
                } else {
                    NameResolution::Resolved(symbol)
                };
                self.ts_members.value_target(
                    &self.symbols,
                    &self.declarations,
                    &self.scope_maps,
                    resolution,
                    name,
                    reference.flags.space.value_target_space(),
                )
            });
        member_values
            .into_iter()
            .flatten()
            .copied()
            .chain(lexical_values)
    }
}

impl<'a> TsSemantic<'a> {
    /// The definite terminal symbol of an internal TypeScript import alias.
    /// Follows forwarding aliases without changing their binding identities.
    /// Returns `None` for non-aliases, opaque or cyclic targets, property-only
    /// targets, and merged contributions without a unique terminal symbol.
    /// The target retains its declaration kind, including `CONST_ENUM`, regardless
    /// of JavaScript emit options. This does not decide runtime access or emission.
    pub fn ts_import_alias_target(&self, alias: SymbolId) -> Option<SymbolId> {
        self.ts_members.import_alias_target(alias)
    }

    /// The initializer's root reference in an internal `import A = N.Member`.
    /// Its access flags account for the whole alias target, not just `N`:
    /// a known type-only target does not read the root, while opaque targets
    /// conservatively retain a read. Ambient and explicit type-only aliases
    /// also have no runtime access. External `require` targets return `None`.
    ///
    /// This describes initializer access, not whether the alias is used or
    /// whether an emit option preserves its target. Use ordinary reference
    /// queries for declaration dependencies even when this reference is unread.
    /// `ast` must be the tree analyzed to produce this model.
    pub fn import_equals_reference(
        &self,
        ast: &Ast<'_>,
        declaration: swc_next_ecma_ast::TsImportEqualsDeclaration,
    ) -> Option<ReferenceId> {
        let node = ecmascript::import_equals_reference_node(ast, declaration)?;
        self.base.reference_of(node)
    }

    /// Members declared by any declaration merged into a TypeScript namespace.
    pub fn ts_namespace_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.ts_members.namespace_members(owner)
    }

    /// Finds a member declared by any declaration merged into a TypeScript namespace.
    /// This returns its identity; its declaration flags do not describe its
    /// resolved visibility in every reference space or member view.
    pub fn ts_namespace_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.ts_members.namespace_member(owner, name)
    }

    /// Members declared by any declaration merged into a TypeScript enum.
    pub fn ts_enum_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.ts_members.enum_members(owner)
    }

    /// Finds a member declared by any declaration merged into a TypeScript enum.
    /// This returns its identity; a merged symbol can also have declarations
    /// whose visibility belongs to a different member view.
    pub fn ts_enum_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.ts_members.enum_member(owner, name)
    }
}
