import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { deserialize, serialize } from "node:v8";
import { withCodSpeed } from "@codspeed/tinybench-plugin";
import { decode, encodeOptions } from "@swc-next/decoder";
import { Bench } from "tinybench";

export async function runBenchmark(url, prepare, register) {
  const file = relative(import.meta.dirname, fileURLToPath(url)).replaceAll(
    "\\",
    "/",
  );
  const directory = process.env.BENCHMARK_INPUT_DIR;
  const simulation = process.env.CODSPEED_ENV !== undefined;
  assert.ok(!simulation || directory, "CodSpeed requires BENCHMARK_INPUT_DIR");
  const dataFile = directory && join(directory, `${file}.bin`);
  const saved = simulation ? deserialize(await readFile(dataFile)) : undefined;
  const data = simulation ? saved.data : await prepare();
  let bench = new Bench({
    name: file,
    time: 100,
    warmupTime: 100,
    throws: true,
  });
  if (simulation) bench = withCodSpeed(bench);
  await register(bench, data);
  assert.ok(bench.tasks.length > 0, `${file} registered no benchmarks`);
  const names = bench.tasks.map((task) => `${bench.name}::${task.name}`).sort();
  assert.equal(new Set(names).size, names.length, "Duplicate benchmark names");
  if (directory && !simulation) {
    await mkdir(dirname(dataFile), { recursive: true });
    await writeFile(dataFile, serialize({ data, names }));
    return names;
  }
  if (simulation) {
    assert.deepEqual(
      names,
      saved.names,
      "Registered benchmarks changed after preparation",
    );
  }
  await bench.run();
  if (!simulation) console.table(bench.table());
  return names;
}

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
