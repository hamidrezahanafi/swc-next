# Typed AST stores 与可参数化 transform insertion holes 执行计划

## 1. 目标

把当前所有 AST 节点共用一个 `NodeData`/`NodeId` 的实现改为：

- `NodeId<T>` 是 4 字节泛型 ID；每个存储域独立自增，ID 中保留存储域信息，避免拿 statement ID 访问 expression storage。
- AST 按热点类型拆成多个 SoA storage，transform 新增某一类节点时只影响对应 storage，不再扩容并复制全局 `NodeData`。`Class` 和 `Function` 作为大对象固定使用独立 storage，不参与后续合并判断。
- 删除可直接用无类型全局 ID 读取任意节点数据的 API；具体节点通过类型安全 ID 访问对应 storage。必须异构处理时，只允许先匹配 storage/kind，再收窄为 typed ID。
- 每个 storage 使用独立、编译期参数化的 hole policy；候选默认值是 `16 live slots + 4 holes`，但必须支持某个特定 storage 使用 `16+0`。transform 优先复用该 storage 的 hole，目标是避开扩容/复制热路径、降低单次插入成本。
- 保持 ESTree/JavaScript 可观察 AST 结构和 builder/getter 的逻辑语义不变。
- 改动按可单独审查的类别提交；完成通用实现后，从 `codex/es2022-static-blocks-transform` 建立独立验证分支，量化收益。

本任务不顺带改变具体 AST 字段、ESTree 节点形状或 transform 输出。`extra_data` 中列表插入时重建连续 range 的问题也不在本轮解决，但要单独统计，防止把它的成本误算成 node storage 插入成本。

hole policy 的核心取舍不是“是否节省内存”，而是：预留 hole 能否显著提升 transform 插入性能，以及 hole 造成的有效节点不连续、密度下降和后填节点乱序是否损害后续 traversal/semantic/codegen 的缓存局部性。内存容量只作为解释这些结果的辅助指标，不作为选择 policy 的首要条件。

## 2. 当前实现与影响面

当前执行流：

```text
Parser / Ast builders
  Ast::add_node(span, NodeKind, NodePayload)
       ↓
单一 NodeData
  [Span × cap][NodeKind × cap][NodePayload × cap]
       ↓ capacity 满
multi_index_vec::grow_to
  分配 2x 新 block，并复制所有已初始化 span/kind/payload
       ↓
Transform builders
  任意新增 expression/statement/class/function
  都可能触发整个 NodeData 扩容
       ↓
Visitor / Semantic / FFI
  用同一个全局 NodeId 查 kind/span/payload
```

关键文件和必须迁移的不变量：

| 范围 | 当前实现 | 迁移要求 |
| --- | --- | --- |
| node storage | `crates/swc_ecma_ast/src/lib.rs` 的 `Ast::nodes: NodeData` | 拆成 typed stores，并区分 live/hole/slot count |
| ID | `crates/swc_ecma_ast/src/node_id/node.rs` 的全局 `NodeId` | 改成 `NodeId<T>` 和只用于异构身份的 erased ID |
| allocation | `crates/swc_allocator/src/multi_index_vec.rs` | 支持按 storage 参数化的 hole policy、hole 复用、checkpoint/truncate |
| 代码生成 | `tasks/ast_tools/src/generator/*` | 从 schema 生成 store 归属、typed access、typed edge conversion |
| parser | `crates/swc_ecma_parser/src/parser.rs` | capacity 估算、speculative checkpoint、parenthesized stripping 都改为多 store |
| visitor | `tasks/ast_tools/src/generator/ast_visitor.rs`、`crates/swc_ecma_ast/src/visit.rs` | path 保留异构身份，但不能作为无类型 payload accessor |
| semantic | `crates/swc_ecma_semantic/src/*` | map/key 使用稳定 erased identity；读取节点前必须 typed dispatch |
| child ranges | `crates/swc_ecma_ast/src/node_id/sub_range.rs`、`ExtraData` | 存储 packed ID；typed range 取值时验证/恢复 `NodeId<T>` |
| replace | `Ast::set_data`、`Ast::replace_data` | 去掉“临时节点必须是全局表尾”的依赖，处理 same-store 与 cross-store replacement |
| FFI | `crates/swc_ecma_ast/src/ffi/ast.rs`、N-API decoder | serialization 时跳过 holes，把多 store 临时压平成兼容 wire layout 并重映射 edge ID |
| layout assertions | `crates/swc_ecma_ast/src/assert_layout.rs` | 保证 typed wrapper 和 `NodeId<T>` 仍是 4 字节 |

