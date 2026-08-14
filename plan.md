# Option-driven Semantic and Transform-time AST Context Plan

## 目标

让 semantic 在首次 AST 遍历时，根据实际开启的 transform options，按需收集后续 pass
需要的结构信息（例如子树是否包含 `yield`/`await`）以及名字、scope、binding、reference
信息。transform 不再为这些信息二次扫描 AST。

所有 transform 对 AST 的创建、插入、替换、删除、字段修改都通过统一的 AST context；
context 在 AST 修改成功的同一事务内同步更新 `Semantic`。generator 等后续 pass 可以始终
查询当前 AST 对应的 semantic，而不是使用“transform 前快照”。

本计划只处理 transform 期间可增量维护的语义信息：

- 新增 binding、reference 和普通 AST node。
- 给已有 reference 重新绑定 `SymbolId`。
- 在已有 scope 中分配不会冲突的临时变量名。
- 修改已有 symbol 的名字，同时保持 declaration/use 的 ID 关系。
- 新节点的 `node_scope`、`node_parent`、`node_declaration`、`node_reference` 索引。
- 按 options 开启的 subtree effect summary，例如 `yield`、`await`。
- generator/async transform 需要的函数边界、候选函数和 function-environment capture 索引。

跨 scope 移动任意已有子树、删除已有 scope、重新解释 `with`/direct eval 等行为不放进
第一阶段；这些操作需要单独的局部重新解析协议。

## 目标执行流

```text
Parser
  AST StringPool 保存源码字符串、解码字符串和 transform 生成字符串
       ↓
读取 TransformerOptions
  计算 SemanticFeatureFlags
  generator=true -> YIELD + GENERATOR_FUNCTIONS + FUNCTION_ENV_CAPTURE
  async transform=true -> AWAIT + ASYNC_FUNCTIONS + FUNCTION_ENV_CAPTURE
       ↓
Semantic analysis
  字符串 canonicalize 为 pooled AstStr
  建立 scope/symbol/reference 索引
  仅为开启的 feature 建立 node -> subtree effects/function facts
  一次性建立 hoist scope -> used names/next suffix 索引
       ↓
Transform AST context
  同时持有 &mut Ast、&mut Semantic、enabled features、current scope/node path
       ↓
ctx.mutate(...)
  创建/插入/替换/删除 AST node
  在同一事务内更新 parent/scope/symbol/reference/subtree effects
       ↓
ctx.declare_generated(scope, base, flags)
  分配 pooled name + hygiene context
  追加 SymbolId/DeclarationId
  更新 scope binding 和 used-name 索引
       ↓
ctx.reference_generated(symbol, flags)
  创建引用节点并追加 ReferenceId
  更新 symbol uses 和 node 索引
       ↓
后续 transform
  直接查询已经更新的 Semantic，不重新扫描 AST/semantic
```

## 不变量

1. `SymbolId`、`DeclarationId`、`ReferenceId` 和 `ScopeId` 创建后保持稳定。
2. 所有会影响 binding resolution 的 transform 操作必须经过 `TransformerCtx`。
3. 不只 binding/reference：任何可能改变已开启 semantic feature 的 AST 修改，都必须经过
   AST context。pass 不能直接持有 `&mut Ast` 调用裸 setter/builder 绕过同步。
4. context 返回一个新 node 时，对应的 node/scope/declaration/reference/effect 索引已经
   可查询，不能留下“稍后补 semantic”的中间状态。
5. 临时变量名一旦分配，必须立即写入目标 scope 和 used-name 索引；后续 pass 立即能看到。
6. 名字比较使用 canonical pooled handle；热路径不复制 `Vec<u8>`，也不比较字符串内容。
7. hygiene 负责区分 transform 生成的私有 binding；scope name index 仍负责输出文本避碰。
8. 未开启的 feature 不分配 side table、不执行对应的 enter/leave 收集逻辑，也不承担后续
   mutation 维护成本。
9. effect summary 必须服从语言边界：`yield` 不穿过 nested function/arrow，`await` 不穿过
   nested async/function 边界；不能简单把整棵语法树的 bit 向根传播。
