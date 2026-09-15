# @swc-next/ast

Generated TypeScript definitions for the ESTree AST returned by `@swc-next/core`
and `@swc-next/parser`.

```ts
import { parseSync } from "@swc-next/parser";
import type { Node, Program } from "@swc-next/ast";

const program: Program = parseSync("const answer = 42;").program;
const node: Node = program.body[0]!;
if (node.type === "VariableDeclaration") {
  console.log(node.declarations[0]!.id);
}
```

`@swc-next/core` also re-exports these types through the `ast` namespace:

```ts
import { parseSync, type ast } from "@swc-next/core";

const program: ast.Program = parseSync("const answer = 42;").program;
```

Packages whose published declarations reference `@swc-next/ast` include it in
`dependencies`, so their types resolve when installed independently.

This package contains types only. Import them with `import type`. The root
entry point exports the generated AST types and the shared `Comment` type. The
`@swc-next/ast/estree` subpath exports only the generated AST node types.

The definitions follow the [ESTree specification](https://github.com/estree/estree)
and include the TypeScript, JSX, and proposal nodes currently emitted by SWC
Next. They describe the actual decoder output: `start` and `end` are UTF-16
offsets, and TypeScript-only fields are optional because they are absent in
JavaScript mode. `Node` is a discriminated union; `NodeMap` maps each `type` to
its node type. Literals share `type: "Literal"`, with separate interfaces for
their value types.

Regenerate `estree.d.ts` from the repository root with `cargo run -p ast_tools`.
Edit the Rust AST schema or the generator, not the generated declaration file.

The shared `Comment` type lives in `index.d.ts`, which is maintained by hand.