`codex/class-typed-store` 上已有 `a434ff6` 等提交，可以用作迁移影响面的参考，但不能直接作为本方案基础：该实现仍为每个 class 保留 common `NodeData` owner，`ClassId` 也复用了 common owner index，因此 transform 插入 class 时仍会增加全局 `NodeData`，没有满足“各 storage 独立自增并彻底去掉全局节点表”的目标。

## 3. SWC/Babel transform 静态统计

### 3.1 统计口径

- SWC：本地 `/data00/home/jinzhixin/rstack/swc` 的 commit `5bf27fd72e`。
- 扫描 284 个 `src/**/*.rs`，范围是 `swc_ecma_compat_*`、`swc_ecma_transforms_{base,classes,module,optimization,proposal,react,typescript}` 和 `swc_ecma_utils`。
- 先从 `swc_ecma_ast` 提取 `#[ast_node]` struct，再统计 `Node { ... }`、`Node::new(...)`、`Node::dummy(...)` 构造点。
- Babel：本地安装的 53 个 `@babel/plugin-transform-*` 的 `lib/**/*.js`；`@babel/core` 7.22.1、`@babel/types` 7.22.3。统计 `t.<builder>(...)`，排除 `is*`、`assert*`、`cloneNode`、`inherits*`、`to*` 等查询/工具函数。
- 这是“源码构造点”而不是运行时调用次数。helper 被调用的次数、fixture 的语法分布和条件分支不会反映在表中，所以它只用于初始分池；最终分池和 hole policy 必须再经过第 8 节的动态插入与读取局部性测试确认。

### 3.2 高频构造点

| 排名 | SWC 节点 | 构造点 | Babel builder | 构造点 |
| ---: | --- | ---: | --- | ---: |
| 1 | `CallExpr` | 91 | `identifier` | 144 |
| 2 | `AssignExpr` | 68 | `callExpression` | 56 |
| 3 | `VarDeclarator` | 68 | `memberExpression` | 56 |
| 4 | `VarDecl` | 50 | `expressionStatement` | 49 |
| 5 | `ReturnStmt` | 36 | `stringLiteral` | 49 |
| 6 | `BlockStmt` | 35 | `assignmentExpression` | 42 |
| 7 | `Ident` | 33 | `variableDeclarator` | 40 |
| 8 | `ArrayLit` | 31 | `variableDeclaration` | 36 |
| 9 | `BinExpr` | 30 | `objectExpression` | 19 |
| 10 | `MemberExpr` | 27 | `numericLiteral` | 18 |
| 11 | `ThisExpr` | 23 | `arrayExpression` | 15 |
| 12 | `ExprStmt` | 22 | `binaryExpression` | 15 |
| 13 | `Str` | 20 | `sequenceExpression` | 15 |
| 14 | `SeqExpr` | 18 | `blockStatement` | 12 |
| 15 | `ObjectLit` | 16 | `objectProperty` | 11 |

Babel 过滤后的 671 个 builder 构造点中，expression/literal/property/name 一组约 498 个（74.2%），statement/declaration 一组约 149 个（22.2%）。SWC 的具体排名也显示相同分布：expression 是最大热点，variable declaration、block、expression statement、return 是第二组热点。

`codex/es2022-static-blocks-transform` 的目标 transform 还会直接构造 call、sequence、assignment、member、identifier、string、arrow/function body、return/expression statement、variable declaration/declarator 和 fallback class property。因此 class/function 在全库静态计数不一定最高，但在目标验证场景中必须独立观察。

### 3.3 建议的 storage 分类

先实现以下互斥 storage；归属由 AST schema 声明并由 `ast_tools` 生成，禁止在 accessor 中手写重复表：

