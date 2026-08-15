# Class Properties Transform Design

## 目标

```js
// Input
class Foo {
  x = 1;
  #y = 2;
  static z = 3;
  get #priv() { return this.#y; }
  [Symbol.iterator]() {}
}

// Output
class Foo {
  constructor() {
    this.x = 1;
    _y.set(this, 2);
  }
  get #priv() { return _y.get(this); }
  [Symbol.iterator]() {}
  static z = 3;
}
```

## Component Transforms

这个 transform 实际上是三个独立的 Babel 插件的组合：

1. **class-properties** — 实例属性（public fields、private fields）
2. **private-methods** — private methods/getter/setter
3. **static-blocks** — `static {}` 块

## Semantic 依赖

- `CLASS_DECLARATIONS` — 需要新增，标记 class 声明
- `PRIVATE_FIELDS` — 需要新增，标记 private field 使用
- `PRIVATE_METHODS` — 需要新增，标记 private method 定义

## 需要新增的 SemanticFeatureFlags

- `CLASS_DECLARATIONS` — 预收集 class 声明
- `PRIVATE_FIELDS` — 预收集 private field 的访问
- `PRIVATE_METHODS` — 预收集 private method 定义

## AST 操作分类

### Public Fields

1. **实例属性**：`x = 1` → 移到 constructor 中作为 `this.x = 1`
2. **静态属性**：`static x = 1` → `Foo.x = 1`（在 class 定义之后）

### Private Fields

1. **`#x = val`** → `_x = new WeakMap(); _x.set(this, val)` 在 constructor 中
2. **`#x` 引用** → `_x.get(this)` / `_x.set(this, val)`
3. **`#x in obj`** → `_x.has(obj)`

### Private Methods

1. **`#method() {}`** → `_method = new WeakSet(); _method.add(this)` 在 constructor 中
2. **`obj.#method()`** → 使用 WeakSet 的 brand check 确保方法绑定

## Scope 操作

- 实例属性移到 constructor 中（同一个 scope）
- private field 创建 WeakMap binding（class scope 外）
- private method 创建 WeakSet binding（class scope 外）

## Helper 依赖

- `_classCallCheck(instance, Constructor)` — 检查是否用 `new` 调用
- `_defineProperty(obj, key, value)` — 定义属性
- `_classPrivateFieldInit(obj, privateMap, value)` — 初始化 private field
- `_classPrivateFieldGet(obj, privateMap)` — 读取 private field
- `_classPrivateFieldSet(obj, privateMap, value)` — 写入 private field
- `_classPrivateMethodInit(obj, privateSet)` — 初始化 private method
- `_classPrivateMethodGet(obj, privateSet, fn)` — 调用 private method
- `_checkPrivateRedeclaration(obj, privateCollection)` — 重新声明检查

## 边界情况

1. **`super()` 前不能有属性赋值**：`class Foo extends Bar { x = 1; constructor() { super(); } }` → 属性赋值必须在 `super()` 之后
2. **没有 constructor 的 class**：需要合成 constructor
3. **`#x in obj` 在 class 外**：需要 WeakMap 的 brand check
4. **Private method 的 `this` 绑定**：方法调用时需要确保 `this` 指向实例
5. **Computed property name**：`[expr] = val` → 需要缓存 `expr`
6. **`Symbol.toStringTag` 等内置 symbol**：`static [Symbol.species] = ...` → 正常处理

## 实现策略

1. 新增 `CLASS_DECLARATIONS`、`PRIVATE_FIELDS`、`PRIVATE_METHODS` semantic feature
2. 在 `enter_class` 中收集 class 的属性和方法信息
3. 在 `exit_class` 中：
   - 创建 WeakMap/WeakSet 变量
   - 将实例属性注入 constructor
   - 替换 `#x` 引用为 WeakMap 操作
   - 处理静态属性
4. `static_blocks` 作为独立 transform 在前处理