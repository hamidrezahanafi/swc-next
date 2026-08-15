# Async to Generator Transform

## Overview

Transforms `async` functions and arrow functions into generator-based wrappers using
the `_asyncToGenerator` helper. This is part of the ES2017 transform set.

## Transform Rules

### Async Function

Input:
```js
async function foo(a, b) {
    let x = await bar();
    return x;
}
```

Output:
```js
var _asyncToGenerator = /* ... */;
function foo(a, b) {
    return _asyncToGenerator(function*() {
        let x = yield bar();
        return x;
    })();
}
```

### Async Arrow Function

Input:
```js
const foo = async () => {
    let x = await bar();
    return x;
};
```

Output:
```js
var _asyncToGenerator = /* ... */;
const foo = _asyncToGenerator(function*() {
    let x = yield bar();
    return x;
});
```

## Key Transformations

### `await` -> `yield`
Every `await` expression in an async function is replaced with `yield`.

### `this` / `arguments` / `super` handling
- `this` in async functions: the wrapper calls `.call(this)` to preserve `this` binding.
- `arguments` in async functions: the wrapper calls `.apply(this, arguments)`.
- `super` in async functions: use `FnEnvHoister` to hoist `super` references.
- `this` in async arrows within subclass constructors: replace `this` with `_this` variable.

### `for-await-of` -> try/catch with generator
`for await (const x of iter)` is rewritten to a complex try/catch pattern using
`_asyncIterator` helper and `yield` in a for loop.

## Implementation Strategy

### State Tracking (Traverse hooks)

- `enter_function` / `exit_function`: Track async function state via `FnState` stack.
- `enter_arrow_function_expression` / `exit_arrow_function_expression`: Track async arrows.
- `enter_class` / `exit_class`: Track subclass context for `_this` rewriting.
- `exit_expression`: Detect `this`, `arguments`, `await`, `super`, `yield*` usage.

### Tree Rewriting (VisitMut)

- `visit_mut_expr`: Replace `await` with `yield`; replace `yield*` with delegate pattern.
- `visit_mut_stmt`: Handle `for-await-of` transformation.
- `visit_mut_this_expression`: Replace `this` with `_this` in constructor arrows.

### Helper Functions

Helpers are emitted as top-level `var` declarations at the program level:
- `_asyncToGenerator`: Core wrapper for async functions.
- `_awaitAsyncGenerator`: Used for `await` in async generators.
- `_asyncIterator`: Used for `for-await-of` iteration.
- `_asyncGeneratorDelegate`: Used for `yield*` in async generators.
- `_wrapAsyncGenerator`: Used for async generator functions.

## File Structure

```
crates/swc_ecma_transformer/
  src/
    es2017/
      mod.rs          -- ES2017 module, exports AsyncToGenerator
      async_to_generator.rs  -- Main transform implementation
```

## Dependencies

- `FnEnvHoister` from `es2015/util/fn_env_hoister.rs` for `super` hoisting.
- `Traverse` trait hooks for enter/exit callbacks.
- `VisitMut` trait for recursive AST rewriting.