| Storage | 主要节点 | 原因 |
| --- | --- | --- |
| `ExpressionStore` | call/member/assignment/binary/sequence/unary/conditional、arrow、array/object、literal、this/super 等 expression-only 节点 | SWC/Babel 最大 transform 构造热点 |
| `StatementStore` | statements、declarations、module declarations、variable declarator、switch case/catch 等 statement 辅助节点 | 第二大热点；static-block 会插入 expression statement、return、var declaration |
| `NamePatternStore` | identifier variants、private name、binding/assignment patterns、property-key leaf | identifier 是 Babel 第一、SWC 前列热点；独立后 expression 插入不会复制所有 binding/name 节点 |
| `FunctionStore` | concrete `Function` record | 固定独立的大对象池；直接保存 function 的完整 typed record/SoA 字段，不能因为静态构造点低于 expression 就并回通用池。arrow 仍属于 expression pool，function body/params 属于辅助池，避免小节点扩容大对象池 |
| `ClassStore` | concrete `Class` record | 固定独立的大对象池；直接保存 class 的完整 typed record/SoA 字段，同时是 static-block 目标路径，必须避免 common owner 间接层 |
| `ClassElementStore` | class body、method/property definition、static block、decorator 等 class 辅助节点 | 与大 `Class` record 分开；static-block 新增 fallback property 时不能迫使整个 `ClassStore` 扩容 |
| `TypeJsxStore` | TypeScript type/declaration 辅助节点与 JSX 专用节点 | transform 多数是删除/改写，构造频率低，先共用冷池，避免过度拆分 |
| `ProgramAuxStore` | Program、function body/params、directive、import/export specifier 等未归入以上类别的低频节点 | 保持分类穷尽，并给未来动态数据留下再拆分入口 |

`ExpressionStore`、`StatementStore`、`FunctionStore`、`ClassStore` 是本轮固定的独立 storage。动态统计只用于决定 `NamePatternStore` 是否继续独立，以及冷门节点如何归入 `TypeJsxStore`/`ProgramAuxStore`；不能把 class/function 合回通用池。不要为每个 concrete `NodeKind` 建一个 allocation；过多小池会增加 dispatch，并使有效节点被更多边界和 holes 打断，损害顺序读取局部性。

`ClassStore` 和 `FunctionStore` 也不应只是“全局 `NodeData` owner + 另一个大对象数组”的双层间接实现。owner 的 `span/kind` 和对象字段应一起归属于各自 typed storage，ID 直接索引该 storage；否则新增 class/function 仍会扩大全局 owner 表。实现时为二者生成专用完整 record/SoA 列，热字段与可选冷字段是否再拆列由 layout benchmark 决定，但不能保留 common owner slot。class element、arrow、body、params 等较小节点不能混入这两个大对象池，它们通过 typed ID/range 被大对象引用。

## 4. ID 与类型安全访问设计

### 4.1 ID 表示

- `NodeId<T>` 保持 `#[repr(transparent)]`、4 字节、`Copy/Eq/Hash`；`T` 表示允许访问的 AST 类型。
- raw `u32` 使用高 4 bit 保存 `StoreTag`，低 28 bit 保存 store-local slot index。最多 16 个 storage，每池最多 `2^28 - 1` 个物理 slot；为 sentinel/overflow 增加显式检查。
- 每个 storage 的 local index 独立增长。扩容只移动本 storage 的 span/kind/payload。
- `ErasedNodeId` 同样是 packed 4 字节，仅供 visitor path、semantic key、module record、root-independent identity 使用；它不提供 payload getter。
- `NodeId<Expr>` 等 union handle 可指向多个允许的 storage，例如 class expression 可位于 `ClassStore`。从 concrete ID 到 union ID 的转换由 schema 生成；反向转换必须检查 store tag 和 concrete kind。
- `AstNode` 返回 `NodeId<Self>` 或对应 typed union ID；删除依赖 `transmute<NodeId, Wrapper>` 且默认所有节点共用一个 index domain 的实现。

### 4.2 访问规则

目标 API：

```text
typed wrapper / NodeId<T>
       ↓ 检查编译期允许的 store 集合
StoreTag + local slot
       ↓
Ast::<specific store>.span/kind/payload(local id)
       ↓
生成的 concrete/union data accessor
```

- concrete getter 直接访问已知 storage。
- union getter 只对该 union 允许的 storage/kind dispatch。
- 全局 `Ast::kind_data(NodeId)`、`Ast::node_kind(NodeId)` 不能再作为任意 payload 入口。内部确实需要异构 dispatch 的位置（visitor、semantic checker、FFI）使用 `match erased.store_tag()`，再调用生成的 checked narrowing API。
- `span` 可以保留泛型便利方法 `span<T>(NodeId<T>)`，但内部也必须按 tag 路由，不能恢复一个全局数组。
- `TypedSubRange<T>`/`ExtraData` 存 packed ID，取元素时恢复 `NodeId<T>` 并在 debug/test build 验证 store/kind。

