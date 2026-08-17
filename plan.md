# Semantic-aware AST Mutation 与 ES2015 Class Transform 执行计划

## 目标

建立 MutationTransaction，作为 transformer 修改 AST 与 Semantic 的唯一入口。

最终要求：

1. Semantic 始终描述当前 AST，而不是 parser 产生的原始 AST。
2. transform 只表达 lowering 意图，不再散落调用 AST setter 和 Semantic mutation API。
3. NodeId、SymbolId、ReferenceId、ScopeId 在语义身份不变时保持稳定。
4. transaction commit 后，parent、scope、binding、reference、use-list、transform facts 和
   module record 全部与 AST 一致。
5. 不采用生产环境整树重新 analyze 的短期方案。
6. ES2015 class transform 直接构建在最终 transaction 模型上。

## 最终执行流

~~~text
Transform pass
  读取旧 AST 与 Semantic facts
       ↓
MutationTransaction::begin(root)
  构造 detached AST
  声明 binding/reference/scope 意图
  记录 replace/insert/remove/move
       ↓
preflight
  校验 identity、scope、parent、binding 冲突
       ↓
commit
  修改 AST edges
  更新 Semantic graph
  更新 effects/captures/module record
       ↓
AST epoch == Semantic epoch
       ↓
后续 pass 读取完全一致的 AST 与 Semantic
~~~

## 一、定义 AST 与 Semantic 的同步契约

涉及文件：

- crates/swc_ecma_ast/src/lib.rs
- crates/swc_ecma_ast/src/visit.rs
- crates/swc_ecma_semantic/src/lib.rs
- crates/swc_ecma_semantic/src/scope.rs
- crates/swc_ecma_transformer/src/state.rs
- crates/swc_ecma_transformer/src/lib.rs

### 核心不变量

1. 所有可达 AST node 都必须在 Semantic 中处于 active 状态。
2. 所有不可达 AST node 都不能从 Semantic 的公共 iterator 和 lookup API 返回。
3. 删除对象不重排 ID，使用 inactive/tombstone 状态。
4. 新对象只 append，现有 ID 不因 transform 重编号。
5. replacement 必须显式声明 identity 策略：
   - Preserve：语义身份不变，只改变 AST 表现。
   - Replace：旧身份失效，创建新身份。
   - Move：身份稳定，parent 或 scope 改变。
6. transaction 外禁止修改已挂载 AST 的结构。
7. detached node 可以在 transaction 内构造，但 commit 前不能连接到当前树。
8. transaction 返回前必须完成所有 Semantic side table 和 transform facts 更新。
9. enabled semantic feature 对应的索引必须保持一致；未启用 feature 不承担额外维护成本。

transform 后的 Semantic 表示当前输出 AST 的语义，而不是源代码语义快照。如果源 class binding
在输出中仍代表同一个运行时 binding，可以保留 SymbolId，但必须更新 flags、declaration kind、
scope map 和相关引用。仅保留 NodeId 而继续使用旧 class 语义是不允许的。

## 二、把 Semantic 改造成可变 ID 图

涉及文件：

- crates/swc_ecma_semantic/src/lib.rs
- crates/swc_ecma_semantic/src/binder.rs
- crates/swc_ecma_semantic/src/context.rs
- crates/swc_ecma_semantic/src/scope.rs
- crates/swc_ecma_semantic/src/module_record.rs

当前 API 主要支持 append 和少量 rename/rebind。class transform 需要完整的对象生命周期管理。

### Node 生命周期

增加内部 API：

~~~rust
register_node(node, parent, scope);
retire_node(node);
reparent_node(node, parent);
rescope_node(node, scope);
~~~

要求：

- node state 明确区分 unregistered、active、inactive。
- node_parents、node_scopes、node_declarations、node_references 同步更新。
- 重新注册 inactive NodeId 必须被拒绝，除非使用明确的 identity restore 协议。

### Scope 生命周期

增加内部 API：

~~~rust
create_scope(parent, owner, kind, flags) -> ScopeId;
retag_scope(scope, kind, flags);
reparent_scope(scope, parent);
retire_scope(scope);
~~~

ScopeTree 增加：

- active state；
- children adjacency；
- 可维护的 parent；
- 可重新计算的 hoist_target；
- scope owner node 的反向索引。

reparent 或 retag 后，必须更新该 scope 及受影响后代的 hoist_target。

### Symbol、Declaration 和 Reference 生命周期

增加内部 API：

