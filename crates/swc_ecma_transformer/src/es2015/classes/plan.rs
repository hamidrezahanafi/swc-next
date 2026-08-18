use swc_next_ecma_ast::{
    AstUtf8, AstWtf8, BindingIdentifier, Class, ClassBody, ExportDefaultDeclaration,
    ExportNamedDeclaration, Expr, Function, IdentifierReference, MethodDefinition,
    MethodDefinitionKind, NodeId, NumericLiteralKind, ScopeId, Stmt, SymbolId,
};
use swc_next_ecma_semantic::{DeclarationId, SymbolFlags};

use crate::mutation::StatementListOwner;

#[derive(Debug, Clone)]
pub(super) struct ClassLoweringPlan {
    pub statement: Stmt,
    pub owner: StatementListOwner,
    pub export: Option<ClassExport>,
    pub class: Class,
    pub body: ClassBody,
    pub binding: BindingIdentifier,
    pub declaration: DeclarationId,
    pub symbol: SymbolId,
    pub output_flags: SymbolFlags,
    pub class_scope: ScopeId,
    pub outer_scope: ScopeId,
    pub heritage: Option<Expr>,
    pub super_members: Vec<NodeId>,
    pub has_super_writes: bool,
    pub constructor_super_calls: Vec<NodeId>,
    pub constructor_this_nodes: Vec<NodeId>,
    pub constructor: ConstructorPlan,
    pub methods: Vec<MethodGroupPlan>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ClassExport {
    Named(ExportNamedDeclaration),
    Default(ExportDefaultDeclaration),
}

impl ClassExport {
    pub fn node_id(self) -> NodeId {
        match self {
            Self::Named(node) => node.node_id(),
            Self::Default(node) => node.node_id(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ClassExpressionPlan {
    pub class: Class,
    pub body: ClassBody,
    pub binding: BindingIdentifier,
    pub declaration: DeclarationId,
    pub symbol: SymbolId,
    pub output_flags: SymbolFlags,
    pub class_scope: ScopeId,
    pub name_scope: ScopeId,
    pub outer_scope: ScopeId,
    pub heritage: Option<Expr>,
    pub definition_tdz_self_references: Vec<NodeId>,
    pub super_members: Vec<NodeId>,
    pub has_super_writes: bool,
    pub constructor_super_calls: Vec<NodeId>,
    pub constructor_this_nodes: Vec<NodeId>,
    pub constructor: ConstructorPlan,
    pub methods: Vec<MethodGroupPlan>,
}

#[derive(Debug, Clone)]
pub(super) struct AnonymousClassExpressionPlan {
    pub class: Class,
    pub body: ClassBody,
    pub class_scope: ScopeId,
    pub outer_scope: ScopeId,
    pub heritage: Option<Expr>,
    pub super_members: Vec<NodeId>,
    pub has_super_writes: bool,
    pub constructor_super_calls: Vec<NodeId>,
    pub constructor_this_nodes: Vec<NodeId>,
    pub constructor: ConstructorPlan,
    pub methods: Vec<MethodGroupPlan>,
}

#[derive(Debug, Clone)]
pub(super) struct AnonymousDefaultClassPlan {
    pub statement: Stmt,
    pub export: ExportDefaultDeclaration,
    pub class: Class,
    pub body: ClassBody,
    pub class_scope: ScopeId,
    pub outer_scope: ScopeId,
    pub heritage: Option<Expr>,
    pub super_members: Vec<NodeId>,
    pub has_super_writes: bool,
    pub constructor_super_calls: Vec<NodeId>,
    pub constructor_this_nodes: Vec<NodeId>,
    pub constructor: ConstructorPlan,
    pub methods: Vec<MethodGroupPlan>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ConstructorPlan {
    Synthetic,
    Existing {
        definition: MethodDefinition,
        function: Function,
        scope: ScopeId,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MethodPlan {
    pub definition: MethodDefinition,
    pub function: Function,
    pub scope: ScopeId,
    pub r#static: bool,
    pub kind: MethodDefinitionKind,
    pub key: MethodKeyPlan,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum MethodKeyPlan {
    Identifier(AstUtf8),
    String {
        value: AstWtf8,
        raw: AstUtf8,
    },
    Numeric {
        kind: NumericLiteralKind,
        raw: AstUtf8,
    },
    Computed(Expr),
    CapturedComputed(IdentifierReference),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum MethodGroupPlan {
    Data(MethodPlan),
    Accessor {
        get: Option<MethodPlan>,
        set: Option<MethodPlan>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ScopeTransitionPlan {
    pub retire: ScopeId,
    pub create_parent: ScopeId,
}

impl ClassLoweringPlan {
    pub fn scope_transition(&self) -> ScopeTransitionPlan {
        ScopeTransitionPlan {
            retire: self.class_scope,
            create_parent: self.outer_scope,
        }
    }
}
