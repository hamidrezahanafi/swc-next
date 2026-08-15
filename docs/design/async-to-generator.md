# Async to Generator Transform Design

## 目标

```js
// Input
async function foo() {
  await bar();
  return baz();
}

// Output
function foo() {
  return _asyncToGenerator(function*() {
    yield bar();
    return baz();
  })();
}
```

## Semantic 依赖

- `AWAIT` — 子树是否包含 `await`
- `ASYNC_FUNCTIONS` — 后序遍历所有 async function
- `FUNCTION_ENV_CAPTURE` — 函数环境捕获（`arguments`、`new.target`、`this`、`super`）

## AST 操作

### 需要新增的 SemanticFeatureFlags

- `AWAIT` — 已定义
- `ASYNC_FUNCTIONS` — 已定义

### 操作分类

1. **识别 async function**：遍历所有 `Function` 节点，检查 `r#async: true`
2. **替换 Function**：将 `async function foo() { ... }` 替换为 `function foo() { return _asyncToGenerator(function*() { ... })(); }`
3. **替换 ArrowFunction**：`async () => {}` → `_asyncToGenerator(function*() {})()`
4. **Helper 调用**：在 Program 顶部插入 `_asyncToGenerator` helper 定义

## Scope 操作

- 新建 generator scope（内部 function* 的 scope）
- 新增 `_asyncToGenerator` binding（module scope）

## Helper 依赖

- `_asyncToGenerator(fn)` — 在 `@swc/helpers` 中定义，将 generator 包装为 async 兼容函数

## 边界情况

1. **`async function` 声明**：转换为普通函数声明，body 用 generator wrapper 替换
2. **`async function` 表达式**：转换为普通函数表达式
3. **`async () => {}` 箭头函数**：转换为 `_asyncToGenerator(function*() {})()`
4. **`super` 在 async arrow 中**：需要 `FUNCTION_ENV_CAPTURE` 捕获 `this`
5. **`arguments` 在 async arrow 中**：需要 `FUNCTION_ENV_CAPTURE` 捕获 `arguments`
6. **`await` 不在 async 中**：语法错误，不处理
7. **嵌套 async**：内层 async 应已被处理，不会出现 `await` 在普通函数中

## Babel 兼容性

Babel 的 `@babel/plugin-transform-async-to-generator` 使用 `_asyncToGenerator` helper，输出格式与上述一致。SWC 对 `async` 箭头函数的处理略有不同：Babel 在箭头函数中会保留 `this` 绑定，而 SWC 将 `async () => {}` 一并转换为 generator wrapper。

## 实现策略

1. 在 `enter_program` 中，利用 `ASYNC_FUNCTIONS` semantic feature 后序遍历所有 async function
2. 对于每个 async function：
   - 创建 `function*` 替换原函数体
   - 包裹 `_asyncToGenerator(func)()`
   - 将 `await` 替换为 `yield`
3. 在 Program 顶部注入 `_asyncToGenerator` helper 声明