~~~rust
declare(scope, node, name, flags) -> SymbolId;
retire_declaration(declaration);
retire_symbol(symbol);
update_symbol_flags(symbol, flags);
move_symbol(symbol, scope);

create_reference(scope, node, name, flags, symbol) -> ReferenceId;
retire_reference(reference);
rename_reference(reference, name);
rebind_reference(reference, symbol);
rescope_reference(reference, scope);
~~~

要求：

- symbols、declarations、references 使用稳定 append-only ID。
- declaration/use-list 支持增删，不依赖不可修改的连续 range。
- scope_maps 和 hoisting_variables 支持删除、迁移和 retag。
- dynamic reference、with 和 direct eval 的保守语义不能因迁移被削弱。
- iterator 和查询 API 自动跳过 inactive entry。

### 名字占用索引

将名字状态拆成两部分：

1. live-name count：精确反映当前 AST 中的名字占用。
2. generated-name watermark：单调增长，保证跨 transaction、跨 pass 的生成名稳定。

删除或迁移 binding/reference 时更新 live count，但不回退 allocation watermark。

## 三、实现 MutationTransaction

组合层放在 transformer crate，避免 AST crate 依赖 Semantic。

~~~text
swc_ecma_ast
  AST storage、NodeId、通用 traversal
       ↓
swc_ecma_semantic
  Semantic graph、受控 mutation primitives
       ↓
swc_ecma_transformer
  MutationTransaction<Ast + Semantic>
~~~

新增目录：

- crates/swc_ecma_transformer/src/mutation/mod.rs
- crates/swc_ecma_transformer/src/mutation/journal.rs
- crates/swc_ecma_transformer/src/mutation/builder.rs
- crates/swc_ecma_transformer/src/mutation/commit.rs
- crates/swc_ecma_transformer/src/mutation/verify.rs

### 核心结构

~~~rust
pub struct MutationTransaction<'tx, 'ast> {
    ast: &'tx mut Ast<'ast>,
    semantic: &'tx mut Semantic<'ast>,
    root: NodeId,
    journal: MutationJournal,
    dirty_roots: DirtyRoots,
    pending_scopes: Vec<PendingScope>,
    pending_symbols: Vec<PendingSymbol>,
    pending_references: Vec<PendingReference>,
}
~~~

transaction 内使用 PendingScopeId、PendingSymbolId 等本地 handle。commit 时统一分配正式 Semantic
ID，避免构造 AST 过程中提前暴露半完成的 Semantic。

### 对 transform 暴露的结构操作

~~~rust
tx.replace(old, new, IdentityPolicy);
tx.insert_before(anchor, nodes);
tx.insert_after(anchor, nodes);
tx.remove_subtree(root);
tx.move_subtree(root, new_parent, new_scope);
~~~

禁止 transform 直接修改已挂载节点的 parent edge 或 statement/expression list。

### Semantic-aware builder

~~~rust
tx.declare_private(scope, "_super", flags) -> PrivateBinding;
tx.binding_identifier(binding) -> BindingIdentifier;
tx.reference(binding, flags) -> IdentifierReference;
tx.unresolved_reference(scope, name, flags) -> IdentifierReference;
tx.create_scope(parent, kind, owner) -> PendingScopeId;
tx.retag_scope(scope, kind);
tx.reparent_scope(scope, parent);
tx.retire_scope(scope);
~~~

PrivateBinding 至少包含：

- 输出文本；
- SyntaxContext；
- pending 或 existing symbol handle；
- declaration scope；
- symbol flags。

SyntaxContext 负责 hygiene identity；SymbolId 和 Semantic indexes 负责语义解析。两者不能互相替代。

## 四、实现原子 commit

commit 固定按以下顺序执行：

~~~text
normalize dirty roots
       ↓
preflight
  parent/scope/identity/binding 冲突校验
       ↓
记录 old subtree summaries
       ↓
retire 被删除的 references/declarations/nodes/scopes
       ↓
应用 AST edge 修改
       ↓
创建、retag、reparent scopes
       ↓
递归登记新增或移动的 nodes
       ↓
提交 declarations
       ↓
提交 references 并解析 pending symbol handles
       ↓
更新 binding/hoisting/used-name/module indexes
       ↓
重算 dirty roots 的 effects/captures/function-order
       ↓
递增并对齐 AST/Semantic epoch
~~~

### Preflight

所有可能失败的检查在修改 AST 和 Semantic 前完成：

