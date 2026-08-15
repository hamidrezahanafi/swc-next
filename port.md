# SWC Transform Porting Plan

## 背景

swc-next 所有 AST 修改都要求通过 AST context（`TraverseCtx` + `&mut Semantic`），
这意味着可以在 AST context 中捕获所有修改并同步到 semantic。当前分支已经实现了
generator transform 作为验证，现在需要把 SWC 的所有 ECMAScript transform 都搬过来。

## 架构模式

### 三种 transform 模式

swc-next 的 transform 通过 `Traverse` trait 实现，有三种模式：

1. **Program-level transform**（如 generator、arrow）：在 `enter_program` 中调用，
   一次性处理整个 program。`needs_node_traversal()` 返回 `false`，`TraverseAction::Skip`。
   适用于需要全局分析或批量 lowering 的 transform。

2. **Node visitor transform**（如 shorthand、sticky_regex）：在具体的 `enter_*`/`exit_*`
   hook 中处理单个节点。适用于局部替换，不需要全局状态。

3. **混合模式**：主体在 `enter_program` 中处理，但部分节点使用 visitor hook
   （如 es2015 中 shorthand 和 sticky_regex 走 visitor，generator 和 arrow 走 program-level）。

### 每个 transform 需要的组件

- `#[derive(Default)] pub struct XxxOptions` — 是否启用、配置参数
- `pub fn hook(options: XxxOptions) -> impl VisitMutHook` — 构造 hook 链（SWC 旧模式）
  或直接在 `Traverse` impl 中组合（swc-next 模式）
- 对应的 `SemanticFeatureFlags` 声明（如需 semantic 预收集信息）
- 对应的 Babel fixtures 测试入口

### Semantic 依赖

每个 transform 如果需要 semantic 预分析，需要在 `TransformerOptions::semantic_features()`
中声明对应的 `SemanticFeatureFlags`。当前已定义的 flags：

| Flag | 用途 |
|------|------|
| `YIELD` | 子树是否包含 `yield` |
| `AWAIT` | 子树是否包含 `await` |
| `GENERATOR_FUNCTIONS` | 后序遍历所有 generator function |
| `ASYNC_FUNCTIONS` | 后序遍历所有 async function |
| `FUNCTION_ENV_CAPTURE` | 函数环境捕获（`arguments`、`new.target`） |

## Transform 清单

### 1. ES2015 Runtime Transforms（轻量，已在 SWC 新 hook 链中）

这些 transform 不需要 semantic 预分析，直接在 visitor hook 中处理。

| Transform | 优先级 | 文件 | 状态 | 说明 |
|-----------|--------|------|------|------|
| shorthand_properties | P0 | `es2015/shorthand.rs` | **已实现** | `{a}` → `{a: a}` |
| sticky_regex | P0 | `es2015/sticky_regex.rs` | **已实现** | `/foo/y` → `new RegExp("foo", "y")` |
| function_name | P0 | `es2015/function_name.rs` | **已实现** | `var f = function() {}` → `var f = function f() {}` |
| duplicate_keys | P0 | `es2015/duplicate_keys.rs` | **已实现** | `{a: 1, a: 2}` → `{a: 1, ["a"]: 2}` |
| instanceof | P0 | `es2015/instanceof.rs` | **已实现** | `a instanceof B` → `_instanceof(a, B)` |
| typeof_symbol | P0 | `es2015/typeof_symbol.rs` | **已实现** | `typeof x` → `_typeof(x)` |
| spread | P0 | `es2015/spread.rs` | **已实现** | `f(...args)` → `f.apply(void 0, args)` |
| for_of | P0 | `es2015/for_of.rs` | **已实现** | `for (var x of arr)` → iterator loop |
| object_super | P1 | `es2015/object_super.rs` | **已实现** | `super.prop` in object literals |

