# Object Super Transform Design

## 目标

将对象字面量方法中的 `super.prop` 引用转换为 helper 调用。

### 输入/输出

```js
// Input:
let obj = {
    foo() {
        super.bar();
    }
}

// Output:
let obj = (() => {
    var _obj = {
        foo() {
            _get(_get_prototype_of(_obj), "bar", this).call(this);
        }
    };
    return _obj;
})();
```

## Semantic 依赖

无。`super` 在对象方法中的行为是静态确定的，不需要运行时 semantic 分析。

## AST 操作分类

### 1. 对象字面量包装

当对象字面量的任何方法中包含 `super` 引用时，需要将整个对象字面量包装为 IIFE：

```js
// Before:
{ foo() { super.bar(); } }

// After IIFE:
(() => { var _obj = { foo() { ... } }; return _obj; })()
```

### 2. super.foo → _get 调用

```js
// super.foo
super.foo → _get(_get_prototype_of(_obj), "foo", this)
```

### 3. super.foo(args) → _get.call

```js
// super.foo(1, 2, 3)
super.foo(1, 2, 3) → _get(_get_prototype_of(_obj), "foo", this).call(this, 1, 2, 3)

// super.foo(...args) → _get.apply
super.foo(...args) → _get(_get_prototype_of(_obj), "foo", this).apply(this, args)
```

### 4. super.foo = val → _set 调用

```js
// super.foo = val
super.foo = val → _set(_get_prototype_of(_obj), "foo", val, this, true)

// super.foo += val → _get + _set
super.foo += val → _set(_get_prototype_of(_obj), "foo", _get(_get_prototype_of(_obj), "foo", this) + val, this, true)
```

## 实现策略

### 方法一：exit_object_expression 中重建

在 `exit_object_expression` hook 中：

1. 遍历所有 property，检查是否有 method/getter/setter 包含 `super` 引用
2. 如果包含，包裹为 IIFE 并重建方法体
3. 非方法（函数表达式、箭头函数等）不应转换 `super`

### 方法二：分离的 Expr visitor

为包含 `super` 的方法体单独生成 visitor，仅替换 `Expr::Super` 相关的模式。

## Helper 依赖

需要以下 runtime helper：
- `_get` — 属性读取
- `_set` — 属性赋值
- `_get_prototype_of` — 获取原型

## 边界情况

1. **嵌套对象**：内层对象方法的 `super` 不应被外层 IIFE 的 `_obj` 捕获
2. **非方法函数**：`{ foo: function() { super.x } }` 中的 `super` 无效，不应转换
3. **箭头函数**：`{ foo() { () => super.x } }` 箭头函数继承 `super`，应该转换
4. **getter/setter**：`{ get foo() { super.x } }` 需要转换
5. **多个方法**：多个方法包含 `super` 时，共享同一个 `_obj` 引用
6. **计算属性键**：`{ [super.foo]() {} }` — 属性键中的 `super` 不应转换（无效语法？）

## Babel 兼容性

与 `@babel/plugin-transform-object-super` 行为一致。