- replacement edge 是否接受新节点类型；
- old/new identity policy 是否有效；
- parent 和 scope 是否 active；
- move 是否跨越不支持的 semantic boundary；
- binding 是否冲突；
- scope retag 是否会产生非法 hoist hierarchy；
- pending reference 的目标 binding 是否存在；
- 同一个 node 是否被多个 journal operation 修改。

通过 preflight 后，commit 阶段只执行确定性操作。

### Dirty roots

- 如果 dirty root A 包含 dirty root B，只保留 A。
- moved subtree 同时污染 old parent path 和 new parent path。
- scope retag/reparent 同时污染 scope subtree 和相关 binding/reference lookup。
- effect delta 只传播到正确的 function/arrow boundary。

避免每个 leaf mutation 都沿祖先链更新，防止 O(edits × depth)。

## 五、同步 Transform Facts

涉及文件：

- crates/swc_ecma_semantic/src/context.rs
- crates/swc_ecma_semantic/src/lib.rs

每个 active node 保存本地 contribution，dirty root commit 时重新聚合 subtree summary：

- yield/await effect count；
- effect owner；
- generator function post-order；
- async function post-order；
- function captures；
- class lowering facts；
- module record contribution。

删除旧 subtree 时扣除 old summary，插入新 subtree 时增加 new summary。跨 function boundary 的
move 必须重新计算 owner，不能仅修改 node_parent。

## 六、接入 TransformerCtx

涉及文件：

- crates/swc_ecma_ast/src/visit.rs
- crates/swc_ecma_transformer/src/state.rs
- crates/swc_ecma_transformer/src/lib.rs

目标：

1. Semantic 不再单独存放在 TransformerImpl。
2. TransformerState/TransformerCtx 同时访问 AST、Semantic、enabled features 和 scope stack。
3. 新增 ctx.transaction(root, closure)。
4. 已有节点进入 hook 时，从 semantic.scope_of(node) 初始化并校验 current scope。
5. 新 scope 只能由 transaction 注册。
6. Transformer 中不再直接传播带结构修改的 TraverseAction::Replace。

### Mutation epoch

AST 和 Semantic 分别保存 mutation epoch：

~~~rust
AstMutationEpoch(u64)
SemanticMutationEpoch(u64)
~~~

规则：

- transaction begin 时 epoch 必须一致。
- detached node construction 不增加 attached-tree epoch。
- commit 完成后同时递增两个 epoch。
- Semantic 查询、pass 边界和 debug verifier 检查 epoch 一致。
- 绕过 transaction 的 structural mutation 会在最近的查询或 pass 边界失败。

AST crate 只提供通用 mutation guard/epoch hook，组合逻辑位于 transformer crate。

## 七、迁移 Generator 与 FnEnvHoister

涉及文件：

- crates/swc_ecma_transformer/src/es2015/generator/compiler/mod.rs
- crates/swc_ecma_transformer/src/es2015/generator/compiler/expression.rs
- crates/swc_ecma_transformer/src/es2015/generator/compiler/statement.rs
- crates/swc_ecma_transformer/src/es2015/util/fn_env_hoister.rs

迁移内容：

1. 删除 transform 内零散的 register_node、declare、reference_generated、reparent_node、
   rename_reference 和 rebind_reference。
2. 所有 private binding 通过 tx.declare_private 创建。
3. state function、parameter、helper reference、普通 statement/expression 整棵生成树递归登记。
4. 捕获 arguments/new.target 时使用 transaction 的 rename/rebind/move 操作。
5. generator 完成后，通过 transaction 删除旧 yield contribution 和 generator facts。
6. FnEnvHoister 注入 program/function/block 时使用 insert 和 move API。

完成标准：

- generator transform 后任意可达 node 都能安全查询 scope_of 和 parent_of。
- 所有生成 reference 都能查询 reference_of 和 symbol_of。
- 增量 Semantic 与测试环境重新 analyze 的标准化结果一致。
- transaction commit 不扫描完整 program。

## 八、增加 Class Lowering Semantic Facts

新增聚合 feature：

~~~rust
SemanticFeatureFlags::CLASS_LOWERING
~~~

内部收集：

- class post-order 列表；
- class node、name binding、class scope、expression-name scope；
- constructor 和 method function scope；
- heritage expression；
- computed key 的求值顺序；
- super()；
- super.prop、super[prop] 的 get/set/call；
- class self-reference；
- constructor 中的 this/new.target；
- nested class boundary；
- decorators、property definitions、private elements、static blocks 是否已被前置 pass 清理。

不暴露多个容易漏配的 class feature。对 class transform 使用完整的 CLASS_LOWERING feature，内部
再按需分配 side table。

