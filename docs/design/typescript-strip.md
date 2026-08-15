# TypeScript Strip Transform Design

## 目标

```js
// Input
function foo<T>(x: T): T {
  const y: number = 1;
  return x;
}

// Output
function foo(x) {
  const y = 1;
  return x;
}
```

```js
// Input: enum
enum Color {
  Red,
  Green = 2,
  Blue
}

// Output
var Color;
(function(Color) {
  Color[Color["Red"] = 0] = "Red";
  Color[Color["Green"] = 2] = "Green";
  Color[Color["Blue"] = 3] = "Blue";
})(Color || (Color = {}));
```

```js
// Input: namespace
namespace Foo {
  export const x = 1;
}

// Output
var Foo;
(function(Foo) {
  Foo.x = 1;
})(Foo || (Foo = {}));
```

## Semantic 依赖

- `TS_TYPE_NODES` — 需要新增，标记所有 TS 类型节点
- `TS_ENUM` — 需要新增，标记所有 enum 声明
- `TS_NAMESPACE` — 需要新增，标记所有 namespace/module 声明

## 需要新增的 SemanticFeatureFlags

- `TS_TYPE_NODES` — 预收集所有 TS 类型注解的位置
- `TS_ENUM` — 预收集所有 enum 声明
- `TS_NAMESPACE` — 预收集所有 namespace 声明

## AST 操作分类

### 类型擦除

1. **删除类型注解**：`x: T` → `x`（移除 `TsTypeAnnotation`）
2. **删除类型参数**：`<T extends Foo>` → 删除 `TsTypeParameterDeclaration`
3. **删除接口**：`interface Foo {}` → 完全删除
4. **删除类型别名**：`type Foo = Bar` → 完全删除
5. **删除类型断言**：`x as T` → `x`（保留 `as` 表达式）
6. **删除类型导入**：`import type { Foo } from 'bar'` → 删除
7. **删除类型导出**：`export type { Foo }` → 删除

### Enum 转换

1. **EnumDeclaration** → `var` + IIFE 模式
2. **EnumMember** → 反向映射（`Color[Color["Red"] = 0] = "Red"`）
3. **const enum** → 内联展开（所有引用替换为常量值）

### Namespace 转换

1. **ModuleDeclaration** → `var` + IIFE 模式
2. **Exported 声明** → 挂载到 namespace 对象上
3. **嵌套 namespace** → 递归展开

## Scope 操作

- 类型删除不会影响 scope（类型不在 runtime scope 中）
- Enum 和 namespace 转换会创建新的 scope（IIFE）
- `import type` 删除后，对应的 binding 也需要删除

## Helper 依赖

- 无（enum 和 namespace 的转换是自包含的，不依赖 helper）

## 边界情况

1. **`import type` vs `import`**：`import type { Foo }` 直接删除；`import { type Foo, Bar }` 保留 `Bar`，删除 `Foo`
2. **`export type`**：`export type { Foo }` 删除
3. **`const enum`**：直接内联值，不生成 runtime 代码
4. **`declare` 声明**：全部删除
5. **`enum` 的 `const` 引用**：跨文件的 `const enum` 引用需要内联
6. **`namespace` 合并**：多个同名 namespace 合并到同一个 IIFE
7. **`abstract class`**：删除 `abstract` 关键字，保留 class
8. **`implements`**：`class Foo implements Bar` → `class Foo`
9. **装饰器**：`@decorator` 保留（由 decorator transform 处理）
10. **`as const`**：`as const` 断言删除，但保留 `const` 上下文

## 实现策略

1. 新增 `TS_TYPE_NODES`、`TS_ENUM`、`TS_NAMESPACE` semantic feature
2. 类型擦除：在 visitor hook 中删除所有 TS 类型节点
3. Enum 转换：在 `enter_program` 中处理所有 enum 声明
4. Namespace 转换：在 `enter_program` 中处理所有 namespace 声明
5. 注意顺序：先 enum/namespace 转换，再类型擦除