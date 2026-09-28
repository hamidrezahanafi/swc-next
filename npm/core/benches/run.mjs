import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { deserialize } from "node:v8";

const files = JSON.parse(process.env.BENCHMARK_FILES ?? "[]");
assert.ok(Array.isArray(files) && files.length > 0, "Set BENCHMARK_FILES");
const directory = process.env.BENCHMARK_INPUT_DIR;
assert.ok(directory, "Set BENCHMARK_INPUT_DIR");
const preparing = process.env.CODSPEED_ENV === undefined;

const names = [];
for (const file of files) {
  // Keep each file's heap and GC history independent. Inherit CodSpeed's V8 flags.
  const result = spawnSync(
    process.execPath,
    [...process.execArgv, fileURLToPath(new URL(file, import.meta.url))],
    { stdio: "inherit" },
  );
  if (result.error) throw result.error;
  assert.equal(
    result.status,
    0,
    `${file} failed (${result.signal ?? result.status})`,
  );
  if (preparing) {
    const saved = deserialize(await readFile(join(directory, `${file}.bin`)));
    names.push(...saved.names);
  }
}
if (preparing) {
  assert.equal(new Set(names).size, names.length, "Duplicate benchmark names");
  await writeFile(
    join(directory, "benchmarks.json"),
    JSON.stringify(names.sort(), null, 2) + "\n",
  );
}
