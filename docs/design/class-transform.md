# Class Transform Design

## 目标

```js
// Input
class Foo extends Bar {
  constructor(x) {
    super(x);
    this.x = x;
  }
  method() {
    return this.x;
  }
}

// Output
function Foo(x) {
  this.x = x;
}
Foo.prototype = Object.create(Bar.prototype, {
  constructor: { value: Foo, enumerable: false, writable: true, configurable: true }
});
Foo.prototype.method = function() {
  return this.x;
};
```

## Semantic 依赖

- `CLASS_DECLARATIONS` — 需要新增，标记所有 class 声明
- `CLASS_METHODS` — 需要新增，标记 class 方法
- `SUPER_CALL` — 需要新增，标记 `super()` 调用
- `FUNCTION_ENV_CAPTURE` — 函数环境捕获

## 需要新增的 SemanticFeatureFlags

- `CLASS_DECLARATIONS` — 预收集所有 class 声明/表达式
- `CLASS_METHODS` — 预收集 class 方法定义
- `SUPER_CALL` — 预收集 `super()` 调用

## AST 操作分类

1. **ClassDeclaration → FunctionDeclaration**：`class Foo {}` → `function Foo() {}`
2. **ClassExpression → FunctionExpression**：`const Foo = class {}` → `const Foo = function() {}`
3. **Constructor**：提取 constructor body 作为函数体
4. **Methods**：挂载到 prototype 上
5. **Static methods**：挂载到 constructor 上
6. **Getter/Setter**：使用 `Object.defineProperty`
7. **`super()` 调用**：在 constructor 中保留
8. **`super.prop` 引用**：转换为 `SuperGet/SuperSet` 调用

## Scope 操作

- 新建 class function scope
- 原有 class scope 的两个层次（class 名 + 方法体）需要合并到 function + prototype 模式

## Helper 依赖

- `_inherits(subClass, superClass)` — 原型链继承
- `_createSuper(Derived)` — `super()` 调用包装（支持 `Reflect.construct`）
- `_createClass(methods, staticMethods)` — 创建 class 的 prototype/static 方法
- `_setPrototypeOf(o, p)` — `Object.setPrototypeOf` polyfill
- `_getPrototypeOf(o)` — `Object.getPrototypeOf` polyfill
- `_defineProperty(obj, key, value)` — `Object.defineProperty` 包装

## 边界情况

1. **`super()` 在 constructor 中**：必须作为第一个表达式调用
2. **`super.prop` 在方法中**：转换为 `_get(_getPrototypeOf(Class.prototype), 'prop', this)`
3. **`new.target`**：在 class constructor 中转换为 `this.constructor`
4. **`extends null`**：特殊处理，`_inherits(Foo, null)`
5. **Class expression**：`let Foo = class {}` 需要保留变量名
6. **Computed method names**：`[Symbol.iterator]() {}` → 使用 `_defineProperty`
7. **Private methods**：不处理（由 class-properties 或其他 transform 处理）
8. **`return` 语句在 constructor 中**：`return` 一个对象时，`this` 绑定会被替代

## Babel 兼容性

Babel 的 `@babel/plugin-transform-classes` 使用 `_createClass`、`_inherits`、`_createSuper` 等 helper。`loose` 模式下：
- `_inherits` → `B.prototype = A.prototype`（直接赋值）
- `_createClass` → 直接赋值到 prototype
- `_createSuper` → 直接调用 `Parent.call(this)`

## 实现策略

1. 新增 `CLASS_DECLARATIONS`、`CLASS_METHODS`、`SUPER_CALL` semantic feature
2. 在 `enter_program` 中遍历所有 class，收集 class 结构信息
3. 对于每个 class：
   - 创建 constructor function
   - 遍历 methods，挂载到 prototype 或 static
   - 处理 `extends` 和 `super()`
   - 注入 helper 调用
4. 在 Program 顶部注入 helper 声明