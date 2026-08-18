# Class Transform 单遍 Semantic 与 AstContext 执行计划

## 目标

修正当前 ES2015 class transform 的职责划分：开启 `classes` 后，class lowering 所需的全部
结构与语义信息由 semantic 在原有的一次 AST 遍历中按需收集；transform 只消费 semantic
产出的 class facts 并执行 lowering，不再自行 analysis 或扫描 AST。

所有 attached AST 的创建、替换、插入、删除、移动和字段修改都通过 AstContext。AstContext
在同一个操作/事务内更新 AST 与 Semantic，不允许 pass 在修改 AST 后再遍历语法树修补
Semantic。

最终必须满足：

1. `SemanticFeatureFlags::CLASS_LOWERING` 关闭时，不创建 class facts，也不承担 class 收集成本。
2. 开启后，semantic 的首次 visitor 同时完成 scope/symbol/reference 和 class transform facts
   收集；finalize 只能解析已记录的 ID/candidate，不能重新遍历 AST 或 Semantic node graph。
3. class transform 中删除 `analysis.rs`，不再扫描 program body、class body、constructor body、
   method body或 `semantic.node_children`。
4. class transform 不再使用 `Visit`/`VisitMut` 查找或改写 class 内部节点；所有 rewrite site
   都来自 semantic，所有改动都通过 AstContext 的定点 mutation API。
5. AstContext commit 不再用 dirty-root AST 重扫来同步 transform facts，而是使用 mutation
   journal 中的 old/new contribution delta 更新 Semantic。
6. 任意 AstContext 操作返回后，AST/Semantic epoch、parent、scope、binding、reference、
   use-list、module record 与所有已开启的 transform facts 保持一致。

## 当前问题

### Transformer 仍在做 analysis

`crates/swc_ecma_transformer/src/es2015/classes/mod.rs` 在 `Classes::transform` 中连续调用：

- `analysis::analyze_program`
- `analysis::analyze_named_expressions`
- `analysis::analyze_anonymous_expressions`
- `analysis::analyze_anonymous_default_exports`

`crates/swc_ecma_transformer/src/es2015/classes/analysis.rs` 随后又执行：

- program/statement-list 扫描与 class placement 判断；
- class member 支持性检查和 getter/setter 分组；
- named class expression 的 heritage/computed-key TDZ reference DFS；
- super member parent/use-kind 分析；
- constructor 内 `super()`/`this` 的 ancestor 过滤；
- statement-list owner 查找。

这意味着 semantic 虽然已有 `ClassLoweringFact`，transform 仍要重新构造四类
`*ClassPlan`，而且存在多次线性 `.find` 和重复 ancestor walk。

### Semantic facts 也不是单遍完成

`crates/swc_ecma_semantic/src/context.rs` 当前只在 visitor 中记录 class node。到
`TransformFactsCollector::finish` 后，再逐个调用 `build_class_lowering_fact`：

- 再遍历 class body 收集 method/property/static block；
- 再通过 `semantic.node_children` DFS class 子树；
- finalize 后再沿 ancestors 计算 `class_counts`。

所以当前的 `CLASS_LOWERING` 只是“先记住 class，再二次分析”，没有达到一次遍历收集。

### Lowering 仍包含扫描和裸 AST 改动

`crates/swc_ecma_transformer/src/es2015/classes/lower.rs` 当前还有：

- `ConstructorFlowAuditor` 对 constructor body 的只读遍历；
- `SuperMemberRewriter`、`DerivedConstructorRewriter` 对 detached clone 的 `VisitMut` 遍历；
- 为每条顶层 statement 重复调用 `audit_constructor_flow`；
- 直接调用 typed setter、`set_syntax_context` 和 `&mut Ast` builder；
- 为匹配 plan 再扫描 program/nested statement lists。

`crates/swc_ecma_transformer/src/es2015/classes/helpers.rs` 还会 parse helper source 后重新调用
`analyze`，再把生成 program 导入 transaction。这同样绕开“semantic 一次收集、AstContext
同步更新”的目标模型。

### Mutation commit 仍依赖重扫

