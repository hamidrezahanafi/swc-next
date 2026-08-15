# ES2020 Optional Chaining Transform Design

## 目标

将 `?.` 可选链操作符转换为条件表达式。

### 输入/输出

```js
// Input:
a?.b
// Output:
a === null || a === void 0 ? void 0 : a.b

// Input:
a?.b()
// Output:
a === null || a === void 0 ? void 0 : a.b()

// Input:
a?.b?.c
// Output:
a === null || a === void 0 ? void 0 : a.b === null || a.b === void 0 ? void 0 : a.b.c

// Input:
a?.b.c()
// Output:
a === null || a === void 0 ? void 0 : a.b.c()
```

## AST 结构

`a?.b` 在 swc-next 中表示为：

```
ChainExpression(
  MemberExpression(object: a, property: b, optional: true)
)
```

`a?.b()` 表示为：

```
ChainExpression(
  CallExpression(
    callee: MemberExpression(object: a, property: b, optional: true),
    optional: false
  )
)
```

`a?.b?.c` 表示为：

```
ChainExpression(
  MemberExpression(
    object: MemberExpression(object: a, property: b, optional: true),
    property: c,
    optional: true
  )
)
```

## 实现策略

1. 在 `exit_chain_expression` hook 中处理
2. 递归处理链式表达式，从根到叶
3. 对于每个 `optional: true` 的节点，构建条件表达式

### 递归函数

```rust
fn lower_chain(expr: Expr, ctx) -> Expr {
  match expr {
    MemberExpression(member) if member.optional => {
      let base = member.object;
      let lowered = lower_chain(base, ctx);
      // 构建 (base === null || base === void 0) ? void 0 : base.prop
      Conditional(
        test: Binary(base, ===, null) || Binary(base, ===, void 0),
        consequent: void 0,
        alternate: MemberExpression { object: lowered, optional: false, ... }
      )
    }
    CallExpression(call) if call.optional => {
      let callee = call.callee; // should be MemberExpression
      let (base, _) = extract_base(callee);
      let lowered = lower_chain(base, ctx);
      // 构建 (base === null || base === void 0) ? void 0 : base.prop(args)
      Conditional(
        test: Binary(base, ===, null) || Binary(base, ===, void 0),
        consequent: void 0,
        alternate: CallExpression { callee: lowered, optional: false, args: call.args }
      )
    }
    MemberExpression(member) => {
      // 非 optional，继续递归
      MemberExpression {
        object: lower_chain(member.object, ctx),
        optional: false,
        ...member
      }
    }
    CallExpression(call) => {
      CallExpression {
        callee: lower_chain(call.callee, ctx),
        optional: false,
        ...call
      }
    }
    _ => expr
  }
}
```

## 边界情况

1. **delete 操作符**：`delete a?.b` → `(a === null || a === void 0) ? true : delete a.b`
2. **函数调用**：`a?.()` → `a === null || a === void 0 ? void 0 : a()`
3. **嵌套链**：`a?.b?.c` → 多层嵌套条件表达式
4. **调用链**：`a?.b()?.c` → `(a === null || a === void 0) ? void 0 : a.b()?.c` → 递归处理