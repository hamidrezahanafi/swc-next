# Module Transforms Design

## 目标

### CommonJS

```js
// Input
import { foo } from './bar';
export const baz = 1;

// Output
const { foo } = require('./bar');
exports.baz = 1;
Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
```

### AMD

```js
// Input
import { foo } from './bar';
export const baz = 1;

// Output
define(['require', 'exports', './bar'], function(require, exports, _bar) {
  'use strict';
  Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
  exports.baz = 1;
});
```

### UMD

```js
// Input
import { foo } from './bar';
export const baz = 1;

// Output
(function(root, factory) {
  if (typeof define === 'function' && define.amd) {
    define(['require', 'exports', './bar'], factory);
  } else if (typeof exports === 'object') {
    factory(require, exports, require('./bar'));
  } else {
    factory(root, (root.returnExports = {}), root.bar);
  }
})(typeof globalThis !== 'undefined' ? globalThis : this, function(require, exports, _bar) {
  'use strict';
  exports.baz = 1;
});
```

### SystemJS

```js
// Input
import { foo } from './bar';
export const baz = 1;

// Output
System.register(['./bar'], function(exports_1, context_1) {
  var foo, baz;
  return {
    setters: [
      function(module_1) {
        foo = module_1.foo;
      },
    ],
    execute: function() {
      exports_1('baz', 1);
    },
  };
});
```

## 公共子模块

所有 module transform 共享以下子模块：

1. **Import 分析** — 分析 `import` 声明，收集 source、specifiers
2. **Export 分析** — 分析 `export` 声明，收集 exported names
3. **Rewriter** — 重写标识符，替换 `import`/`export` 为对应格式
4. **Scope hoisting** — 作用域提升（可选，用于 tree-shaking）

## 需要新增的 SemanticFeatureFlags

- `IMPORT_EXPORT` — 预收集所有 import/export 声明
- `REEXPORT` — 预收集所有 re-export（`export { x } from 'y'`）
- `EXPORT_DEFAULT` — 预收集 `export default` 声明

## AST 操作分类

### Import → require/define

1. **ImportDeclaration** → `const X = require('module')`（CJS）
2. **ImportDefaultSpecifier** → `const X = require('module').default`
3. **ImportNamespaceSpecifier** → `const X = require('module')`
4. **NamedImport** → `const { x } = require('module')` 或 destructure

### Export → exports/define

1. **NamedExport** → `exports.X = value`
2. **ExportDefault** → `exports.default = value`
3. **ExportAll** → `Object.keys(require('module')).forEach(k => exports[k] = require('module')[k])`
4. **Re-export** → `Object.defineProperty(exports, 'x', { get: () => require('module').x })`

## Helper 依赖

- `_interopRequireDefault(obj)` — ES module 的 `default` interop
- `_interopRequireWildcard(obj)` — ES module 的 namespace interop
- `_exportStar(exports, target)` — re-export all
- 各格式特有：UMD wrapper、SystemJS register

## 边界情况

1. **`import x from 'y'` vs `import * as x from 'y'`**：前者需要 `.default`，后者是 namespace
2. **`export default function`**：`function` 声明提升
3. **`export { x } from 'y'`**：re-export 需要 getter（延迟引用）
4. **`export * from 'y'`**：export star 排除 `default`
5. **Mixed exports**：`named` + `default` 同时存在
6. **Live bindings**：ES module 的 export 是实时绑定，CJS 需要 getter
7. **`import.meta`**：`{ import.meta: { url } }` → CJS 中转换为 `{ url: require('url').pathToFileURL(__filename).href }`
8. **`__esModule` 标记**：CJS/AMD/UMD 需要 `Object.defineProperty(exports, '__esModule', { value: true })`
9. **循环依赖**：CJS 中通过 `require()` 的延迟执行处理
10. **TypeScript 互操作**：`import type` 不生成 import

## 实现策略

1. 新增 `IMPORT_EXPORT` semantic feature
2. 在 `enter_program` 中分析所有 import/export 声明
3. 根据配置选择目标格式（CJS/AMD/UMD/SystemJS）
4. 重写 Module 的 body，替换 import/export 为对应格式
5. 注入 format-specific wrapper（function 声明、define 调用、System.register 等）