`crates/swc_ecma_semantic/src/lib.rs` 的 `prepare_dirty_transform_facts`、
`refresh_dirty_transform_facts` 和 `refresh_function_lists` 会在 transaction 前后遍历 dirty
subtree，并重新构建 class facts。即使 class transform 删除自己的 analysis，这条路径仍会
造成 transform-time 重复遍历。

## 目标执行流

~~~text
TransformerOptions
  classes=true -> SemanticFeatureFlags::CLASS_LOWERING
       ↓
Semantic 原有 visitor（唯一一次 AST analysis traversal）
  class stack / method stack / constructor control-flow stack
  同步收集 placement、scope、member、rewrite site、evaluation order
  binder 同步记录 declaration/reference ID
       ↓
Semantic finalize
  只解析已记录的 candidate ID
  生成 ClassTransformInfo post-order + NodeId 索引
  不遍历 AST，不遍历 semantic.node_children
       ↓
ES2015 Classes
  semantic.class_transform_infos_post_order()
  直接按 info 计算 helper 集合并执行 lowering
  不运行 analysis visitor，不扫描 program/class/function body
       ↓
AstContext transaction
  定点 replace/insert/remove/move/build
  journal 同时记录 Semantic delta
       ↓
AstContext commit
  原子应用 AST edge + Semantic graph + transform-fact delta
  不对 dirty subtree 重新 Visit/DFS
       ↓
AST epoch == Semantic epoch
~~~

## 一、定义 Semantic 提供给 class transform 的完整输入

涉及文件：

- `crates/swc_ecma_semantic/src/lib.rs`
- `crates/swc_ecma_semantic/src/context.rs`
- `crates/swc_ecma_semantic/src/checker.rs`

保留一个聚合 feature：

~~~rust
SemanticFeatureFlags::CLASS_LOWERING
~~~

不拆成 `CLASS_METHODS`、`SUPER_CALLS` 等多个 flag，避免调用方开启 class transform 时漏配。

将当前偏原始的 `ClassLoweringFact` 扩展为 transform 可直接消费的
`ClassTransformInfo`。该结构保持 semantic-neutral：只描述源 AST、语义身份、求值顺序和
rewrite site，不包含 helper 名称或输出 AST 节点。

建议结构如下，最终字段名可按 Rust lifetime/arena 约束调整：

~~~rust
pub struct ClassTransformInfo<'a> {
    pub class: Class,
    pub placement: ClassPlacement,
    pub statement: Option<Stmt>,
    pub statement_list_owner: Option<NodeId>,
    pub export: Option<ClassExportPlacement>,

    pub class_scope: ScopeId,
    pub name_scope: Option<ScopeId>,
    pub outer_scope: ScopeId,
    pub binding: Option<BindingIdentifier>,
    pub declaration: Option<DeclarationId>,
    pub symbol: Option<SymbolId>,

    pub heritage: Option<Expr>,
    pub constructor: ConstructorTransformInfo<'a>,
    pub methods: ArenaVec<'a, MethodGroupInfo>,
    pub definition_tdz_self_references: ArenaVec<'a, NodeId>,
    pub super_accesses: ArenaVec<'a, SuperAccessInfo>,

    pub flags: ClassTransformFlags,
    pub unsupported: Option<ClassUnsupportedReason>,
}
~~~

`ClassPlacement` 至少区分：

- declaration；
- named class expression；
- anonymous class expression；
- named default export declaration；
- anonymous default export declaration；
- named export declaration；
- 当前 lowering 不支持的 placement。

Semantic 只保存 statement-list owner 的 `NodeId`。transform 需要 transaction owner 时，
通过一次 `ast.kind_data(owner)` 转成 `StatementListOwner`，不能再次搜索 statement list。

### Member facts

`MethodGroupInfo` 直接保存 lowering 所需的最终分组：

- constructor 单独保存，重复/非法 constructor 标记 unsupported；
- data method 保持 source order；
- 同名 getter/setter 在 semantic 收集期间完成分组；
- computed getter/setter 不合并，保持当前行为；
- method key 在 semantic 中归一化为 identifier/string/numeric/computed 四类；
- 保存 definition、function、function scope、static/kind/computed；
- 保存 generator/async/TS annotation/decorator/function-body-scope 等支持性结论。

