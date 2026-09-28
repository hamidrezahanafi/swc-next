import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

export function parseProfile(text) {
  const results = new Map();
  for (const part of text.split(/^part: /m).slice(1)) {
    const name = part.match(
      /^desc: Trigger: Client Request: .*::((?:encoder|decoder)\/[^\r\n]+)$/m,
    )?.[1];
    if (!name) continue; // Ignore runtime metadata and process termination.
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
  assert.equal(
    results.size,
    6,
    `Expected all six codec measurements in ${directory}`,
  );
  return results;
}

export function renderSummary(base, merge) {
  assert.deepEqual(
    [...merge.keys()].sort(),
    [...base.keys()].sort(),
    "Benchmark sets differ",
  );
  assert.ok(
    [...base.keys()].some((name) => name.startsWith("encoder/")) &&
      [...base.keys()].some((name) => name.startsWith("decoder/")),
    "Missing encoder or decoder measurements",
  );
  const lines = [
    "<!-- codec-benchmark-diff -->",
    "## JavaScript codec benchmark diff",
    "",
    "CodSpeed CPU Simulation · instruction counts (Ir) · lower is better.",
    "Encoder: plain AST → buffer. Decoder: precomputed parser buffer → fully traversed AST.",
    "Parsing, file I/O and input preparation are excluded. These counts are not wall-clock timings.",
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
    "Usage: render-codec-summary.mjs <base profiles> <merge profiles>",
  );
  process.stdout.write(
    renderSummary(await loadProfiles(base), await loadProfiles(merge)),
  );
}
