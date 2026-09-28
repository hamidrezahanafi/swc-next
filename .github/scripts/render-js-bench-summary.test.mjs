import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  loadProfiles,
  parseProfile,
  renderSummary,
} from "./render-js-bench-summary.mjs";

const profile = (name, events, totals) => `part: 1
desc: Trigger: Client Request: npm/core/benches/example.bench.js::${name}
events: ${events}
summary: 999999
totals: ${totals}
`;

test("accepts newly registered cases and detects missing measurements", async (t) => {
  const directory = await mkdtemp(join(tmpdir(), "swc-bench-profile-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const name = "custom/new.bench.ts::options";
  await writeFile(join(directory, "benchmarks.json"), JSON.stringify([name]));
  await writeFile(join(directory, "1.out"), profile(name, "Ir", "123"));
  assert.deepEqual([...(await loadProfiles(directory))], [[name, 123]]);
  await writeFile(
    join(directory, "benchmarks.json"),
    JSON.stringify([name, "another.bench.js::case"]),
  );
  await assert.rejects(
    loadProfiles(directory),
    /Missing or unexpected measurements/,
  );
});

test("reads combined benchmark dumps, not metadata or summary counters", () => {
  const text =
    profile("encoder/jquery-1.9.1.js", "Ir Dr Dw", "120") +
    "part: 2\ndesc: Trigger: Client Request: Metadata: runtime::version\nevents: Ir\ntotals: 900\n" +
    profile("decoder/jquery-1.9.1.js", "Dr Ir Dw", "10 240") +
    "part: 4\ndesc: Trigger: Program termination\nevents: Ir\ntotals: 0\n";
  assert.deepEqual(
    [...parseProfile(text)],
    [
      ["encoder/jquery-1.9.1.js", 120],
      ["decoder/jquery-1.9.1.js", 240],
    ],
  );
});

test("rejects partial, invalid and duplicate benchmark data", () => {
  const valid = profile("encoder/a", "Ir", "123");
  assert.throws(
    () => parseProfile(valid.replace("totals: 123", "")),
    /Incomplete profile/,
  );
  assert.throws(
    () => parseProfile(profile("encoder/a", "Dr", "123")),
    /Incomplete profile/,
  );
  assert.throws(
    () => parseProfile(profile("encoder/a", "Ir", "NaN")),
    /Invalid totals/,
  );
  assert.throws(
    () => parseProfile(profile("encoder/a", "Ir", "9007199254740992")),
    /Invalid Ir/,
  );
  assert.throws(
    () => parseProfile(profile("encoder/a", "Ir", "0")),
    /Invalid Ir/,
  );
  assert.throws(() => parseProfile(valid + valid), /Duplicate benchmark/);
});

test("compares matching benchmark sets and reports the direction of change", () => {
  const base = new Map([
    ["encoder/a", 100],
    ["decoder/a", 200],
  ]);
  const merge = new Map([
    ["encoder/a", 110],
    ["decoder/a", 180],
  ]);
  const summary = renderSummary(base, merge);
  assert.match(summary, /encoder\/a.*110.*100.*\+10\.000%/);
  assert.match(summary, /decoder\/a.*180.*200.*-10\.000%/);
  assert.throws(
    () => renderSummary(base, new Map([["encoder/a", 110]])),
    /Benchmark sets differ/,
  );
  assert.throws(
    () => renderSummary(new Map(), new Map()),
    /No benchmark measurements/,
  );
});