10. 删除或替换节点时不能只存 bool。使用计数或 old/new subtree delta，确保同一子树存在多个
    `yield`/`await` 时可以正确清除和保留祖先摘要。

## Phase 1：统一字符串身份

涉及文件：

- `crates/swc_ecma_ast/src/strings.rs`
- `crates/swc_ecma_ast/src/lib.rs`
- `crates/swc_ecma_semantic/src/binder.rs`
- `crates/swc_ecma_semantic/src/lib.rs`

任务：

- [ ] 在 AST crate 增加 `AstStr`，统一表达 UTF-8 和 WTF-8 名字。
- [ ] 扩展 `StringPool` 的存储来源，明确支持：源码 slice、parser 解码字符串、其他模块/
      transform 创建的字符串。
- [ ] 增加名字 canonicalization API；相同字节序列和编码类型只得到一个稳定的 pooled
      handle。
- [ ] semantic 建立 symbol/reference 时将名字 canonicalize；AST identifier 字段仍可保留
      原始 source slice，以免破坏 span/source 输出。
- [ ] 将 `Symbol.name`、`Reference.name`、scope binding map 的 key 迁移到 canonical
      `AstStr`。
- [ ] `Semantic::binding`/`lookup` 提供 pooled-name 热路径；仅在外部文本查询入口做一次
      intern/转换。
- [ ] 验证 `AstStr` 的 `Hash`/`Eq` 只比较稳定 handle，不读取或复制字符串内容。

完成标准：semantic scope lookup、symbol/reference name lookup 和临时名索引不再使用
`&[u8]`/`Vec<u8>` 作为内部 key。

## Phase 2：从 transform options 派生可选 semantic features

涉及文件：

- `crates/swc_ecma_transformer/src/lib.rs`
- `crates/swc_ecma_transformer/src/es2015/mod.rs`
- `crates/swc_ecma_semantic/src/lib.rs`
- `crates/swc_ecma_semantic/src/context.rs`
- `crates/swc_ecma_semantic/src/checker.rs`
- `tasks/testsuite/src/runner/babel_transform.rs`
- `tasks/benchmark/benches/generator_transform.rs`

任务：

- [ ] 在 semantic crate 定义不依赖 transformer crate 的 `SemanticFeatureFlags`。第一批包含
      `YIELD`、`AWAIT`、`GENERATOR_FUNCTIONS`、`ASYNC_FUNCTIONS` 和
      `FUNCTION_ENV_CAPTURE`；后续可以继续增加 `THIS`、`SUPER` 等结构事实。
- [ ] 为 `TransformerOptions` 增加 `semantic_features()`：根据真正启用的 transform 开关
      生成 flags。generator 关闭时不请求 yield/generator facts；async transform 关闭时不
      请求 await/async facts。
- [ ] 扩展 `AnalyzeOptions` 接收 `features`。调用方必须先解析/构建
      `TransformerOptions`，再把 `options.semantic_features()` 传给 `analyze`；semantic
      crate 不反向依赖 transformer crate。
- [ ] 调整 Babel runner、benchmark 和正式编译入口的顺序，避免当前“先 analyze，后读取
      transform options”的流程。
- [ ] `SemanticContext` 只为开启的 feature 初始化 collector/side table；关闭时分支应在
      visitor 外层或初始化阶段消除，不能在每个 node 上做无意义工作。
- [ ] `check_syntax` 与 transform feature 解耦。即使 syntax check 需要观察 `yield`/`await`
      做 early error，也不代表要保留 transform 使用的 subtree side table。

建议接口：

```rust
let transformer_options = load_transformer_options(...);
let semantic_features = transformer_options.semantic_features();
let semantic = analyze(
    program,
    &ast,
    AnalyzeOptions {
        check_syntax,
        features: semantic_features,
    },
);
```

完成标准：generator-only 配置只收集 generator 所需事实；完全不启用相关 transform 时，
semantic 的时间和内存不包含 yield/await transform summary。

## Phase 3：在 Semantic 中建立可选的 subtree effect summary

涉及文件：