class 按 post-order lowering，避免外层 transaction 先破坏嵌套 class 的 facts。

## 九、明确 Class Transform Pipeline

涉及文件：

- crates/swc_ecma_transformer/src/lib.rs
- crates/swc_ecma_transformer/src/es2015/mod.rs
- crates/swc_ecma_transformer/src/es2022/mod.rs
- crates/swc_ecma_transformer/src/proposal/mod.rs

把 transform 执行顺序改为显式依赖图，不依赖 enter_program 中的偶然调用顺序。

~~~text
decorators / TypeScript lowering
       ↓
class properties / private methods / static blocks
       ↓
arrow、async、generator 等方法体前置要求
       ↓
ES2015 class transform
       ↓
输出中不再存在 Class AST
~~~

具体方法体 transform 的前后顺序由语义测试确定并写入 pass prerequisites。任何要求 Class AST
存在的 pass 必须在 class transform 前执行。

## 十、基于 Transaction 实现 ES2015 Class Transform

新增建议目录：

- crates/swc_ecma_transformer/src/es2015/classes/mod.rs
- crates/swc_ecma_transformer/src/es2015/classes/analysis.rs
- crates/swc_ecma_transformer/src/es2015/classes/plan.rs
- crates/swc_ecma_transformer/src/es2015/classes/lower.rs
- crates/swc_ecma_transformer/src/es2015/classes/super.rs
- crates/swc_ecma_transformer/src/es2015/classes/tests.rs

### Class lowering 执行流

~~~text
Semantic class facts
  class/name/scope/constructor/method/super/computed keys
       ↓
ClassLoweringPlan
  输出 AST 结构
  helper/private binding
  scope transition
  identity policy
       ↓
MutationTransaction
  构造 detached constructor/method/helper tree
       ↓
replace class expression/declaration
insert helper/init statements
move preserved method/constructor subtrees
       ↓
commit
  Semantic 与 AST 同步
~~~

### 基础 Class

实现：

- anonymous/named class expression；
- class declaration；
- 默认和显式 constructor；
- instance/static method；
- getter/setter；
- helper 注入；
- class strict mode。

### ScopeTransitionPlan

每个 class 先构建明确的 scope transition：

~~~rust
ClassScopePlan {
    reuse: Vec<(ScopeId, NewScopeRole)>,
    reparent: Vec<(ScopeId, ScopeId)>,
    retire: Vec<ScopeId>,
    create: Vec<PendingScope>,
}
~~~

原则：

- method function scope 尽量保留。
- 显式 constructor 的 function scope 尽量保留。
- synthetic constructor 创建新 scope。
- named class expression 的 expression-name scope 根据输出结构复用或迁移。
- class scope 不能无条件当作 function scope，必须显式 retag。
- 移动 subtree 时同步更新 root parent、node scopes 和 scope parent。
- scope role 改变后重新计算 hoist_target 和 strict flags。

### Derived Class

实现：

- heritage expression 只求值一次；
- _super 等 private binding；
- inheritance/helper 调用；
- super()；
- super property get/set/call；
- derived constructor 的 this 初始化；
- constructor return substitution；
- extends null；
- new.target。

不能假设 super() 一定是 constructor 的第一个 statement。lowering 必须支持控制流中的合法 super
调用，并正确处理 this 初始化状态。

### 求值顺序与语义身份

覆盖：

- computed method keys 及其 side effect 顺序；
- heritage 与 computed key 的相对求值顺序；
- named class expression TDZ；
- class self-reference；
- static/instance element 顺序；
- nested class；
- export class/export default class；
- direct eval/with；
- TypeScript class 与 interface merge 的前置 pipeline。

### Helper 和生成名字

所有 helper alias、super capture 和 computed-key temp 使用 tx.declare_private。

统一 helper registry 必须保证：

- 同一 program 只注入一次；
- helper declaration/reference 全部进入 Semantic；
- 与用户 binding 冲突时使用 generated-name allocator；
- helper 插入位置不破坏 directive、module 和 scope 边界。

## 十一、一致性验证器

新增测试工具：

~~~rust
assert_semantic_matches_reanalysis(program, ast, incremental_semantic);
~~~

重新 analyze 只作为测试 oracle，不进入生产 transform。

标准化比较：

- 每个可达 node 的 parent 和 scope；
- scope kind、parent、hoist target、flags；
- declaration node、name、flags；
- reference node、name、flags、目标 declaration node；
- symbol uses；
- binding/hoisting lookup；
- live-name index；
- yield/await effects；
- captures；
- generator/async/class post-order；
- module record；
- inactive entry 不从公共 API 返回。

