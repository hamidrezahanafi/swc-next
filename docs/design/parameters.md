# Parameters Transform Design

## 目标

```js
// Input: default parameters
function foo(a = 1) {}

// Output
function foo(a) {
  if (a === void 0) { a = 1; }
}
```

```js
// Input: rest parameters
function foo(a, ...rest) {}

// Output
function foo(a) {
  for (var _len = arguments.length, rest = new Array(_len - 1), _key = 0; _key < _len - 1; _key++) {
    rest[_key] = arguments[_key + 1];
  }
}
```

```js
// Input: destructured parameters
function foo({a, b}) {}

// Output
function foo(_ref) {
  var a = _ref.a, b = _ref.b;
}
```

## Semantic 依赖

- `FUNCTION_ENV_CAPTURE` — 函数环境捕获（`arguments` 对象）
- `PARAMETER_DEFAULTS` — 需要新增，标记有默认值的参数

## 需要新增的 SemanticFeatureFlags

- `PARAMETER_DEFAULTS` — 预收集需要转换的参数

## AST 操作分类

1. **默认参数 → function body 检查**：在函数体开头插入 `if (a === void 0) a = defaultValue;`
2. **Rest 参数 → arguments 循环**：在函数体开头插入 `arguments` 复制循环
3. **解构参数 → 临时变量**：`{a, b}` → `_ref` + 解构声明
4. **参数长度**：`Function.length` 在转换后需要调整（通过 `_len` 变量）

## Scope 操作

- 默认参数有自己的 scope（参数默认值表达式与函数体不同 scope，但 SWC 转换后合并到函数体）
- 需要新建变量声明在函数体 scope 中

## Helper 依赖

- 无（标准模式直接使用 `arguments` 对象）
- 可选：`_initializerWarningHelper`（`Object.is` 检测）

## 边界情况

1. **`arguments` 在默认值中**：`function(a = arguments)` → 指向参数对象
2. **默认值引用其他参数**：`function(a = 1, b = a)` → 保持顺序
3. **默认值中的 `this`**：`function(a = this)` → 指向调用上下文
4. **默认值中的 TDZ**：`function(a = a)` → 运行时错误
5. **Rest 参数后的 `arguments`**：rest 参数不包含在 `arguments` 中
6. **`new.target` 在默认值中**：`function(a = new.target)` → 需要 `FUNCTION_ENV_CAPTURE`
7. **`eval` 在默认值中**：`function(a = eval('1'))` → 需要特殊处理
8. **参数解构 + 默认值**：`function({a = 1})` → 组合转换

## Babel 兼容性

Babel 的 `@babel/plugin-transform-parameters` 处理所有三种情况。`loose` 模式下跳过 TDZ 检查。

## 实现策略

1. 新增 `PARAMETER_DEFAULTS` semantic feature 标记需要处理的函数
2. 在 `enter_function` 中检查参数是否有默认值、rest、或解构
3. 对于每个需要转换的函数：
   - 处理默认参数：在函数体开头插入 `if` 语句
   - 处理 rest 参数：在函数体开头插入 `arguments` 复制循环
   - 处理解构参数：用 `_ref` 替换参数，在函数体开头插入解构声明