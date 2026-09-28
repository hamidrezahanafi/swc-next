import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

export function parseProfile(text) {
  const results = new Map();
  for (const part of text.split(/^part: /m).slice(1)) {
    const trigger = part.match(
      /^desc: Trigger: Client Request: ([^\r\n]+)$/m,
    )?.[1];
    if (!trigger || trigger.startsWith("Metadata:")) continue;
    // The plugin prefixes the URI with the file that called bench.add(). The
    // remainder is our Bench.name (the benchmark file) and the task name.
    const separator = trigger.indexOf("::");
    if (separator === -1) continue;
    const name = trigger.slice(separator + 2);
    const events = part
      .match(/^events:\s*(.+)$/m)?.[1]
      .trim()
      .split(/\s+/);
    const totals = part.match(/^totals:([^\r\n]*)$/m)?.[1].trim();
    assert.ok(
      events?.includes("Ir") && totals !== undefined,
      `Incomplete profile: ${name}`,
    );
    const values = totals ? totals.split(/\s+/) : [];
    assert.ok(
      values.length <= events.length &&
        values.every((value) => /^\d+$/.test(value)),
      `Invalid totals: ${name}`,
    );
    // Callgrind omits trailing zero counters.
    const instructions = Number(values[events.indexOf("Ir")] ?? "0");
    assert.ok(
      Number.isSafeInteger(instructions) && instructions > 0,
      `Invalid Ir: ${name}`,
    );
    assert.ok(!results.has(name), `Duplicate benchmark: ${name}`);
    results.set(name, instructions);
  }
  return results;
}

export async function loadProfiles(directory) {
  const results = new Map();
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith(".out")) continue;
    for (const [name, count] of parseProfile(
      await readFile(join(directory, entry.name), "utf8"),
    )) {
      assert.ok(!results.has(name), `Duplicate benchmark: ${name}`);
      results.set(name, count);
    }
  }
  const expected = JSON.parse(
    await readFile(join(directory, "benchmarks.json"), "utf8"),
  );
  assert.ok(
    Array.isArray(expected) &&
      expected.length > 0 &&
      expected.every((name) => typeof name === "string"),
    "Invalid benchmark manifest",
  );
  assert.deepEqual(
    [...results.keys()].sort(),
    expected.toSorted(),
    `Missing or unexpected measurements in ${directory}`,
  );
  return results;
}

export function renderSummary(base, merge) {
  assert.deepEqual(
    [...merge.keys()].sort(),
    [...base.keys()].sort(),
    "Benchmark sets differ",
  );
  assert.ok(base.size > 0, "No benchmark measurements");
  const lines = [
    "<!-- js-benchmark-diff -->",
    "## JavaScript benchmark diff",
    "",
    "CodSpeed CPU Simulation · instruction counts (Ir) · lower is better.",
    "Each row measures its bench.add() callback. Input preparation is excluded; counts are not wall-clock timings.",
    "",
    `Merge: \`${process.env.COMPARE_SHA ?? "local"}\`; base: \`${process.env.BASE_SHA ?? "local"}\`.`,
    "",
    "| Benchmark | Merge Ir | Base Ir | Change |",
    "| --- | ---: | ---: | ---: |",
  ];
  for (const name of [...base.keys()].sort()) {
    const before = base.get(name),
      after = merge.get(name);
    const delta = (after / before - 1) * 100;
    const label = name.replaceAll("|", "\\|").replaceAll("`", "'");
    lines.push(
      `| \`${label}\` | ${after.toLocaleString("en-US")} | ${before.toLocaleString("en-US")} | ${delta > 0 ? "+" : ""}${delta.toFixed(3)}% |`,
    );
  }
  return lines.join("\n") + "\n";
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const [base, merge] = process.argv.slice(2);
  assert.ok(
    base && merge,
    "Usage: render-js-bench-summary.mjs <base profiles> <merge profiles>",
  );
  process.stdout.write(
    renderSummary(await loadProfiles(base), await loadProfiles(merge)),
  );
}
