import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { deserialize, serialize } from "node:v8";

const [mode, directory] = process.argv.slice(2);
assert.ok(
  ["prepare", "run"].includes(mode) && directory,
  "Usage: node .github/scripts/codec-bench.mjs <prepare|run> <input directory>",
);
const load = (path) => import(pathToFileURL(resolve(path)).href);
const { decode, encodeOptions } = await load("packages/decoder/index.js");
const { encode } = await load("packages/encoder/index.js");
const cases = ["jquery-1.9.1.js", "effect-schema-ast.ts", "unicode.ts"];

// Reading every enumerable field forces lazy nodes, arrays and strings. This
// traversal is deliberately part of the decoder measurement on both revisions.
function consume(value) {
  if (value === null || typeof value !== "object") return 0;
  let nodes = typeof value.type === "string" ? 1 : 0;
  for (const key of Object.keys(value)) nodes += consume(value[key]);
  return nodes;
}

if (mode === "prepare") {
  const { parseSync } = await load("npm/core/src-js/binding.js");
  await mkdir(directory, { recursive: true });
  for (const name of cases) {
    const source =
      name === "unicode.ts"
        ? Array.from(
            { length: 100 },
            (_, i) => `export const 文${i}: string = "😀 café \\u0061";`,
          ).join("\n")
        : await readFile(resolve("tasks/benchmark/files", name), "utf8");
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
    // Serialization materializes all getters; the benchmark deserializes a
    // plain AST, so encoding cannot accidentally include decoder work.
    const program = deserialize(serialize(result.program));
    assert.ok(consume(program) > 0, `${name}: empty AST`);
    await writeFile(
      resolve(directory, `${name}.bin`),
      serialize({
        source,
        sourceBytes,
        buffer,
        program,
      }),
    );
  }
} else {
  const require = createRequire(resolve("npm/core/package.json"));
  const { Bench } = await import(
    pathToFileURL(require.resolve("tinybench")).href
  );
  const { withCodSpeed } = await import(
    pathToFileURL(require.resolve("@codspeed/tinybench-plugin")).href
  );
  const bench = withCodSpeed(new Bench({ time: 100, warmupTime: 100 }));
  let encoded, decodedNodes;
  for (const name of cases) {
    const { source, sourceBytes, buffer, program } = deserialize(
      await readFile(resolve(directory, `${name}.bin`)),
    );
    bench.add(`encoder/${name}`, () => {
      encoded = encode(program);
    });
    bench.add(`decoder/${name}`, () => {
      decodedNodes = consume(decode(buffer, source, sourceBytes).program);
    });
  }
  await bench.run();
  assert.ok(encoded.byteLength > 0 && decodedNodes > 0);
}