## 5. 可参数化 hole 布局

每个 storage 的物理 slot 按自己的 `HolePolicy` 组织。生产候选从 `live_per_chunk = 16`、`holes_per_chunk = 4` 开始，但这两个值不能散落成 magic numbers：

```text
HolePolicy {
  live_per_chunk,
  holes_per_chunk,
}

chunk 0: [live × live_per_chunk][hole × holes_per_chunk]
chunk 1: [live × live_per_chunk][hole × holes_per_chunk]
...
```

- policy 必须按 storage 独立配置，例如 `ExpressionStore = 16+4`、`StatementStore = 16+4`、`ClassStore = 16+0`、`FunctionStore = 16+0`。这只是待验证组合，不是预设结论。
- 优先使用 const generic 或 schema 生成的 associated constants，使 `holes_per_chunk = 0` 时 hole 分支可在编译期消除。不要为 benchmark 引入每次 allocation 都读取的运行时配置或动态分支。
- benchmark 可通过专用 Cargo feature、生成配置或 benchmark-only const profile 编译不同 policy；生产 API 不暴露运行时调节开关。
- `live_per_chunk` 默认保持 16，以便单独测量 hole 数量的影响；验证完 `0/1/2/4/8` 后，才考虑是否同时调整 live chunk 大小。
- 对 policy 做编译期约束：`live_per_chunk > 0`、chunk slot 数不会溢出 index/capacity，且 0-hole specialization 与普通路径产生相同的 live ID/AST 行为。
- `NodeKind` 增加内部 `Hole`；hole 的 span/payload 写入确定的零值，不能留下未初始化字节供 FFI 或 debug tooling 读取。
- storage 分别维护 `slot_len`、`live_len`、hole bitmap/free list、capacity、grow count、copied bytes，以及插入时的 hole lookup/consume 次数。
- parser 顺序构建时，每写满 `live_per_chunk` 个 source nodes 就物化 `holes_per_chunk` 个 holes；后续 source node 从下一个 chunk 开始。`holes_per_chunk = 0` 时不创建 `Hole` slot/free-list entry。
- transform builder 优先消费对应 storage 的 hole；没有 hole 才追加新 chunk/扩容。物理顺序不再代表 source/creation 顺序，AST 顺序只由 edge/range 决定。
- hole 会让 parser 产生的有效节点在物理数组中每个 chunk 被打断；transform 后填 hole 还会把新节点插入旧节点之间。实现和 benchmark 必须显式测量这种不连续带来的 cache miss、分支和顺序扫描成本，不能只看扩容次数。
- `node_count()` 返回 live count；新增 `slot_count()`/`hole_count()` 仅用于内部测试和 benchmark。
- 当前 parser 的 `baseline + 50%` 全局 reserve 不能与 hole 开销叠加。capacity estimate 要改成按各 storage policy 计算：`ceil(live / live_per_chunk) * (live_per_chunk + holes_per_chunk)`，并保留小文件下限。对 `16+0` 不增加 hole capacity。
- speculative parser checkpoint 保存所有 store 的 `slot_len/live_len/free-list checkpoint` 及 `extra_data` 长度；restore 必须精确撤销已分配节点和 hole 消耗。
- `replace_data`：
  - same-store replacement 可把新 payload 搬到旧 slot，并把临时新 slot 归还为 hole；不再要求新节点位于全局表尾。
  - cross-store replacement 不伪造旧 typed ID，必须更新 parent edge/root 为新 typed ID，并将旧 slot 标成 hole。
  - semantic-bearing node identity 是否需要保留由现有 invalidation 规则决定；禁止通过跨 store 复制 payload 保留一个类型错误的 ID。
- live AST 不做自动 compact/reindex，保证所有已发出的 ID 稳定。只允许在 FFI serialization 时建立临时 dense mapping。

需要覆盖的边界：