### 2. ES2015 Heavy Transforms（需要语义分析/需要设计文档）

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| generator | P0 | 复杂 | `docs/design/generator-transform.md` | **已实现** | `function*` → regenerator state machine |
| arrow_functions | P0 | 中等 | — | **已实现** | `() => {}` → `function() {}`，需 `FUNCTION_ENV_CAPTURE` |
| template_literal | P0 | 简单 | — | **已实现** | `` `hello ${name}` `` → `"hello ".concat(name)` |
| spread | P0 | 中等 | — | **已实现** | `f(...args)` → `f.apply(void 0, args)` |
| block_scoped_functions | P0 | 简单 | — | **已实现** | 块级函数声明 → `let` 函数表达式 |
| computed_props | P0 | 中等 | — | **已实现** | `{ [key]: val }` → `_define_property({}, key, val)`（单属性） |
| for_of | P0 | 中等 | — | **已实现** | `for (var x of arr)` → iterator loop |
| new_target | P1 | 中等 | — | **已实现** | `new.target` → `this.constructor`/`void 0`/`instanceof` |
| object_super | P1 | 中等 | `docs/design/object-super.md` | **Stub（待实现）** | `super.prop` in object literals |
| block_scoping | P1 | 复杂 | `docs/design/block-scoping.md` | 待实现 | `let`/`const` → `var`，循环闭包捕获 |
| classes | P1 | 复杂 | `docs/design/class-transform.md` | 待实现 | `class Foo {}` → `function Foo() {}` |
| destructuring | P1 | 复杂 | `docs/design/destructuring.md` | 待实现 | `var {a, b} = obj` → `var a = obj.a, b = obj.b` |
| parameters | P1 | 复杂 | `docs/design/parameters.md` | 待实现 | 默认参数、rest 参数、解构参数 |
| regenerator | P0 | 复杂 | 复用 generator 设计 | 已合并到 generator | 已被 generator 替代 |

### 3. ES2016

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| exponentiation | P0 | 简单 | — | **已实现** | `a ** b` → `Math.pow(a, b)` |

### 4. ES2017

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| async_to_generator | P0 | 复杂 | `docs/design/async-to-generator.md` | **已实现** | `async function` → generator wrapper，需 `AWAIT` + `ASYNC_FUNCTIONS` |

### 5. ES2018

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| object_rest_spread | P1 | 中等 | — | 待实现 | `var {a, ...rest} = obj` / `var z = {...a, ...b}` |

### 6. ES2019

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| optional_catch_binding | P0 | 简单 | — | **已实现** | `try {} catch {}` → `try {} catch (_unused) {}` |

### 7. ES2020

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| nullish_coalescing | P0 | 简单 | — | **已实现** | `a ?? b` → `a !== null && a !== void 0 ? a : b` |
| optional_chaining | P0 | 中等 | `docs/design/optional-chaining.md` | **已实现** | `a?.b?.()` → `var _a; (_a = a) == null ? void 0 : _a.b()` |
| export_namespace_from | P1 | 简单 | — | **已实现** | `export * as ns from "mod"` |

### 8. ES2021

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| logical_assignments | P0 | 简单 | — | **已实现** | `a ||= b` → `a || (a = b)` |

### 9. ES2022

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| class_properties | P1 | 复杂 | `docs/design/class-properties.md` | 待实现 | class fields、private fields、static fields |
| private_in_object | P1 | 简单 | — | **已实现** | `#x in obj` → `_checkInRHS(obj)` |
| static_blocks | P1 | 中等 | — | 待实现 | `class A { static {} }` → IIFE |

### 10. ES3

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| member_expr_lits | P0 | 简单 | — | **已实现** | `obj.const` → `obj["const"]` |
| property_literals | P0 | 简单 | — | **已实现** | `{ "bar": 1 }` → `{ bar: 1 }` |
| reserved_words | P0 | 简单 | — | **已实现** | `var abstract = 1` → `var _abstract = 1` |

### 11. Bugfixes

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| async_arrows_in_class | P1 | 简单 | — | **已实现** | class property 中的 async arrow |
| edge_default_param | P1 | 简单 | — | **已实现** | Edge 默认参数处理 |
| safari_id_destructuring | P1 | 中等 | — | **Stub（待实现）** | Safari 解构冲突修复 |
| template_literal_caching | P1 | 中等 | — | **已实现** | 模板字面量缓存优化 |

### 12. TypeScript

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| ts_strip | P1 | 复杂 | `docs/design/typescript-strip.md` | 待实现 | 删除 TypeScript 类型注解、enum、namespace |
| ts_enum | P1 | 中等 | — | 待实现 | TypeScript enum → JavaScript |