- `crates/swc_ecma_semantic/src/lib.rs`
- `crates/swc_ecma_semantic/src/context.rs`
- `crates/swc_ecma_semantic/src/checker.rs`
- `crates/swc_ecma_semantic/src/binder.rs`

任务：

- [ ] 为开启的 feature 建立按 `NodeId` 查询的 `SubtreeEffects`。至少支持
      `contains_yield(node)` 和 `contains_await(node)`，并保留内部 count/delta，而不是只存
      一个无法正确删除的 bool。
- [ ] 在 semantic 已有的单次 visitor enter/leave 中做后序汇总，不启动第二个 visitor；
      disabled feature 对应的 collector 不创建。
- [ ] 明确定义传播边界：yield 只传播到所属 function/arrow 边界；await 按 async transform
      的实际语义传播到所属 function/arrow 边界；class field/static block 等特殊边界与现有
      generator 行为保持一致。
- [ ] 同一次遍历中按需记录 generator/async function 的 post-order 列表，使嵌套函数可以按
      lowering 所需顺序处理，而不再由 transform 扫描整棵 AST。
- [ ] `FUNCTION_ENV_CAPTURE` 开启时，利用 semantic 已解析的 references 和当前函数边界，
      建立每个目标函数的 unresolved `arguments`、`new.target` 等 capture 索引；不要在
      generator `analysis.rs` 中再访问全部节点收集。
- [ ] collector 遍历时可以先记录 capture candidate 及 boundary owner；等 binder 完成
      resolution 后在 semantic finalize 阶段按 `ReferenceId` 过滤 unresolved `arguments`。
      这里允许遍历 candidate/ID 数组，但不能再次遍历 AST。
- [ ] 为 effect/function/capture side table 提供只读查询 API，并在 feature 未开启时返回
      明确的 `FeatureNotEnabled`/debug assertion，避免误以为 `false` 表示语法不存在。
- [ ] 记录每个 effect summary 的 boundary owner，供 transform mutation 时沿正确祖先路径
      更新，避免新建 nested function 中的 yield 污染外层函数。

建议接口：

```rust
semantic.contains_effect(node, SubtreeEffect::Yield) -> bool;
semantic.contains_effect(node, SubtreeEffect::Await) -> bool;
semantic.generator_functions_post_order() -> &[Function];
semantic.function_captures(function) -> FunctionCaptures<'_>;
```

完成标准：generator transform 删除独立 `YieldFacts`/`YieldCollector` 之后，所有原来
`contains_yield(subtree)` 的判断都可直接查询 semantic，且 transform 阶段没有预分析 AST
遍历。

## Phase 4：把 Semantic 改为可增量维护的 ID 图

涉及文件：

- `crates/swc_ecma_semantic/src/lib.rs`
- `crates/swc_ecma_semantic/src/binder.rs`
- `crates/swc_ecma_semantic/src/context.rs`
- `crates/swc_ecma_semantic/src/scope.rs`

任务：

- [ ] 为 `Semantic` 增加受控的 mutation API，不直接公开内部 arena。
- [ ] 支持向已有 scope 追加 symbol/declaration/reference，并同步扩展所有 `node_*`
      side tables。
- [ ] 将 declaration/use 的只读压缩 range 改成支持按 symbol 增量追加的索引结构；保留
      `declarations(symbol)` 和 `uses(symbol)` 的稳定查询语义。
- [ ] 支持把已有 reference 从 unresolved/旧 symbol 重新绑定到新 symbol，并同步维护旧/
      新 symbol 的 use index。
- [ ] 新 AST node 创建时立即登记 `node_scope` 和 `node_parent`；替换节点时明确是继承旧
      node 的语义身份，还是给新 node 注册新身份。
- [ ] 新增 symbol 时立即写入对应 scope binding map；rename 时原子更新 symbol name 和
      scope map key。
- [ ] 对 arena append、side-table resize、重复注册、错误 scope 和无效 ID 增加 debug
      invariant 检查。

建议的最小 mutation API：

