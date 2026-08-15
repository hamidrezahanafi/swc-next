# Destructuring Transform Design

## 目标

```js
// Input: object destructuring
var { a, b = 3, c: d } = obj;

// Output
var _obj = obj,
    a = _obj.a,
    b = _obj.b === void 0 ? 3 : _obj.b,
    d = _obj.c;
```

```js
// Input: array destructuring
var [a, b, ...rest] = arr;

// Output
var a = arr[0],
    b = arr[1],
    rest = arr.slice(2);
```

```js
// Input: nested destructuring
var { a: { b } } = obj;

// Output
var _obj = obj,
    _obj_a = _obj.a,
    b = _obj_a.b;
```

## Semantic 依赖

- 无（不需要新增 semantic feature，纯本地 AST 操作）

## AST 操作分类

1. **替换 VariableDeclarator**：`var {a, b} = obj` → 多个 `var` 声明
2. **替换 AssignmentExpression**：`({a, b} = obj)` → 逗号表达式
3. **替换 Parameter**：`function({a, b}) {}` → `function(_ref) { var a = _ref.a, b = _ref.b; }`
4. **替换 ForIn/ForOf left**：`for (var [a, b] of arr)` → 展开

## Scope 操作

- 参数解构需要新建 function scope（添加 `_ref` 参数）
- 变量声明解构在同一 scope 中展开

## Helper 依赖

- `_slicedToArray(arr, n)` — 数组解构（`[a, b] = arr`）
- `_toArray(arr)` — 数组解构用于 rest（`[a, ...rest] = arr`）
- `_toConsumableArray(arr)` — 展开运算符
- `_arrayLikeToArray(arr, n)` — 类数组转换
- `_iterableToArray(arr)` — 可迭代对象转换
- `_unsupportedIterableToArray(arr, n)` — 不支持的可迭代对象
- `_nonIterableRest()` — 非可迭代对象 rest 错误
- `_objectDestructuringEmpty(obj)` — 对象解构空值检查
- `_extends()` — 对象展开

## 边界情况

1. **默认值**：`{a = 1} = obj` → `var _a = obj.a, a = _a === void 0 ? 1 : _a`
2. **嵌套解构**：`{a: {b}} = obj` → 需要中间变量
3. **Rest 模式**：`{a, ...rest} = obj` → `var rest = _objectWithoutProperties(obj, ['a'])`
4. **数组解构的迭代器**：`[a, b] = arr` → 需要检查 `Symbol.iterator`
5. **`null`/`undefined` 检查**：`{a} = null` 抛出 TypeError
6. **`for...of` 中的解构**：`for (var [a, b] of arr)` → 展开为 `for (var _ref of arr) { var a = _ref[0], b = _ref[1]; }`
7. **参数解构**：`function({a, b})` → `function(_ref) { var a = _ref.a, b = _ref.b; }`
8. **Computed 属性名**：`{ [key]: value } = obj` → 需要缓存 key

## Babel 兼容性

Babel 的 `@babel/plugin-transform-destructuring` 支持 `loose` 模式：
- 数组解构跳过迭代器检查，直接使用下标
- 对象解构跳过 `hasOwnProperty` 检查

## 实现策略

1. 使用 visitor hook 遍历 `VariableDeclarator`、`AssignmentExpression`、`FormalParameter`
2. 对于每个解构模式：
   - 创建临时变量 `_ref` 保存待解构对象
   - 展开每个属性/元素到对应变量
   - 处理默认值
   - 处理 rest 模式
3. 参数解构在处理 `FormalParameters` 时完成