比较不依赖新建对象的绝对 ID 顺序；通过 node identity、declaration node 和 scope path 建立映射。

## 十二、Class 测试矩阵

至少覆盖：

1. 无 constructor 和显式 constructor。
2. named/anonymous class expression。
3. nested class。
4. base/derived/extends null。
5. heritage 和 computed key side effects。
6. constructor/method 中引用 class 名。
7. conditional super()。
8. super get/set/call。
9. derived constructor return object/primitive。
10. new.target。
11. helper 同名冲突。
12. direct eval 和 with。
13. export class/export default class。
14. static/instance method、getter/setter。
15. generator/async method。
16. arrow 中的 this/arguments/super/new.target。
17. class properties/private methods/static blocks 的 pipeline 组合。
18. TypeScript class、decorator 和 interface merge 的前置 pipeline。
19. Babel fixtures。
20. 相关 Test262 用例。

## 十三、性能验证

增加 benchmark 和统计：

- transaction commit 时间；
- dirty root 数量；
- 新增/失效 node 数量；
- 新增/迁移/失效 scope 数量；
- symbol/reference mutation 数量；
- effect delta 传播长度；
- class 数量和 nested class 深度扩展曲线；
- generator/class 组合 transform；
- transaction 相对 transform 本身的开销。

性能约束：

1. commit 不扫描完整 program。
2. 普通 leaf replacement 与受影响 subtree/ancestor path 相关。
3. 批量 class lowering 对同一 dirty root 只汇总一次。
4. generated-name allocation 均摊 O(1)。
5. disabled feature 不创建对应 side table，也不参与 commit 更新。

## 十四、迁移剩余 Transform 并完成架构收口

在 generator 和 class 验证 transaction 后，迁移所有仍修改结构的 transform：

- destructuring；
- block scoping；
- parameters；
- async；
- class properties/private methods；
- optional chaining；
- object rest/spread；
- helper 注入相关 pass；
- 其他使用 raw builder/setter/TraverseAction::Replace 的 pass。

最终收口：

1. transformer 内禁止直接调用 AST structural setter。
2. Semantic 原始 mutation primitives 降为 crate-private。
3. transform 只能通过 MutationTransaction。
4. pass 边界强制检查 epoch。
5. debug/test 模式运行 transaction invariant verifier。
6. CI 运行 incremental Semantic 与 reanalysis 的标准化对比。

## 提交拆分

保持每个提交可独立编译，不 amend，不把 Semantic 图重构与 class lowering 塞进同一个提交。

1. feat(semantic): introduce active mutable semantic graph
2. feat(semantic): add scope lifecycle and rescope APIs
3. feat(semantic): add symbol reference retirement and migration
4. feat(semantic): make name indexes mutation aware
5. feat(transformer): add mutation transaction journal
6. feat(transformer): add semantic-aware AST builders
7. feat(transformer): implement atomic transaction commit
8. feat(semantic): update transform facts from transaction deltas
9. refactor(transformer): move semantic into transformer context
10. feat(transformer): add AST and semantic mutation epochs
11. refactor(transformer): migrate generator transform
12. refactor(transformer): migrate function environment hoister
13. test(transformer): compare incremental semantic with reanalysis
14. feat(semantic): collect class lowering facts
15. refactor(transformer): make transform ordering dependency driven
16. feat(transformer): implement base ES2015 class lowering
17. feat(transformer): implement class scope transition plans
18. feat(transformer): implement derived classes and super lowering
19. test(transformer): port Babel class transform fixtures
20. test(transformer): add class Test262 conformance coverage
21. perf(transformer): benchmark mutation transactions
22. refactor(transformer): migrate remaining structural transforms
23. refactor(transformer): prohibit raw structural AST mutation

## 最终完成标准

1. 所有 transformer structural mutation 都通过 MutationTransaction。
2. 任意 pass 完成后，Semantic 可以完整查询当前 AST。
3. class transform 后不存在 stale class scope、declaration、reference 或 transform facts。
4. 任意生成 node 都有正确 parent 和 scope。
5. 任意生成 binding/reference 都进入 scope map、declaration/use-list 和名字索引。
6. stable ID 在语义身份保留时不变化。
7. transaction commit 不触发完整 AST/Semantic rebuild。
8. incremental Semantic 与测试 oracle 的重新 analyze 结果一致。
9. Babel class fixtures 和相关 Test262 用例通过。
10. benchmark 证明 transaction 成本与实际 dirty subtree 规模相关。
