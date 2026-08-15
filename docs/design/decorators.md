# Decorators Transform Design

## 目标

### 2023-11 规范（新版）

```js
// Input
@decorator
class Foo {
  @dec method() {}
  @dec field = 1;
}

// Output: 2023-11 规范使用 @decorator 作为标准语法，
// 需要 Babel 辅助函数来 polyfill
```

### 2022-03 规范（旧版，legacy）

```js
// Input
@decorator
class Foo {
  @dec method() {}
  @dec field = 1;
}

// Output
var Foo = (dec) => {
  class Foo {
    method() {}
    field = 1;
  }
  Foo = dec(Foo) || Foo;
  return Foo;
})();
```

## 配置

- `version: "2023-11" | "2022-03"` — 使用的 decorator 规范版本
- `decoratorsBeforeExport: boolean` — decorator 在 export 之前还是之后

## 需要新增的 SemanticFeatureFlags

- `DECORATOR` — 预收集所有 decorator 表达式
- `DECORATED_CLASS` — 预收集所有被装饰的 class
- `DECORATED_MEMBER` — 预收集所有被装饰的成员

## AST 操作分类

### 2023-11 规范

1. **ClassDecorator** → `addInitializer` 注册
2. **FieldDecorator** → `init` 钩子
3. **MethodDecorator** → `get`/`set` 钩子
4. **AutoAccessor** → `initialize` 钩子

### 2022-03 (Legacy) 规范

1. **ClassDecorator** → `dec(Class) || Class`
2. **MethodDecorator** → `Object.defineProperty(Class.prototype, key, dec(descriptor))`
3. **FieldDecorator** → `init` 在 constructor 中
4. **装饰器应用顺序**：从内到外（方法 → 类）

## Helper 依赖

- 2023-11：`_applyDecoratedDescriptor`、`_initDefineProp`、`_decorateClass`
- 2022-03：`_decorate`、`_classPrivateFieldInit`、`_applyDecoratedDescriptor`

## 边界情况

1. **`export default @dec class {}`**：decorator 在 export 之前
2. **多个 decorator**：`@a @b class Foo` → 从右到左应用
3. **Decorator 参数**：`@dec(1, 2)` → 参数在 decorator 外层
4. **`@register` 模式**：`@register` 返回函数
5. **Private 成员装饰**：`@dec #x = 1` → 需要 WeakMap 支持
6. **Static 成员装饰**：`@dec static x = 1` → 装饰在 constructor 上
7. **Getter/Setter 装饰**：`@dec get x() {}` → 装饰 descriptor 的 get/set
8. **Auto accessor**：`@dec accessor x = 1` → 2023-11 特有的语法

## 实现策略

1. 新增 `DECORATOR`、`DECORATED_CLASS`、`DECORATED_MEMBER` semantic feature
2. 根据配置选择规范版本
3. 在 `enter_class`/`exit_class` 中处理 decorator
4. 收集所有 decorator 信息，按规范顺序应用
5. 注入 helper 调用