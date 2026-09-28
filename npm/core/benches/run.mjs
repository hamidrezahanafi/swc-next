import assert from "node:assert/strict";
import { writeFile } from "node:fs/promises";
import { join } from "node:path";

const files = JSON.parse(process.env.BENCHMARK_FILES ?? "[]");
assert.ok(Array.isArray(files) && files.length > 0, "Set BENCHMARK_FILES");
const directory = process.env.BENCHMARK_INPUT_DIR;
assert.ok(directory, "Set BENCHMARK_INPUT_DIR");

const names = [];
for (const file of files) {
  const { default: benchmarks } = await import(new URL(file, import.meta.url));
  names.push(...benchmarks);
}
assert.equal(new Set(names).size, names.length, "Duplicate benchmark names");
if (process.env.CODSPEED_ENV === undefined) {
  await writeFile(
    join(directory, "benchmarks.json"),
    JSON.stringify(names.sort(), null, 2) + "\n",
  );
}