```rust
semantic.register_node(node, parent, scope);
semantic.declare(scope, node, name, flags) -> SymbolId;
semantic.reference(scope, node, name, flags, symbol) -> ReferenceId;
semantic.rebind_reference(reference, symbol);
semantic.rename_symbol(symbol, name);
```

完成标准：新增 generator 私有变量及其所有引用只产生 arena append 和 ID/index 赋值，
不触发全树 semantic rebuild。

## Phase 5：建立 hoist-scope used-name 索引

涉及文件：

- `crates/swc_ecma_semantic/src/binder.rs`
- `crates/swc_ecma_semantic/src/lib.rs`
- `crates/swc_ecma_ast/src/hygiene.rs`

任务：

- [ ] semantic finalize 时一次性建立每个 hoist scope 的名字占用索引。
- [ ] 索引覆盖会与 hoisted 临时变量发生文本冲突的声明，以及作用域内需要保留的
      unresolved/dynamic references。
- [ ] NameAllocator 使用 `HashMap<AstStr, u64>`：key 是 canonical base name，value 是
      下一次需要尝试的 suffix。
- [ ] 分配 `_tmp` 时先查询 scope index；冲突则尝试 `_tmp1`、`_tmp2`，成功后立即登记
      exact name 并更新 base 的 next suffix。
- [ ] 定义 ancestor/descendant scope 的冲突规则；与当前 generator `scopes_overlap` 行为
      保持一致，但把计算移动到 semantic 建索引阶段，只做一次。
- [ ] 分配名字时同时创建 fresh `Mark`/`SyntaxContext`，返回文本 handle、context 和
      `SymbolId` 的组合对象。
- [ ] transform 新增或 rename binding 时同步修改 used-name 索引，保证跨 pass 可见。

建议接口：

```rust
ctx.declare_private(
    target_scope,
    base_name,
    symbol_flags,
) -> PrivateBinding { name, context, symbol };
```

完成标准：任意数量 generator function 都不再分别遍历
`semantic.iter_symbols()`/`iter_references()`；名字分配均摊 O(1)，suffix 冲突时只与实际
冲突数量相关。

## Phase 6：让 AST context 成为 AST 与 Semantic 的唯一修改入口

涉及文件：

- `crates/swc_ecma_ast/src/visit.rs`
- `crates/swc_ecma_transformer/src/state.rs`
- `crates/swc_ecma_transformer/src/lib.rs`

任务：

- [ ] 将 transformer 入口从 `&Semantic` 改成 `&mut Semantic`。
- [ ] 让 transform 使用的 AST context 同时访问 AST、semantic、enabled features、当前 node
      path 和 current scope；`TransformerCtx` 可以是该 context 的具体实现，但同步协议不
      放在各个 pass 内。
- [ ] 进入/离开 scope creator 时维护 scope stack；已有节点从 `semantic.scope_of(node)`
      初始化，新创建 scope 由 context 注册。
- [ ] 增加 semantic-aware AST helper：普通 node、binding identifier、resolved reference、
      unresolved reference、字段修改、replacement、insertion 和 removal。
- [ ] 明确禁止 transform 通过裸 `&mut Ast` builder/setter 修改树；逐步把 transform 相关
      builder/setter 迁移到 context helper。AST crate 的 parser/build 阶段仍可使用底层 API。
- [ ] 每个 mutation 先取得 old subtree summary，完成 AST 修改和 node/parent/scope 注册后计算
      new summary，再将 delta 沿祖先链传播到对应 function boundary。
- [ ] 批量 lowering 使用 mutation transaction：记录受影响的最小 roots，commit 时每个 root
      只汇总一次，避免逐 leaf 修改造成 O(edits × depth)。事务提交前不能把半更新 AST 暴露
      给其他 pass。
- [ ] 插入完整子树时递归登记 parent/scope/symbol/reference，并只计算 enabled effects；已通过
      helper 单独登记的节点不得重复注册。
- [ ] 删除/替换子树时同步移除或转移 declaration/use/capture/function-order/effect 索引；对
      暂不支持的跨 scope move 明确拒绝并要求使用 remove + reinsert 协议。