### 13. React

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| jsx | P1 | 复杂 | `docs/design/jsx-transform.md` | 待实现 | JSX → `jsx()`/`jsxs()`/`jsxDEV()` |
| jsx_self | P1 | 简单 | — | 待实现 | `__self` 属性 |
| jsx_src | P1 | 简单 | — | 待实现 | `__source` 属性 |
| display_name | P1 | 简单 | — | 待实现 | `Component.displayName` |
| pure_annotations | P1 | 简单 | — | 待实现 | `/*#__PURE__*/` 注解 |
| refresh | P2 | 中等 | — | 待实现 | React Fast Refresh |

### 14. Module

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| common_js | P2 | 复杂 | `docs/design/module-transforms.md` | 待实现 | `import/export` → `require()`/`exports` |
| amd | P2 | 复杂 | 同上 | 待实现 | `import/export` → AMD `define()` |
| umd | P2 | 复杂 | 同上 | 待实现 | `import/export` → UMD wrapper |
| system_js | P2 | 复杂 | 同上 | 待实现 | `import/export` → SystemJS |

### 15. Proposal

| Transform | 优先级 | 复杂度 | 设计文档 | 状态 | 说明 |
|-----------|--------|--------|----------|------|------|
| decorators | P2 | 复杂 | `docs/design/decorators.md` | 待实现 | 2022-03 / 2023-11 decorators |
| export_default_from | P2 | 简单 | — | 待实现 | `export default from "mod"` |
| import_attributes | P2 | 简单 | — | 待实现 | `import json from "file.json" with { type: "json" }` |
| explicit_resource_management | P2 | 中等 | — | 待实现 | `using` / `await using` |

## 执行策略

### 阶段划分

#### Phase 1：基础设施和轻量 transforms（P0，简单）
聚焦于不需要复杂语义分析的 visitor-style transforms，快速建立完整的 transform 框架。

1. **ES3 transforms**（member_expr_lits、prop_lits、reserved_word）
2. **ES2015 runtime**（function_name、duplicate_keys、instanceof、typeof_symbol）
3. **ES2016**（exponentiation）
4. **ES2019**（optional_catch_binding）
5. **ES2020 light**（nullish_coalescing、export_namespace_from）
6. **ES2021**（logical_assignments）
7. **Bugfixes**（所有 4 个）

**交付标准**：每个 transform 有对应的 Babel fixture 测试通过，
`cargo test -p swc_next_ecma_transformer` 全部通过。

#### Phase 2：中等复杂度 transforms（P0，中等）
需要一些 AST 操作但不需要全新语义 feature 的 transforms。

1. **ES2015**（template_literal、spread、computed_props、block_scoped_functions、
   new_target、object_super、for_of）
2. **ES2018**（object_rest_spread）
3. **ES2020**（optional_chaining）
4. **ES2022**（private_in_object、static_blocks）

**交付标准**：同上 + 不需要新增 `SemanticFeatureFlags`。

#### Phase 3：复杂 transforms（P0-P1，需要设计文档）
需要设计文档和新增 semantic feature 的 transforms。

1. **async_to_generator** — 需要 `AWAIT` + `ASYNC_FUNCTIONS` semantic feature
2. **block_scoping** — 循环闭包捕获、temporal dead zone
3. **classes** — class 到 function 的 lowering
4. **destructuring** — 完整解构模式处理
5. **parameters** — 默认参数、rest、解构参数
6. **class_properties** — class fields、private fields、static blocks

**交付标准**：每个设计文档完成评审，实现通过对应 Babel fixtures。

#### Phase 4：扩展 transforms（P1-P2）
TypeScript、React、Module、Proposal 等。

1. **TypeScript strip** — 类型擦除、enum、namespace
2. **JSX** — JSX transform 全家桶
3. **Module transforms** — CJS、AMD、UMD、SystemJS
4. **Decorators** — 2022-03 / 2023-11
5. **其他 proposal** — export_default_from、import_attributes、explicit_resource_management

**交付标准**：每个 transform 通过对应 Babel fixtures。

### 执行顺序原则

1. **P0 > P1 > P2**：按优先级排序
2. **简单 > 中等 > 复杂**：每个优先级内由易到难
3. **无依赖 > 有依赖**：基类 transforms 优先于依赖它们的 transforms
4. **轻量 runtime > 重量级**：visitor-style 优先于 program-level batch transforms

### 测试策略

