import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { deserialize, serialize } from "node:v8";
import { decode, encodeOptions } from "@swc-next/decoder";

// Force lazy nodes, arrays and strings. Traversal is part of the measurement.
export function consume(value) {
  if (value === null || typeof value !== "object") return 0;
  let nodes = typeof value.type === "string" ? 1 : 0;
  for (const key of Object.keys(value)) nodes += consume(value[key]);
  return nodes;
}

export async function prepare() {
  const { parseSync } = await import("../src-js/binding.js");
  const cases = [];
  for (const name of [
    "jquery-1.9.1.js",
    "effect-schema-ast.ts",
    "unicode.ts",
  ]) {
    const source =
      name === "unicode.ts"
        ? Array.from(
            { length: 100 },
            (_, i) => `export const 文${i}: string = "😀 café \\u0061";`,
          ).join("\n")
        : await readFile(
            new URL(`../../../tasks/benchmark/files/${name}`, import.meta.url),
            "utf8",
          );
    const sourceBytes = Buffer.from(source);
    const buffer = parseSync(
      sourceBytes,
      source.length,
      encodeOptions({
        lang: name.endsWith(".ts") ? "ts" : "js",
        sourceType: "unambiguous",
        comments: "none",
        preserveParens: true,
      }),
    );
    const result = decode(buffer, source, sourceBytes);
    assert.equal(result.diagnostics.length, 0, `${name}: parser diagnostics`);
    // Materialize all getters before encoding so its measurement excludes decoding.
    const program = deserialize(serialize(result.program));
    assert.equal(program.type, "Program", `${name}: missing AST`);
    cases.push({ name, source, sourceBytes, buffer, program });
  }
  return cases;
}