- [ ] 为 debug build 增加 AST mutation epoch/semantic epoch 校验，尽早发现绕过 context 的裸
      AST 修改；不能依赖约定静默产生 stale semantic。

完成标准：context mutation/transaction 返回时，scope、binding、reference、used names、
function/capture indexes 和所有 enabled subtree effects 都与当前 AST 一致。

## Phase 7：迁移 generator

涉及文件：

- `crates/swc_ecma_transformer/src/es2015/generator/compiler/mod.rs`
- `crates/swc_ecma_transformer/src/es2015/generator/compiler/expression.rs`
- `crates/swc_ecma_transformer/src/es2015/generator/compiler/statement.rs`
- `crates/swc_ecma_transformer/src/es2015/generator/analysis.rs`

任务：

- [ ] 删除 generator 私有 `NameAllocator`、`scan_existing`、`scopes_overlap` 和
      `observe_name`。
- [ ] `_state`、`_tmp`、`_arguments` 等全部通过 `ctx.declare_private` 分配。
- [ ] generator 生成的 binding identifier 和 identifier reference 通过返回的
      `PrivateBinding` 创建，并立即登记 declaration/reference。
- [ ] 捕获原始 `arguments` 时使用 `ReferenceId` 执行 rename/rebind，而不只修改 AST 文本和
      `SyntaxContext`。
- [ ] hoisted variables/functions 注入目标 function scope 时同步设置 parent/scope 索引。
- [ ] generator 完成后允许后续 pass 查询生成节点的 `scope_of`、`reference_of`、
      `symbol_of` 和 `uses`，不再依赖“semantic 只对应 transform 前 AST”的约定。
- [ ] 删除独立 `YieldFacts`/`YieldCollector` 和 generator transform 前的 analysis visitor；
      `contains_yield`、generator post-order 和 capture 信息全部查询 semantic。
- [ ] generator lowering 对 yield node 的删除、生成 statement/state-machine node、function
      flag 修改都通过 AST context transaction，让 `YIELD` summary 在 commit 后同步消失或
      更新。
- [ ] 如果后续 pass 需要查询 generator 生成树中的 await/yield 等 enabled effect，直接读取
      更新后的 semantic，不允许补跑 visitor。

完成标准：generator transform 的耗时随 generator 数量近似线性增长；同一 program 中前面
generator/pass 生成的名字会被后面的分配立即看到。

## Phase 8：按相同协议迁移公共 transform 工具

涉及文件：

- `crates/swc_ecma_transformer/src/es2015/util/fn_env_hoister.rs`
- 后续 block scoping、destructuring、class transform 的实现文件

任务：

- [ ] 把 `FnEnvHoister` 的 capture declaration/reference 迁移到 context API。
- [ ] 提取 transformer 公共 `PrivateBinding`/name allocation helper，generator 不保留特例。
- [ ] destructuring 创建 temp 时复用相同 allocator 和 semantic 注册流程。
- [ ] block scoping 的 rename 使用 `SymbolId` 修改 declaration + uses；生成 loop wrapper
      binding 时同步 scope。
- [ ] class transform 的 private temp、super helper capture 和 computed-key temp 使用相同
      机制。
- [ ] async transform 直接复用 semantic 的 `AWAIT` summary，不再运行 pass-local
      `contains_await` visitor。
- [ ] 每迁移一个 pass，就删除该 pass 自己的 used-name/effect/capture AST scan，并把它所需
      feature 映射加入 `TransformerOptions::semantic_features()`。

## 提交拆分

保持每个提交可独立编译，不 amend：

1. `feat(ast): add canonical pooled AST strings`
   - `AstStr`、StringPool 第三类字符串来源和 canonicalization。
2. `feat(semantic): collect transform facts on demand`
   - `SemanticFeatureFlags`、options 传递、yield/await/function/capture summary。
3. `refactor(semantic): support incremental symbols and references`
   - 可变 semantic ID/index 和 mutation API。
4. `feat(semantic): index used names by hoist scope`
   - 一次性 used-name/suffix 索引。
5. `refactor(transformer): synchronize semantic through AST context`
   - `&mut Semantic`、current scope、semantic-aware mutation transaction 和 effect delta。