transform 不再重新读取整个 class body 来判断 `body.len() == methods.len()`，semantic 直接记录：

- `has_properties`
- `has_private_elements`
- `has_static_blocks`
- `has_decorators`
- `has_unsupported_ts_shape`
- `has_unsupported_method_shape`

### Rewrite-site facts

Semantic 必须把目前 lowering visitor 运行时才推导的信息提前记录：

~~~rust
pub struct ConstructorTransformInfo<'a> {
    pub kind: ConstructorKind,
    pub definition: Option<MethodDefinition>,
    pub function: Option<Function>,
    pub scope: Option<ScopeId>,
    pub super_calls: ArenaVec<'a, NodeId>,
    pub this_nodes: ArenaVec<'a, NodeId>,
    pub new_targets: ArenaVec<'a, NodeId>,
    pub returns: ArenaVec<'a, ConstructorReturnInfo>,
    pub abrupt_completions: ArenaVec<'a, AbruptCompletionInfo>,
    pub labels: ArenaVec<'a, AstUtf8>,
    pub top_level_statements_with_return: ArenaVec<'a, NodeId>,
}

pub struct SuperAccessInfo {
    pub member: NodeId,
    pub parent: NodeId,
    pub usage: SuperUsage,
    pub computed: bool,
    pub optional: bool,
    pub method: MethodDefinition,
}
~~~

`SuperUsage` 至少区分 get、call、simple set、compound set、logical set、prefix update、postfix
update 和 unsupported tagged/optional form。需要哪些 `_super_base/_super_prop/_super_value/
_super_old` temp 也作为 usage 派生字段记录，`prepare_super_write_temps` 不再查看 parent AST。

constructor control-flow 信息在 semantic visitor 已有 control-target/finalizer 路径上同步收集。
每个 return/break/continue 保存 lowering 所需的 target、label、finalizer depth 和是否跨越
completion boundary。由此删除 `ConstructorFlowAuditor`，也不需要 lowering 再遍历 constructor。

### Support decision

所有“当前是否能 lower”的判断集中在 semantic info 中，并保留明确原因，不再在 transform
里静默 `continue`。`ClassUnsupportedReason` 至少覆盖：

- decorators/properties/private/static block 未被前置 pass 清理；
- TS-only class/method shape；
- generator/async method；
- optional/tagged super access；
- direct eval 与 computed key 的当前限制；
- scope/placement/parent 不变量不满足；
- TDZ self-reference 出现 write；
- 不支持的 class element 或 constructor 形态。

公开查询提供：

