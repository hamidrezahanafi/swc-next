# `@swc-next/core-browser`

The browser WebAssembly API for SWC Next. The package includes its WebAssembly
binary and browser JavaScript bindings.

```js
import init, { parseSync, printSync } from "@swc-next/core-browser";

await init();

const { program } = parseSync("const value = 1;");
program.body[0].declarations[0].init.value = 2;
console.log(printSync(program).code);
```

Call `init()` before parsing or printing. It loads the bundled `.wasm` file;
subsequent calls are idempotent.

`parse(source, options)` returns a `Promise<ParseResult>` with the same result as
`parseSync`. In this WebAssembly package, computation still runs synchronously
on the calling thread. The native `@swc-next/core` and `@swc-next/parser` packages
run `parse` on the worker pool; AST decoding happens lazily on the calling thread.