1. 每个 transform 实现后，启用对应的 Babel fixture 测试
2. 对于有对应 Babel packages 的 transform（如 `babel-plugin-transform-arrow-functions`），
   直接用 Babel fixtures 验证
3. 对于 SWC 特有的 transform（如 `typeof_symbol`），使用 `tasks/testsuite` 中的
   babel 兼容测试或自定义 fixture
4. 所有 transform 共用 `cargo test -p swc_next_ecma_transformer` 测试入口

### 设计文档模板

对于标为"需要设计文档"的复杂 transforms，在 `docs/design/` 下创建文档，包含：

1. **目标**：transform 的输入输出示例
2. **Semantic 依赖**：需要哪些 `SemanticFeatureFlags`，是否需要新增 feature
3. **AST 操作分类**：新建节点、替换节点、删除节点、移动节点
4. **Scope 操作**：新建 scope、新增 binding、rename、rebind
5. **Helper 依赖**：是否需要 `_ts_generator` 等 helper 函数
6. **边界情况**：`with`、direct eval、TypeScript 互操作
7. **Babel 兼容性**：与 Babel 输出的差异点

## 进度跟踪

### 已完成

- [x] es2015.shorthand_properties — `{a}` → `{a: a}`
- [x] es2015.sticky_regex — `/foo/y` → `new RegExp("foo", "y")`
- [x] es2015.function_name — `var f = function() {}` → `var f = function f() {}`
- [x] es2015.duplicate_keys — `{a: 1, a: 2}` → `{a: 1, ["a"]: 2}`
- [x] es2015.instanceof — `a instanceof B` → `_instanceof(a, B)`
- [x] es2015.typeof_symbol — `typeof x` → `_typeof(x)`
- [x] es2015.template_literal — `` `hello ${name}` `` → `"hello ".concat(name)`
- [x] es2015.block_scoped_functions — 块级函数声明 → `let` 函数表达式
- [x] es2015.new_target — `new.target` → `this.constructor` / `void 0` / `this instanceof F ? ...`
- [x] es2015.computed_props — `{ [key]: value }` → `_define_property({}, key, value)`（单属性）
- [x] es2015.arrow_functions — `() => {}` → `function() {}`
- [x] es2015.generator — `function*` → regenerator state machine
- [x] es2015.spread — `f(...args)` → `f.apply(void 0, args)`
- [x] es2015.for_of — `for (var x of arr)` → iterator loop
- [x] es2016.exponentiation — `a ** b` → `Math.pow(a, b)`
- [x] es2019.optional_catch_binding — `try {} catch {}` → `try {} catch (_unused) {}`
- [x] es2020.nullish_coalescing — `a ?? b` → `a !== null && a !== void 0 ? a : b`
- [x] es2020.optional_chaining — `a?.b?.()` → `(_a = a) == null ? void 0 : _a.b()`
- [x] es2020.export_namespace_from — `export * as ns from "mod"` → `import * as _ns from "mod"; export { _ns as ns }`
- [x] es2021.logical_assignments — `a ||= b` → `a || (a = b)`
- [x] es2022.private_in_object — `#x in obj` → `_checkInRHS(obj)`
- [x] es2022.static_blocks — `class A { static {} }` → `class A { static #_ = (() => {})() }`
- [x] es3.member_expr_lits — `obj.const` → `obj["const"]`
- [x] es3.property_literals — `{ "bar": 1 }` → `{ bar: 1 }`
- [x] es3.reserved_words — `var abstract = 1` → `var _abstract = 1`
- [x] es2015.util.fn_env_hoister — 函数环境提升工具
- [x] bugfixes.edge_default_param — `{a = 1}` → `{a: a = 1}` in arrow params
- [x] bugfixes.async_arrows_in_class — class 方法中 async arrow → function expression

### P0（已全部完成）

所有 P0 transform 已实现，包括：

- [x] ES3: member_expr_lits, property_literals, reserved_words
- [x] ES2015: shorthand, sticky_regex, function_name, duplicate_keys, instanceof, typeof_symbol, template_literal, spread, computed_props, block_scoped_functions, for_of, new_target, arrow_functions, generator, object_super
- [x] ES2016: exponentiation
- [x] ES2017: async_to_generator
- [x] ES2016: exponentiation
- [x] ES2019: optional_catch_binding
- [x] ES2020: nullish_coalescing, optional_chaining
- [x] ES2021: logical_assignments
- [x] ES2022: private_in_object, static_blocks