- `16+0`：0、1、15、16、17、31、32 个 live nodes，确认无 `Hole`、无 free-list 分配。
- `16+4`：0、1、15、16、17、20、21、31、32 个 live nodes；连续消费 4 个 holes 及第 5 次 transform 插入。
- 至少再覆盖一个非 4 配置（如 `16+2`），防止实现暗含固定 4。
- 所有配置覆盖 checkpoint 跨 chunk 回滚、same-store/cross-store replace、pool index 与 packed ID overflow。

## 6. API 兼容性结论

“AST API 没有结构变化”只对 ESTree/JavaScript 输出成立，对 Rust 内部 API 不成立：

- 不变：具体 AST 节点名、字段、builder 的逻辑参数、getter/setter 的语义、ESTree JSON/N-API 输出结构。
- 会变：公开的 `NodeId` 类型、`AstNode::node_id` 签名、`TraverseCtx::path/current_node/parent/ancestor` 的 ID 类型、semantic/module-record 中保存的 node identity，以及依赖 `node_kind/kind_data` 的调用点。
- wire format 可通过 serializer 临时压平保持兼容，但 serializer 自身需要重写并会有 remap 成本；必须纳入 benchmark。
- 如果 crate 需要维持源码兼容，可临时提供 deprecated `type AnyNodeId = ErasedNodeId` 和只做 identity 的转换层；不能提供能重新访问任意 payload 的旧 `get node` 兼容层。

因此验收描述应是“无 AST schema/ESTree breaking change；有 Rust typed-ID API migration”，不能标成完全无结构变化。

## 7. 分阶段实施

### Phase 0：建立基线与动态统计（已完成）

- [x] 通用实现从 `origin/main` 的 `a5fa7d1a` 开始；目标 transform 分支为 `11543d5b`。验证开始时的 `origin/main` 为 `40009f2f`。
- [x] 为 `Ast`/各候选 storage 增加 benchmark/debug counters：实际 hole policy、live、slots、holes created/used、hole lookup、grow count、grow copied bytes。
- [x] 增加 class-heavy fixture，并覆盖 static block 位于字段前后、多个 block、class declaration/expression 与 nested class。
- [x] 结合 SWC/Babel 静态构造统计及实际 benchmark metrics 固定 node-kind → storage 映射；`NamePatternStore` 保持独立，`ClassStore`/`FunctionStore` 无条件独立。

### Phase 1：typed ID 基础设施（已完成）

- [x] 在 `node_id/node.rs` 实现 `NodeId<T>`、4 字节 packed `AstId`、`StoreTag`、checked narrowing、sentinel/overflow。
- [x] 增加 layout assertion，确保 concrete wrapper、union wrapper、typed ID、erased ID 均为 4 字节。
- [x] 先引入可独立编译的 typed storage domains，再按类别拆分实际 storage。

### Phase 2：schema 与生成器（已完成）

- [x] 在 AST schema 中为每个 concrete node 标注唯一 storage；生成映射和穷尽性测试。
- [x] 修改 `ast_builder`：builder 选择目标 storage 并返回 typed wrapper。
- [x] 修改 `ast_property`：getter/setter 按 packed tag 索引所属 storage。
- [x] 修改 `ast_node`：生成 concrete/union typed conversions，不再假设单一 index domain。
- [x] 修改 `ast_extra_compact`、clone、visitor generator，确保所有 node edge 都使用 packed typed ID。
- [x] 执行 `cargo run -p ast_tools` 并提交 regenerated files。

### Phase 3：多 storage parser/build 路径（已完成）

- [x] `Ast` 持有八个独立 storage；class/function ID 直接索引独立记录，不创建 common owner。
- [x] parser scratch buffer 使用 packed identity，并在恢复具体语法类型时收窄。
- [x] checkpoint/truncate 保存并恢复所有 storage。
- [x] `strip_parenthesized_nodes` 分 store 扫描并跳过 holes。
- [x] 全局 `baseline + 50%` reserve 改为 per-store estimate。

### Phase 4：访问方迁移并删除 global get-node（已完成）

- [x] 迁移 `swc_ecma_ast` 的 root、property、sub-range、clone、visitor。
- [x] 迁移 parser 的节点访问与 speculative parse 路径。
- [x] semantic side tables 改为 per-store，避免不同 local index 冲突；异构 key 仍使用 packed identity。
- [x] 迁移 transformer 的 `TraverseCtx`、normalize、arrow 等异构 ancestor 判断，并在 transform 开始时启用 hole reuse。
- [x] codegen 和 testsuite runner 通过 workspace/conformance 验证。
- [x] 删除可由 erased/global ID 直接取得裸 `NodeData` 的入口；保留 `node_kind`/`kind_data` 作为 checked heterogeneous dispatch，不把它们暴露为 raw payload getter。

