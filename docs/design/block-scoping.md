# Block Scoping Transform Design

## 目标

```js
// Input
{
  let x = 1;
  const y = 2;
  console.log(x);
}

// Output
{
  var x = 1;
  var y = 2;
  console.log(x);
}
```

```js
// Input: for loop closure capture
var funcs = [];
for (let i = 0; i < 3; i++) {
  funcs.push(function() { return i; });
}

// Output
var funcs = [];
var _loop = function(i) {
  funcs.push(function() { return i; });
};
for (var i = 0; i < 3; i++) {
  _loop(i);
}
```

## Semantic 依赖

- `LET_DECLARATIONS` — 需要新增，标记 `let`/`const` 声明的位置
- `FOR_LOOP_LET` — 需要新增，标记 for 循环的 `let` 声明

## 需要新增的 SemanticFeatureFlags

- `LET_DECLARATIONS` — 预收集所有 `let`/`const` 声明
- `FOR_LOOP_LET` — 预收集 for 循环的 `let` 声明（用于闭包捕获分析）

## AST 操作分类

1. **替换声明**：`let x = 1` → `var x = 1`
2. **闭包捕获**：for 循环中的 `let` 声明如果有闭包引用，需要创建 `_loop` IIFE
3. **TDZ 检查**：`let x = y; let y = 1;` → 需要 `_check` 调用（可选）

## Scope 操作

- 不需要新建 scope（`let` → `var` 后在同一 scope 中）
- for 循环的 `let` 每次迭代需要新建 scope（通过 IIFE 模拟）

## Helper 依赖

- 无（标准模式不需要 helper）
- 可选：`_temporalUndefined`、`_temporalRef`（如果需要 TDZ 检查）

## 边界情况

1. **`let` 在 for 中**：`for (let i = 0; ...)` 每次迭代创建新绑定
2. **`for...in`/`for...of` 的 `let`**：`for (let x in obj)` 类似处理
3. **闭包捕获**：for 循环体中的函数引用 `let` 变量时需要 IIFE
4. **`switch` 中的 `let`/`const`**：`case` 块不会创建新 scope，需要处理
5. **`const` 重新赋值**：`const` → `var` 后不会检查重新赋值，需要 `_readOnlyError`（可选）
6. **TDZ（Temporal Dead Zone）**：`let x = x` 在 `var` 中不会报错，需要 `_temporalRef`
7. **`typeof` 例外**：`typeof x` 在 TDZ 中会报错，但 `typeof undeclared` 不会

## Babel 兼容性

Babel 的 `@babel/plugin-transform-block-scoping` 处理：
- `let`/`const` → `var`
- for 循环闭包捕获（`_loop` IIFE）
- TDZ 检查（`_temporalRef`）
- `const` 重新赋值检查（`_readOnlyError`）

SWC 的 `loose` 模式跳过 TDZ 和 `const` 检查，`assumeArray` 影响 for-of 的展开。

## 实现策略

1. 新增 `LET_DECLARATIONS` 和 `FOR_LOOP_LET` semantic feature
2. 在 `enter_program` 中预处理所有 let/const 声明
3. 分两步：先处理 for 循环闭包捕获，再替换 let/const → var
4. 对于 TDZ 检查，在声明前插入 `_temporalRef` 调用