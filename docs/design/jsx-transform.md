# JSX Transform Design

## 目标

```js
// Input (React 17+ automatic runtime)
const App = () => <div className="app">Hello</div>;

// Output
import { jsx as _jsx } from "react/jsx-runtime";
const App = () => _jsx("div", { className: "app", children: "Hello" });
```

```js
// Input (React 17+ automatic runtime, dev)
const App = () => <div className="app">
  <span>Hello</span>
</div>;

// Output (dev mode with jsxDEV)
import { jsxDEV as _jsxDEV } from "react/jsx-dev-runtime";
const App = () => _jsxDEV("div", { className: "app", children: _jsxDEV("span", { children: "Hello" }, void 0, false, { fileName: "App.jsx", lineNumber: 2 }, this) });
```

```js
// Input (classic runtime)
const App = React.createElement("div", { className: "app" }, "Hello");

// Output (same)
```

## 配置

- `runtime: "automatic" | "classic"` — 使用哪个 runtime
- `importSource: string` — 自动 runtime 的 import 源（默认 `"react"`）
- `pragma: string` — classic runtime 的 pragma（默认 `"React.createElement"`）
- `pragmaFrag: string` — fragment pragma（默认 `"React.Fragment"`）
- `throwIfNamespace: boolean` — 是否在 XML namespace 标签上抛出错误
- `useBuiltIns: boolean` — 是否使用原生 `Array.isArray` 等
- `development: boolean` — 是否使用 `jsxDEV`（开发模式）

## Semantic 依赖

- `JSX_ELEMENTS` — 需要新增，标记所有 JSX 元素
- `REACT_IMPORT` — 需要新增，检测是否已有 React import

## 需要新增的 SemanticFeatureFlags

- `JSX_ELEMENTS` — 预收集所有 JSX 元素
- `REACT_IMPORT` — 预收集 `import React from 'react'` 声明

## AST 操作分类

### JSXElement → JSX 函数调用

1. **JSXElement** → `_jsx(type, props, key)` 或 `React.createElement(type, props, ...children)`
2. **JSXFragment** → `_jsxFragment()` 或 `React.createElement(React.Fragment, ...)`
3. **JSXText** → 字符串字面量
4. **JSXExpressionContainer** → 保留表达式

### Props 处理

1. **属性展开**：`{...props}` → 展开到 props 对象中
2. **字符串属性**：`className="app"` → `{ className: "app" }`
3. **表达式属性**：`id={getId()}` → `{ id: getId() }`
4. **布尔属性**：`disabled` → `{ disabled: true }`
5. **key 属性**：特殊处理，作为 JSX 调用的第二个参数

### 自动 Import

1. 自动 runtime：在模块顶部添加 `import { jsx as _jsx } from 'react/jsx-runtime'`
2. 检查是否已有 import，避免重复

## Scope 操作

- 自动 runtime 需要添加 import binding（module scope）
- 不需要新建 scope

## Helper 依赖

- 自动 runtime：`jsx`/`jsxs`/`jsxDEV` 从 `react/jsx-runtime` 或 `react/jsx-dev-runtime`
- Classic runtime：`React.createElement`（用户代码中已存在）

## 边界情况

1. **`key` 属性**：自动 runtime 中作为第二个参数，classic 中放在 props 中
2. **`ref` 属性**：自动 runtime 中作为第三个参数
3. **`__self` 和 `__source`**：开发模式中附加
4. **Fragment**：`<></>` → `_jsxFragment()` 或 `React.createElement(React.Fragment, null)`
5. **HTML 实体**：`&nbsp;` → 保留为 `\u00A0`
6. **自闭合标签**：`<br />` → 无 children
7. **XML namespace**：`<ns:tag>` → 根据配置抛错或保留
8. **Props 中的 Spread**：`<Comp {...props} key="a">` → key 在 spread 之后
9. **Children 混合**：文本 + 表达式 + JSX 元素混合作为 children
10. **`this` 在 JSX 中**：开发模式自动注入 `this`

## 实现策略

1. 新增 `JSX_ELEMENTS` semantic feature
2. 根据配置选择 runtime 模式
3. 在 `enter_program` 中处理 import 注入
4. 在 visitor hook 中处理 JSX 元素转换
5. 自动 runtime 和 classic runtime 共享相同的 JSX→函数调用逻辑，仅 import 和处理方式不同