### Phase 5：参数化 holes 与 replacement（已完成）

- [x] 实现 const-generic per-store hole policy、`Hole`、hole reuse 和 counters；支持 `16+0`、`16+2`、`16+4`，0-hole 分支可编译期消除。
- [x] policy 集中定义，并由 benchmark 打印实际 profile；提供 all-zero、hot-2、expression-only、statement-only profiles。
- [x] builder 区分 parser source allocation 与 transform insertion；transform 优先 holes。
- [x] benchmark 分离 insertion-only 与 post-transform read-only visit，并提供 Criterion 与 Gungraun 入口。
- [x] same-store/cross-store replacement 支持独立 class/function store，并检查 double free/hole read。
- [x] 单测覆盖 0/1/15/16/17/31/32、`16+2` checkpoint、4 holes 消耗及第 5 次插入；workspace/conformance 覆盖集成路径。

### Phase 6：FFI/N-API 兼容（已完成）

- [x] serializer 构造 packed ID → dense wire ID 映射，输出 live nodes 并跳过 holes。
- [x] 重写 payload/extra-data/node-range/root 中的 ID；共享 extra slot 只映射一次。
- [x] wire schema 与对外节点形状不变；N-API 测试覆盖跨多个 hole chunk 的声明，并增加共享 extra-data 回归测试。
- [x] class-heavy metrics 单独输出 FFI buffer size；validation 分支继续记录 serialize instructions。

### Phase 7：正确性验证（已完成）

