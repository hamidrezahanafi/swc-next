import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const metrics = ["Ir", "Ct"];
const changeFormatter = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 3,
  useGrouping: false,
});

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
      metrics.every((metric) => events?.includes(metric)) &&
        totals !== undefined,
      `Incomplete profile: ${name}`,
    );
    const values = totals ? totals.split(/\s+/) : [];
    assert.ok(
      values.length <= events.length &&
        values.every((value) => /^\d+$/.test(value)),
      `Invalid totals: ${name}`,
    );
    const counts = {};
    for (const metric of metrics) {
      // Callgrind omits trailing zero counters.
      const value = Number(values[events.indexOf(metric)] ?? "0");
      assert.ok(
        Number.isSafeInteger(value) && value > 0,
        `Invalid ${metric}: ${name}`,
      );
      counts[metric] = value;
    }
    assert.ok(!results.has(name), `Duplicate benchmark: ${name}`);
    results.set(name, counts);
  }
  return results;
}

export async function loadProfiles(directory) {
  const results = new Map();
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith(".out")) continue;
    for (const [name, counts] of parseProfile(
      await readFile(join(directory, entry.name), "utf8"),
    )) {
      assert.ok(!results.has(name), `Duplicate benchmark: ${name}`);
      results.set(name, counts);
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
    "CodSpeed CPU Simulation · instruction counts (Ir) and throughput cycle estimates (Ct).",
    "Values are merge / base in K (1,000), truncated to whole thousands; Ct is converted from centi-cycles to cycles first. Changes use the unrounded counts, rounded to at most three decimal places. Lower is better: 🟢 improvement, 🔴 regression, ⚪ unchanged.",
    "Each row measures its bench.add() callback. Input preparation is excluded; Ct is a model estimate, not measured wall-clock time.",
    "",
    `Merge: \`${process.env.COMPARE_SHA ?? "local"}\`; base: \`${process.env.BASE_SHA ?? "local"}\`.`,
    "",
    "| Benchmark | Ir (K) | Change | Ct (K) | Change |",
    "| --- | ---: | ---: | ---: | ---: |",
  ];
  for (const name of [...base.keys()].sort()) {
    const before = base.get(name),
      after = merge.get(name);
    const label = name.replaceAll("|", "\\|").replaceAll("`", "'");
    const cells = [`\`${label}\``];
    for (const metric of metrics) {
      const divisor = metric === "Ct" ? 100_000 : 1_000;
      const pair = [after[metric], before[metric]]
        .map((value) => Math.trunc(value / divisor).toLocaleString("en-US"))
        .join(" / ");
      const delta = (after[metric] / before[metric] - 1) * 100;
      const magnitude = changeFormatter.format(Math.abs(delta));
      const change =
        magnitude === "0"
          ? "0% ⚪"
          : `${delta < 0 ? "−" : "+"}${magnitude}% ${delta < 0 ? "🟢" : "🔴"}`;
      cells.push(pair, change);
    }
    lines.push(`| ${cells.join(" | ")} |`);
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
