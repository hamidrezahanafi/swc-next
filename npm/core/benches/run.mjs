import assert from "node:assert/strict";
import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { deserialize, serialize } from "node:v8";
import { withCodSpeed } from "@codspeed/tinybench-plugin";
import { Bench } from "tinybench";

const [mode = "local", inputDirectory] = process.argv.slice(2);
assert.ok(
  ["local", "prepare", "run"].includes(mode),
  "Expected local, prepare or run",
);
assert.ok(
  mode === "local" || inputDirectory,
  "prepare/run requires an input directory",
);
const files = (await readdir(import.meta.dirname, { recursive: true }))
  .filter((file) => /\.bench\.(js|ts)$/.test(file))
  .sort();
assert.ok(files.length > 0, "No .bench.js or .bench.ts files found");
const directory =
  inputDirectory ?? (await mkdtemp(join(tmpdir(), "swc-next-bench-")));

async function execute(phase) {
  const names = [];
  for (const file of files) {
    const module = await import(
      pathToFileURL(join(import.meta.dirname, file)).href
    );
    assert.equal(
      typeof module.default,
      "function",
      `${file} must export a default registration function`,
    );
    const dataFile = join(directory, `${file}.bin`);
    const data =
      phase === "prepare"
        ? await module.prepare?.()
        : deserialize(await readFile(dataFile));
    if (phase === "prepare") {
      await mkdir(dirname(dataFile), { recursive: true });
      await writeFile(dataFile, serialize(data));
    }
    let bench = new Bench({
      name: file.replaceAll("\\", "/"),
      time: 100,
      warmupTime: 100,
      throws: true,
    });
    if (phase === "run") bench = withCodSpeed(bench);
    await module.default(bench, data);
    assert.ok(bench.tasks.length > 0, `${file} registered no benchmarks`);
    names.push(...bench.tasks.map((task) => `${bench.name}::${task.name}`));
    if (phase === "run") {
      await bench.run();
      if (!process.env.CODSPEED_ENV) {
        console.log(file);
        console.table(bench.table());
      }
    }
  }
  assert.equal(new Set(names).size, names.length, "Duplicate benchmark names");
  const manifest = join(directory, "benchmarks.json");
  if (phase === "prepare") {
    await writeFile(manifest, JSON.stringify(names.sort(), null, 2) + "\n");
  } else {
    assert.deepEqual(
      names.sort(),
      JSON.parse(await readFile(manifest, "utf8")),
      "Registered benchmarks changed after preparation",
    );
  }
}

try {
  if (mode === "local") {
    await execute("prepare");
    await execute("run");
  } else {
    await execute(mode);
  }
} finally {
  if (!inputDirectory) await rm(directory, { recursive: true, force: true });
}