- [x] `cargo fmt --all`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace --locked`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `pnpm test`
- [x] parser/codegen/resolver conformance 与 Babel transform runner 已运行；snapshots 无 typed-store 引入的变化。Babel runner 保留基线已有的 42 个 output diff 与 7 个其他失败。
- [x] ESTree conformance 全量运行完成，无 panic，snapshots 无变化。

## 8. 插入性能与读取局部性的取舍验证

### 8.1 分支流程

1. 通用实现分支：从最新 `origin/main` 创建 `codex/typed-node-stores`，按第 9 节提交。当前 worktree 是 detached `origin/main`，实际实施前先显式建分支。
2. 通用实现和正确性验证全部完成后，从 `codex/es2022-static-blocks-transform` 创建 `codex/es2022-static-blocks-typed-node-stores`。
3. 为避免目标分支比 `origin/main` 少 19 个提交导致对比混入无关 main 差异，先在验证分支合并同一个 `origin/main`，把这个 merge commit 记录为 control baseline；不 rebase。
4. 按原顺序 cherry-pick typed-store 分类提交。出现冲突立即停止并询问，不继续自动解决或改写历史。
5. 比较 control baseline 与实验 tip；两边包含完全相同的 main 和 static-block transform，唯一变量是 typed stores/holes。

### 8.2 benchmark 组合

- parser-only control：现有 TypeScript、cal.com TSX、class-heavy fixture，用于观察预留 holes 对初始构建和有效节点连续性的影响。
- function-heavy control：大量 declaration/expression/arrow、嵌套 function、默认参数和 async/generator 组合，单独观测 `FunctionStore` 插入和读取局部性。
- insertion-only：每次迭代使用相同 parsed input 的 fresh clone/arena，只包围 static-block transform 中创建/插入新节点的阶段，报告每个 inserted node 的 wall time、instructions、hole lookup、grow 次数。
- post-transform read-only：预先生成完全相同的 transformed AST，对它重复运行 visit、semantic read/check 和 codegen，重点比较有效节点连续与带 holes 布局的 cache/branch 行为；计时中不得再做 transform。
- parse + semantic + static-block transform + codegen 端到端，用于确认 insertion gain 是否能抵消后续读取局部性损失。
- static-block cases 按 class 数量和每类新增节点数量扩展，至少覆盖“holes 足够”和“holes 耗尽后扩容”两侧。
- hole policy 消融矩阵至少包含：
  - 所有 storage `16+0`。
  - 仅 `ExpressionStore` 使用 `16+4`。
  - 仅 `StatementStore` 使用 `16+4`。
  - 仅 `ClassStore` 使用 `16+4`。
  - 仅 `FunctionStore` 使用 `16+4`。
  - expression + statement 使用 `16+4`，class + function 使用 `16+0`。
  - 所有 storage `16+4`。
- 对插入阶段显示收益的 storage 再跑 `holes_per_chunk = 1/2/4/8`，画出“插入 latency/instructions”与“post-transform cache misses/read wall time”的取舍曲线；插入收益不能抵消读取局部性损失的特定类型在生产配置中明确设为 0。
- 一般 transform control：arrow transform，防止只对 static-block 特化。
- FFI serialize control：同一 transformed AST 的 buffer size、instructions、临时 bytes。

使用现有 Criterion wall-time benchmark 和 Gungraun/Callgrind；Gungraun 至少记录：

- insertion phase 的 `Ir`/instructions、wall time、每 inserted node 成本
- post-transform read-only phase 的 cache reads/misses、branch mispredicts、wall time
- 每 store grow count 与 copied bytes
- holes created、looked up、used、remaining
- 有效节点密度、live slots 之间的物理 gap、顺序扫描跳过的 hole 数
- allocator total/retained bytes、physical slot bytes，仅作为解释指标

每个 wall-time case 至少独立运行 10 次并报告 median、离散度和相对变化；Callgrind 以 baseline comparison 输出。每条结果必须记录完整 per-store policy，不能只写“with holes”。插入阶段和 post-transform 读取阶段必须分开报告；只报告 allocator bytes、扩容次数或端到端总时间都不足以判断 hole policy。

### 8.3 验收条件

- 所有正确性测试通过；ESTree、transform output 和 diagnostics 无非预期变化。
- static-block 的 transform-heavy case 中，global full-table copy 被消除；对启用 `16+4` 的目标 storage，在该 chunk 的 4 个 hole 用尽前不发生 grow。使用其他 policy 时按其 `holes_per_chunk` 验证，并确认 hole lookup 自身没有抵消插入收益。
- 主要结果必须报告 insertion-only latency/instructions 和 post-transform cache/read latency；allocator retained bytes、grow copied bytes 只用于解释原因，不能用 theoretical capacity 或内存增减直接宣称收益。
- parser-only、semantic、visit、codegen、FFI 如有回退必须列出。任一核心只读场景稳定回退超过 2% 时，先调查有效节点不连续、store dispatch、hole 扫描或 serializer remap，再决定该 storage 是否应启用 holes。
- 某个 storage 的生产 policy 是否为非零，取决于插入收益与后续读取局部性损失的综合结果；若插入收益很小，或被 cache miss/read wall-time 回退抵消，则设为 `16+0`。不因内存增长本身否决 holes，也不因内存下降本身接受 holes。

最终产出一张二维取舍表：横向是各 policy 的 transform insertion latency/instructions，纵向是 transformed AST 的 visit/semantic/codegen cache misses 与 wall time；内存指标放在附属列。明确结论是收益来自“分池”、来自哪些特定 storage 的 holes，以及插入收益是否抵消了不连续代价。至少保留 control、typed stores/all `16+0`、逐 storage hole 消融、最终 per-store policy 四组可复现结果；不同 policy 可以由同一 revision 的 benchmark compile profiles 产生，不要求为每个组合创建生产提交。

结果表以性能取舍为主，使用以下结构：

| Per-store policy | Insert ns/node | Insert Ir/node | Hole lookup | Post-visit L1/LL misses | Visit time | Semantic time | Codegen time | End-to-end | 辅助内存指标 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| all `16+0` |  |  |  |  |  |  |  |  |  |
| expression `16+4` only |  |  |  |  |  |  |  |  |  |
| statement `16+4` only |  |  |  |  |  |  |  |  |  |
| final policy |  |  |  |  |  |  |  |  |  |

最终结论先描述 insertion 改善，再描述有效节点不连续造成的读取代价；不得以“内存增长/下降”作为主结论。

### 8.4 通用 synthetic 基线结果

以下结果用于验证 benchmark 能否捕捉取舍，不替代第 8.1 节 static-block 分支上的最终选型。Criterion 使用 1 秒 warm-up、2 秒 measurement、10 samples；表中为估计区间中值。

| Case | all `16+0` | hot stores `16+4` | `16+4` 相对变化 |
| --- | ---: | ---: | ---: |
| 已有 4 holes 的边界插入 | 1.782 µs | 0.260 µs | -85.4% |
| detached insert 64 | 1.832 µs | 3.273 µs | +78.7% |
| program-body insert 64 | 2.362 µs | 3.439 µs | +45.6% |
| post-insert visit 64 | 1.934 µs | 2.058 µs | +6.4%（10 samples 噪声较大） |
| detached insert 512 | 14.777 µs | 25.363 µs | +71.6% |
| program-body insert 512 | 18.538 µs | 26.794 µs | +44.5% |
| post-insert visit 512 | 11.492 µs | 11.315 µs | -1.5%（区间重叠） |

Gungraun 对“已有 4 holes”边界插入测得：`16+4` 为 1,322 instructions，`16+0` 为 3,303 instructions；启用 holes 减少约 60.0% instructions。512 节点 post-insert visit 的 instructions 完全相同（115,395）；`16+4` 的 D1/LL read misses 为 301/21，`16+0` 为 323/22，而 branch mispredicts 为 26 vs 24。该 synthetic case 中没有观察到稳定的只读局部性回退，但这不代表 parser/static-block 的真实物理间隔没有代价。

64 节点 metrics 解释了持续插入回退：all-zero 的 statement/name store 各 grow 一次、各复制 1,275 bytes；`16+4` 虽先消耗每个热 store 的 16 个 holes，之后持续插入仍各 grow 一次，并因物理 slots 更多各复制 1,675 bytes，同时每个 store 做 64 次 hole lookup。也就是说，hole 对“短 burst 且预留量足够”收益很大，对“远超预留量的持续 append”会增加 lookup 和不连续成本。最终 policy 必须以 static-block 的实际 burst 分布决定，不能全局固定为 `16+4`。

## 9. 分类提交

每个提交都应可编译，并运行与范围相称的测试；不 amend、不 rebase：

1. `bench(ast): record transform insertion and locality pressure`
   - insertion/read-only counters、静态/动态统计脚本或 benchmark helper、fixtures；不改变生产布局。
2. `refactor(ast): introduce generic typed node ids`
   - `NodeId<T>`、erased identity、tag/layout tests；暂用兼容 backend。
3. `refactor(ast): generate typed node store access`
   - schema storage mapping 与 ast_tools 生成器；包含 regenerated files。
4. `refactor(parser): allocate ast nodes in typed stores`
   - `Ast` 多 storage、parser/build/checkpoint/capacity migration。
5. `refactor(ast): remove global node data access`
   - visitor、semantic、transformer、codegen 的 typed dispatch；删除 global get-node API。
6. `feat(ast): parameterize insertion holes by node store`
   - per-store hole policy、0-hole specialization、hole reuse、replacement 与边界测试；保持可独立回退。
7. `refactor(napi): flatten typed node stores for ast transfer`
   - FFI remap、decoder compatibility、round-trip tests。
8. `test(ast): cover typed stores across conformance paths`
   - 只放跨 crate 回归和 snapshots；生产修复回到对应类别提交，不把逻辑混进测试提交。
9. `bench(transformer): compare insertion speed and ast locality`
   - 验证分支的 insertion-only、post-transform read-only benchmark 与取舍结果；不混入新的生产优化。

## 10. 完成定义

- [x] schema 中每个 concrete AST node 恰好属于一个 storage。
- [x] `Class`/`Function` 直接存于各自独立 typed storage，没有 global/common owner slot，也不会随 expression/statement pool 一起扩容。
- [x] 任意节点 ID 仍为 4 字节，store-local ID 独立自增且 overflow 有检查。
- [x] 仓库中不存在可用 erased/global ID 直接读取任意 raw payload 的 API；异构路径只保留 checked dispatch。
- [x] 每个 storage 使用集中定义的 hole policy；`16+0` 不创建 holes，非零配置按参数创建并优先供 transform 消费。
- [x] parser rollback、replace、visitor path、semantic identity、FFI remap 都有测试。
- [x] Rust/JS/conformance/clippy 全部通过。
- [x] 分类提交完成并可分别回退 hole policy。
- [ ] `codex/es2022-static-blocks-typed-node-stores` 上完成 control、all-zero、逐 storage hole 消融和最终 per-store policy 对比，分别记录插入性能和 post-transform 读取局部性，说明哪些类型应为 0；内存只作为辅助列。