### P1（待实现）

- [x] es2015.object_super — `super` in object literals, 使用 `_get`/`_set` helper
- [ ] es2015.block_scoping（需设计文档）
- [ ] es2015.classes（需设计文档）
- [ ] es2015.destructuring（需设计文档）
- [ ] es2015.parameters（需设计文档）
- [x] es2017.async_to_generator — `async function` → `_asyncToGenerator(function*(){...})()`
- [ ] es2018.object_rest_spread
- [ ] es2022.class_properties（需设计文档）
- [ ] bugfixes.safari_id_destructuring_collision_in_function_expression（Stub 已创建，需完整实现）
- [x] bugfixes.template_literal_caching — 模板字面量缓存优化
- [ ] typescript.strip（需设计文档）
- [ ] typescript.enum
- [ ] react.jsx（需设计文档）
- [ ] react.jsx_self
- [ ] react.jsx_src
- [ ] react.display_name
- [ ] react.pure_annotations
- [ ] proposal.export_default_from
- [ ] proposal.import_attributes

### P2（远期）

- [ ] react.refresh
- [ ] module.common_js（需设计文档）
- [ ] module.amd（需设计文档）
- [ ] module.umd（需设计文档）
- [ ] module.system_js（需设计文档）
- [ ] proposal.decorators（需设计文档）
- [ ] proposal.explicit_resource_management

## 文件结构

```
crates/swc_ecma_transformer/src/
├── lib.rs                    # Transformer 入口，组合所有 transform
├── state.rs                  # TransformerState、TransformerCtx
├── assumptions.rs            # Babel assumptions
├── es2015/
│   ├── mod.rs                # ES2015 组合（已存在）
│   ├── arrow.rs              # 箭头函数（已实现）
│   ├── shorthand.rs          # 简写属性（已实现）
│   ├── sticky_regex.rs       # sticky 正则（已实现）
│   ├── function_name.rs      # 函数名
│   ├── duplicate_keys.rs     # 重复键
│   ├── instanceof.rs         # instanceof
│   ├── typeof_symbol.rs      # typeof Symbol
│   ├── template_literal.rs   # 模板字面量
│   ├── spread.rs             # 展开
│   ├── computed_props.rs     # 计算属性
│   ├── block_scoped_fn.rs    # 块级函数
│   ├── for_of.rs             # for-of
│   ├── new_target.rs         # new.target
│   ├── object_super.rs       # 对象 super
│   ├── block_scoping/        # 块级作用域（模块）
│   ├── classes/              # 类（模块）
│   ├── destructuring/        # 解构（模块）
│   ├── parameters/           # 参数（模块）
│   ├── generator/            # 生成器（已实现）
│   │   ├── mod.rs
│   │   └── compiler/
│   │       ├── mod.rs
│   │       ├── expression.rs
│   │       └── statement.rs
│   └── util/                 # 工具
│       ├── mod.rs
│       └── fn_env_hoister.rs
├── es2016/
│   └── mod.rs                # 指数运算
├── es2017/
│   └── mod.rs                # async to generator
├── es2018/
│   └── mod.rs                # object rest/spread
├── es2019/
│   └── mod.rs                # optional catch binding
├── es2020/
│   └── mod.rs                # 含 nullish/optional/export-ns
├── es2021/
│   └── mod.rs                # logical assignments
├── es2022/
│   └── mod.rs                # class properties, static blocks, private-in
├── es3/
│   └── mod.rs                # member/prop/reserved
├── bugfixes/
│   └── mod.rs                # 所有 bugfixes
├── typescript/
│   └── mod.rs                # TS strip
├── react/
│   └── mod.rs                # JSX 全家桶
├── module/
│   └── mod.rs                # CJS/AMD/UMD/SystemJS
└── proposal/
    └── mod.rs                # decorators, etc.
```

## 验证

- 每个 transform 实现后，运行 `cargo test -p swc_next_ecma_transformer` 确保编译通过
- 运行 `cargo test -p testsuite` 确保 Babel fixtures 通过
- 运行 `cargo clippy -p swc_next_ecma_transformer -- -D warnings` 确保无 lint 警告
- 对于可 benchmark 的 transform，运行对应 benchmark 确保无回归