6. `perf(transformer): consume semantic generator facts`
   - generator 名字分配与 yield/capture 查询迁移，删除 transform 前分析 visitor。
7. 后续每个 transform 单独提交迁移，不和 generator 性能修改混在一起。

现有 hygiene 对象的提交保持独立，不 squash 到上述提交。

## 验证

不新增 Rust test 或新 fixture，使用已有 Babel generator 端到端测试验证行为。

- [ ] 运行已有 transformer Babel generator fixtures。
- [ ] 运行已有 workspace Rust test、unit test 和 clippy；不增加专项 Rust test。
- [ ] 使用本地 `tasks/benchmark/benches/generator_transform.rs`，确保只启用 generator
      transform。
- [ ] benchmark 分别统计 parse、semantic、transform、codegen，不能把 semantic 更新成本
      隐藏到其他阶段。
- [ ] semantic benchmark 同时记录 `features=empty`、generator-only features 和未来
      async-only features，确认按需收集确实避免未启用功能的时间/内存成本。
- [ ] 覆盖 1、48、192 个 generator 的规模，比较 SWC 和 SWC Next 的增长曲线。
- [ ] 单独记录 semantic effect collection、name allocation、generator lowering、AST context
      transaction commit 和 transform 后遍历的 profile 数据。
- [ ] 优化前后使用相同输入、相同 generator-only 配置和相同输出校验。

性能验收：

- generator 数量增长时，SWC Next transform 不再出现由全量 symbol/reference scan 导致的
  O(G²) 曲线。
- 单 generator 不因 mutable semantic/context 引入明显固定开销。
- semantic 一次性 used-name 索引成本为 O(symbols + references)，并计入 semantic 阶段。
- `YIELD`/`AWAIT` summary 在 semantic 原有遍历内线性建立；feature 关闭时无 side-table
  allocation，只有 options 派生的固定成本。
- generator transform 阶段不再出现 `YieldCollector`/capture collector 的整树遍历。
- AST context 批量 transaction 的 effect 更新与受影响子树和祖先路径相关，不因全 program
  大小触发重建。
- codegen 输出与已有 Babel fixtures 一致。

## 风险和处理

- declaration/use 当前按 symbol 压缩为连续 range。先改为可增量索引，再开放 mutation
  API，避免在 transform 热路径重排全量数组。
- `with` 和 direct eval 可能让文本 rename 具备动态语义。第一阶段只允许创建带 hygiene 的
  私有 binding；对 dynamic reference 的 rebind 保守拒绝或维持 unresolved。
- AST crate 不能依赖 semantic crate。组合 `Ast + Semantic` 的 mutation API 放在
  transformer 的 AST context；AST crate 只提供字符串池、hygiene、底层 node builder 和
  mutation hook 所需的通用接口。semantic feature 类型也不能依赖 transformer options；由
  transformer 提供单向映射。
- 裸 AST builder 无法根据名字推导正确 symbol；API 必须接收明确的 `ScopeId`/`SymbolId`，
  不能在 transform 热路径重新按文本解析。
- 对已有节点的原地 replacement 必须定义 NodeId 继承规则；未定义前不得静默沿用旧
  declaration/reference 记录。
- 高频 leaf mutation 若逐次沿祖先链传播 effect delta，可能退化为 O(edits × depth)。
  generator lowering 必须使用 transaction 合并 dirty roots，再在 commit 时做最小范围汇总。
- 仅有 AST mutation hook 无法自动推断新 identifier 是 binding、resolved reference 还是普通
  property name。context API 仍必须要求调用方提供 mutation kind、scope 和可用的 SymbolId，
  不能只根据 NodeId/字符串猜测 semantic。
- options 可能在 semantic 完成后被修改。编译入口必须冻结一份 `TransformerOptions`，先由
  它派生 semantic features，再把同一份 options 交给 transformer；transform 不接受中途开启
  一个未收集 feature 的开关。

`plan.md` 作为当前方案说明随临时 benchmark 一并提交；后续实现仍按上述提交边界拆分。