~~~rust
semantic.class_transform_infos_post_order()
    -> Result<&[ClassTransformInfo<'_>], FeatureNotEnabled>;
semantic.class_transform_info(class: NodeId)
    -> Result<Option<&ClassTransformInfo<'_>>, FeatureNotEnabled>;
~~~

post-order slice 是 transform 的唯一 class worklist；NodeId 索引用于测试和其他 pass 定点查询，
不允许调用方通过 `.iter().find(...)` 建立二次索引。

## 二、在 Semantic 的同一次遍历中完成收集

涉及文件：

- `crates/swc_ecma_semantic/src/context.rs`
- `crates/swc_ecma_semantic/src/checker.rs`
- `crates/swc_ecma_semantic/src/binder.rs`

### Collector 状态

当 `CLASS_LOWERING` 开启时，`TransformFactsCollector` 增加：

~~~rust
class_stack: Vec<ClassInfoBuilder>,
method_stack: Vec<MethodContext>,
constructor_control_stack: Vec<ControlContext>,
class_infos_post_order: ArenaVec<ClassTransformInfo>,
class_info_by_node: ArenaVec<Option<ClassInfoId>>,
~~~

收集规则：

1. enter class 时创建 builder 并压栈，同时记录 parent/placement/scope phase。
2. enter/leave method、property、static block 时只更新栈顶 class；进入 nested class 后，内部
   事件只属于 nested builder，天然实现 nested class boundary。
3. visit `super`、`this`、`new.target`、return、label、break/continue 时，按当前 class、method、
   arrow/function boundary 和 control stack 直接归属到对应 rewrite-site list。
4. binding/reference 创建时由 binder 把 `DeclarationId`/`ReferenceId` candidate 回传给
   collector；不要在 finalize 重新从 AST 查找 identifier。
5. leave class 时弹出 builder，形成 post-order draft。

### Finalize 约束

finalize 可以：

- 将 candidate `ReferenceId` 解析到最终 `SymbolId`；
- 过滤 named class 自引用与 TDZ definition reference；
- 把临时 `Vec` 搬入 allocator-backed `ArenaVec`；
- 校验已记录的 scope/parent/symbol ID；
- 根据已记录字段得出 `unsupported` 和 helper requirement flags。

finalize 不可以：

- 调用 `class.body(ast).body(ast)` 遍历 elements；
- 调用 `semantic.node_children`/`semantic.ancestors` 收集 class 内容；
- 对每个 class 再运行 visitor；
- 通过 program body 查找 statement/export/owner；
- 重新分析 constructor control flow。

删除 `build_class_lowering_fact` 的 AST/semantic DFS 实现。mutation 时如需重建单个 class info，
也必须由 AstContext journal 中的 typed field/member/rewrite-site delta 更新，不能复用旧函数重扫。

### Post-order 与计数

在 visitor leave class 时自然 append post-order；`class_counts` 使用每个 node 的本地
contribution 在同一次 leave-node 聚合，不再在 finish 对每个 class 沿 ancestors 累加。

## 三、建立唯一的 AstContext 修改入口

涉及文件：

- `crates/swc_ecma_transformer/src/state.rs`
- `crates/swc_ecma_transformer/src/mutation/mod.rs`
- `crates/swc_ecma_transformer/src/mutation/journal.rs`
- `crates/swc_ecma_transformer/src/mutation/builder.rs`
- `crates/swc_ecma_transformer/src/mutation/commit.rs`
- `crates/swc_ecma_ast/src/visit.rs`

组合层仍放在 transformer crate，避免 `swc_ecma_ast` 反向依赖 semantic crate。对 pass 暴露的
对象统一命名/表达为 AstContext；现有 `MutationTransaction` 可以保留为内部 journal/commit
实现，但 class lowering 不应再直接取得裸 `&mut Ast`。

建议入口：

~~~rust
ctx.ast_context().transaction(root, |ast| {
    ast.build_generated(...);
    ast.replace_expression(...);
    ast.replace_statements(...);
    ast.move_subtree(...);
    ast.declare_private(...);
    ast.reference(...);
});
~~~

AstContext API 分为：

- 只读查询：node data、span、string pool、Semantic facts；
- typed builder：创建普通 node/binding/reference/scope，同时记录本地 semantic contribution；
- attached edit：replace/insert/remove/move/retag/reparent；
- detached edit：clone/build/rewrite，但 origin、parent、scope、identity 和 contribution 在创建时
  一并登记；
- commit：原子应用 journal。

禁止 class transform：

- 直接调用 `tx.build_detached(|ast: &mut Ast| ...)`；
- 持有 `&mut Ast` 的自定义 `VisitMut`；
- 直接调用 typed setter 或 `Ast::set_syntax_context`；
- 创建 identifier 后再单独补 `tx.reference/tx.declare`；
- 修改 attached edge 后再调用 semantic repair API。

为常用场景增加高层原子 API，例如：

~~~rust
ast.generated_binding(scope, base, flags) -> PrivateBinding;
ast.generated_reference(scope, binding, flags) -> IdentifierReference;
ast.replace_at(node, replacement, IdentityPolicy);
ast.replace_statement_list(owner, edits);
ast.clone_with_origins(root) -> DetachedClone;
ast.rewrite_clone_node(clone, origin, replacement);
ast.move_preserving_identity(root, parent, scope);
~~~

`replace_statement_list` 接收按 anchor/NodeId 描述的 edits，不要求调用方先复制并扫描整个
statement list。AstContext 内部利用 owner 的 edge index 完成 splice，并同步 parent/order facts。

## 四、用 contribution delta 同步 Semantic，删除 commit 重扫

涉及文件：

- `crates/swc_ecma_semantic/src/lib.rs`
- `crates/swc_ecma_semantic/src/context.rs`
- `crates/swc_ecma_transformer/src/mutation/journal.rs`
- `crates/swc_ecma_transformer/src/mutation/commit.rs`

为每个 active node 保存 enabled feature 的本地 contribution/aggregate summary：

- yield/await count 与 effect owner；
- generator/async/class count；
- function capture candidates；
- class-info owner/member/rewrite-site contribution；
- post-order worklist identity/order metadata。

每个 AstContext operation 在 journal 中同时记录 old/new semantic delta：

~~~rust
TransformFactEdit::Attach { node, parent, contribution }
TransformFactEdit::Detach { node, parent, summary }
TransformFactEdit::Move { node, old_parent, new_parent, summary }
TransformFactEdit::Retarget { old, new }
TransformFactEdit::UpdateClassField { class, old, new }
~~~

commit 使用已保存的 root summary 向 old/new ancestor path 增减计数，并定点更新 worklist/index。
不能再次收集 dirty subtree。

需要删除或改写以下重扫路径：

- `prepare_dirty_transform_facts`
- `refresh_dirty_transform_facts`
- `collect_node_postorder` 在 facts refresh 中的使用
- `refresh_effect_owners`
- `refresh_function_lists`
- mutation 后重新调用 `build_class_lowering_fact`

删除/移动 subtree 为维护 node scope/reference scope 而遍历 Semantic adjacency 是允许的，但不能
读取 AST 重新做语法分类；后续可用 scope-region summary 进一步批量化。attached AST 不得被
`Visit/VisitWith` 二次扫描。

## 五、让 class transform 直接消费 Semantic info

涉及文件：

- `crates/swc_ecma_transformer/src/es2015/classes/mod.rs`
- `crates/swc_ecma_transformer/src/es2015/classes/analysis.rs`
- `crates/swc_ecma_transformer/src/es2015/classes/plan.rs`
- `crates/swc_ecma_transformer/src/es2015/classes/lower.rs`

### 删除 analysis 层

`Classes::transform` 改为：

1. 取得 `class_transform_infos_post_order()`；feature 缺失是配置错误，不返回空数组掩盖。
2. 过滤 `unsupported.is_none()`，unsupported 原因仅用于诊断/debug 统计。
3. 从 info flags 一次汇总 program helper requirements。
4. 按 semantic 给出的 post-order 逐个 lower。

删除 `analysis.rs`。`plan.rs` 只保留真正的输出/transaction plan；若某结构与
`ClassTransformInfo` 一一对应，则删除重复 plan 类型，lower 直接借用 semantic info。

不能再出现：

- `program.body(...).iter()` 用于找 class；
- `classes.iter().find(...)`；
- `statement_list(...).find(...)`；
- `references_to_symbol_in_subtrees`；
- `supported_super_members`/`has_super_writes`；
- `supported_expression_members`；
- `constructor_nodes` ancestor 过滤；
- 用 ancestor depth 重新排序 expression plan。

post-order 已保证 nested class 先于 outer class。placement/owner index 已保证声明替换定点发生，
不需要重新扫描 program 或 nested owner。

### 删除 lowering 中的 analysis visitor

用 semantic rewrite sites 替代：

- `ConstructorFlowAuditor`
- `audit_constructor_flow`
- `SuperMemberRewriter`
- `DerivedConstructorRewriter` 的全子树 `VisitMut`

lowering 根据 `SuperAccessInfo`、constructor returns、abrupt completions、this/super/new.target
列表，调用 AstContext 定点替换。需要包裹 statement/control-flow 时，由 semantic info 直接
给出 owner/anchor/finalizer context，AstContext 完成 edge splice。

保留 lowering 的代码生成逻辑，例如 super get/set 表达式、completion wrapper、helper call
和 scope transition；删除其中“为了找到改写位置而遍历”的部分。

### Scope 与 identity

每个 info 预先包含 class/name/outer/method/constructor scope。lowering 仍显式声明：

- 哪个 class scope retag/reparent/retire；
- synthetic constructor 创建哪个 scope；
- existing method/function scope 如何复用；
- moved key/body 保留还是替换 NodeId；
- binding 的 class flags 如何改为 function flags；
- export/module record 如何 retarget。

这些意图只提交给 AstContext，不允许 lower 直接调用 Semantic mutation primitive。

## 六、移除 helper 的 runtime parse + analyze

涉及文件：

- `crates/swc_ecma_transformer/src/es2015/classes/helpers.rs`
- `crates/swc_ecma_transformer/build.rs`（如采用 build-time template）
- `crates/swc_ecma_transformer/src/mutation/builder.rs`

当前 `publish_source` 每次 parse helper，再单独 `analyze` helper program。改为 build-time 生成或
显式定义的 `HelperTemplate`：

- template 包含 AST construction recipe；
- binding/reference/scope role 在 template 中明确标注；
- 实例化时由 AstContext 分配私有名字、SyntaxContext、SymbolId、DeclarationId、ReferenceId；
- helper statements 通过 program statement-list edit 插入；
- 同一 program 的 helper registry 去重。

运行时不再对 helper 调用 `analyze`，也不再通过 `import_generated_program` 扫描生成 AST 来猜测
semantic identity。

如果 build-time template 作为独立提交过大，可以先手写现有 class helper 的 construction
recipe；不能以保留 runtime reanalysis 作为最终状态。

## 七、测试与防回归

### Semantic 单遍收集测试

在 `crates/swc_ecma_semantic/src/lib.rs` 扩展现有 class facts 测试，覆盖：

- declaration/named expression/anonymous expression/export placement 与 owner；
- nested class post-order 与 outer class boundary；
- constructor/method/accessor grouping 和 source order；
- heritage/computed key TDZ self-reference；
- super get/call/set/compound/logical/update/unsupported usage；
- constructor `this`、`super()`、`new.target`、return、label、break/continue/finally context；
- decorator/property/private/static/TS/async/generator unsupported reason；
- `CLASS_LOWERING` 关闭时返回 `FeatureNotEnabled` 且不创建 side table。

增加 test-only traversal instrumentation：开启 `CLASS_LOWERING` 后，每个 AST node 的 semantic
enter/leave 只发生一次；finalize 的 class facts 路径不得读取 children/运行 visitor。

### AstContext 同步测试

在 `crates/swc_ecma_transformer/src/mutation` 覆盖：

- attach/detach/move/replace class 或含 nested class 的 subtree；
- class member字段定点替换；
- transform facts post-order/index/count 的 delta；
- generated binding/reference/scope 即时可查询；
- commit 后 `assert_semantic_matches_reanalysis` 一致；
- transaction 失败时 AST 与 Semantic 都不变。

测试 oracle 可以重新 analyze，生产代码不能。

### Class transform 回归

保留 `crates/swc_ecma_transformer/src/es2015/classes/mod.rs` 的现有 runtime、Babel 与 Test262
覆盖，并增加架构断言：

- classes 模块不包含 `mod analysis`；
- classes 生产代码不调用 `Visit`、`VisitMut`、`visit_with`、`visit_mut_with`；
- classes 生产代码不调用 `analyze`；
- classes 生产代码不接收裸 `&mut Ast` 或调用 attached typed setter；
- transform facts commit 路径不调用 AST visitor/`build_class_lowering_fact`。

建议验证命令：

~~~bash
cargo test -p swc_next_ecma_semantic class_lowering
cargo test -p swc_next_ecma_transformer es2015::classes
cargo test -p swc_next_ecma_transformer mutation
cargo clippy -p swc_next_ecma_semantic -p swc_next_ecma_transformer --all-targets -- -D warnings
~~~

按仓库约定，不运行会卡住的 storage/native watcher 测试。

## 八、性能验收

增加 class transform 分阶段统计或 benchmark，至少覆盖 1、32、128 个 class，以及深层 nested
class：

- semantic 首次遍历；
- class facts finalize；
- class lowering；
- AstContext commit；
- helper 实例化。

验收条件：

1. semantic 对 AST node 只做一次 analysis visitor traversal。
2. facts finalize 与 candidate/reference 数量相关，不与 class subtree 总节点数再次线性相关。
3. class transform 不出现 AST analysis/visitor traversal。
4. commit 成本与 journal edit 数量、受影响 ancestor path 和必须 rescope 的 semantic entries
   相关，不重新扫描 dirty AST subtree。
5. 多 class 场景不再有 `facts.iter().find`、statement-list `.find`、重复 ancestors 导致的
   O(C²) 或 O(C × subtree) 增长。
6. helper 不再产生 runtime parse + semantic analyze 成本。

## 实施顺序与提交拆分

每个提交独立编译，不 rebase，不 amend：

1. ✅ `refactor(semantic): collect complete class transform info in one traversal` (325064f)
   - 引入 builder stacks、placement/member/rewrite-site/control-flow facts；
   - finalize 只解析 candidates；
   - 注意：`build_class_lowering_fact` 仍保留供 dirty refresh 路径使用。
2. ✅ `test(semantic): cover class transform info and single-pass collection` (77bbafd)
   - 完整 facts 矩阵与 traversal instrumentation。
3. ✅ `refactor(transformer): expose semantic-aware ast context mutations` (b083558)
   - class 所需 typed builder、定点 rewrite、statement-list edit API；
   - pass 不再获得裸 `&mut Ast`。
4. ❌ `refactor(semantic): update transform facts from ast context deltas`
   - journal contribution；
   - 删除 dirty-root facts refresh 重扫。
5. ✅ `refactor(transformer): consume semantic class transform info` (3af1317)
   - 删除 `classes/analysis.rs`；
   - 合并/精简 `plan.rs`；
   - 直接消费 post-order infos。
6. ⏳ `refactor(transformer): replace class visitors with targeted ast context rewrites`
   - ✅ 删除 ConstructorFlowAuditor（Visit trait），改用 typed AST 遍历；
   - ❌ 删除 SuperMemberRewriter 与 DerivedConstructorRewriter（仍基于 VisitMut）；
   - ❌ 所有生成和改动通过 AstContext。
7. ❌ `refactor(transformer): instantiate class helpers without reanalysis`
   - helper template/recipe 与 registry；
   - 删除 runtime `analyze`/generated program semantic import。
8. ✅ `test(transformer): enforce class semantic and ast context architecture`
   - 架构断言（classes 模块无 analysis 模块、无 Visit 引用库）；
   - 现有 fixtures 全部通过。
9. ❌ `perf(transformer): benchmark single-pass class lowering`
   - 记录 semantic/finalize/lower/commit/helper 分项与规模曲线。

## 最终完成标准

- [ ] `CLASS_LOWERING` facts 在 semantic 原有的一次 AST 遍历内收集完成。
- [x] semantic finalize 不遍历 AST/class subtree/Semantic node graph。
- [x] `crates/swc_ecma_transformer/src/es2015/classes/analysis.rs` 已删除。
- [ ] class transform 只迭代 semantic post-order info，不扫描 program/class/constructor/method。
- [ ] class transform 生产代码没有 `Visit`/`VisitMut` 和裸 attached AST setter。
- [ ] helper 发布不运行 parser + semantic analyze。
- [ ] 所有 class lowering mutation 经过 AstContext，并在同一 commit 同步 Semantic。
- [ ] transform facts 使用 journal delta 更新，不执行 dirty AST subtree refresh。
- [ ] commit 后增量 Semantic 与测试 oracle 的重新 analyze 结果一致。
- [x] 现有 class Rust tests、Babel fixtures、相关 Test262 和 clippy 通过。
- [ ] benchmark 显示 class 数量增长时 semantic 只遍历一次，transform/commit 不引入重复
      AST 扫描。
