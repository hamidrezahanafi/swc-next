#!/usr/bin/env node
// Compare eager and lazy decoder access patterns using identical pre-parsed buffers.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { cpus } from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";

const SCRIPT = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT), "../../..");
const SCENARIOS = ["root", "top-level", "full-walk"];

function parseArgs(args) {
  const options = {
    fixtures: path.join(ROOT, "tasks/benchmark/files"),
    output: path.join(ROOT, "target/decoder-benchmark"),
    trials: 3, timeMs: 500, warmupMs: 250, minRounds: 10, files: [],
  };
  for (let i = 0; i < args.length; i++) {
    const key = args[i], value = args[++i];
    if (value === undefined) throw new Error(`Missing value for ${key}`);
    if (key === "--file") options.files.push(value);
    else if (key === "--fixtures") options.fixtures = path.resolve(value);
    else if (key === "--output") options.output = path.resolve(value);
    else if (key === "--trials") options.trials = Number(value);
    else if (key === "--time-ms") options.timeMs = Number(value);
    else if (key === "--cpu") options.cpu = Number(value);
    else if (key === "--warmup-ms") options.warmupMs = Number(value);
    else if (key === "--min-rounds") options.minRounds = Number(value);
    else throw new Error(`Unknown option: ${key}`);
  }
  if (!Number.isInteger(options.trials) || options.trials < 1) throw new Error("Invalid trials");
  if (!Number.isFinite(options.timeMs) || options.timeMs <= 0) throw new Error("Invalid time-ms");
  if (!Number.isFinite(options.warmupMs) || options.warmupMs < 0) throw new Error("Invalid warmup-ms");
  if (!Number.isInteger(options.minRounds) || options.minRounds < 1) throw new Error("Invalid min-rounds");
  if (options.files.length === 0) {
    options.files = readdirSync(options.fixtures).filter(name => /\.(js|jsx|ts|tsx)$/.test(name)).sort();
  }
  for (const name of options.files) {
    if (path.basename(name) !== name) throw new Error(`Invalid file: ${name}`);
  }
  return options;
}

export function collectKeys(value, keysByType) {
  if (value === null || typeof value !== "object" || value instanceof RegExp) return;
  if (Array.isArray(value)) {
    for (const entry of value) collectKeys(entry, keysByType);
    return;
  }
  if (typeof value.type === "string") {
    const keys = keysByType.get(value.type) ?? new Set();
    for (const key of Object.keys(value)) keys.add(key);
    keysByType.set(value.type, keys);
  }
  for (const entry of Object.values(value)) collectKeys(entry, keysByType);
}

export function makeWalker(keysByType) {
  // Schema keys are learned once from eager output, outside the timer, then used
  // for both representations. Object.keys(lazyNode) would skip prototype getters.
  function walk(value) {
    if (value === null || value === undefined) return 0;
    switch (typeof value) {
      case "number": return Number.isFinite(value) ? value : 0;
      case "string": return value.length;
      case "boolean": return value ? 1 : 0;
      case "bigint": return value.toString().length;
    }
    if (value instanceof RegExp) return value.source.length + value.flags.length;
    let checksum = 1;
    if (Array.isArray(value)) {
      for (const entry of value) checksum += walk(entry);
    } else {
      const keys = keysByType.get(value.type) ?? Object.keys(value);
      for (const key of keys) checksum += walk(value[key]);
    }
    return checksum;
  }
  return walk;
}

function materialize(value) {
  if (value === null || typeof value !== "object" || value instanceof RegExp) return value;
  if (Array.isArray(value)) return Array.from(value, materialize);
  const plain = typeof value.toJSON === "function" ? value.toJSON() : value;
  return Object.fromEntries(Object.entries(plain).map(([key, entry]) => [key, materialize(entry)]));
}

async function worker(config) {
  const { decode: eager, encodeOptions } = await import(pathToFileURL(path.join(ROOT, "packages/decoder/index.js")));
  const { decode: lazy } = await import(pathToFileURL(path.join(ROOT, "packages/decoder/lazy/index.js")));
  const { parseSync, langFromPath, sourceTypeFromPath } = await import(pathToFileURL(path.join(ROOT, "npm/core/src-js/binding.js")));
  const decode = config.variant === "eager" ? eager : lazy;
  const results = [];
  let checksum = 0;
  for (const name of config.files) {
    const file = path.join(config.fixtures, name);
    const source = readFileSync(file, "utf8");
    const sourceBuffer = new TextEncoder().encode(source);
    const buffer = parseSync(sourceBuffer, source.length, encodeOptions({
      lang: langFromPath(file), sourceType: sourceTypeFromPath(file),
      preserveParens: true, comments: "none",
    }));
    let reference = eager(buffer, source, sourceBuffer).program;
    const keysByType = new Map();
    collectKeys(reference, keysByType);
    for (const [type, keys] of keysByType) keysByType.set(type, [...keys]);
    const walk = makeWalker(keysByType);
    const expected = walk(reference);
    let validation = decode(buffer, source, sourceBuffer);
    assert.equal(walk(validation.program), expected, `${name}: full-walk checksum`);
    // Fully compare the representations on one trial; never time this allocation.
    if (config.validate && config.variant === "lazy" && config.scenario === "full-walk") assert.deepEqual(materialize(validation.program), reference);
    const nodeCount = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength).getInt32(0, true);
    reference = undefined;
    validation = undefined;
    const read = config.scenario === "root" ? program => program.type.length
      : config.scenario === "top-level" ? program => {
        let sum = program.type.length;
        for (const node of program.body) sum += node.type.length + node.start + node.end;
        return sum;
      } : walk;
    const run = () => { checksum = (checksum + read(decode(buffer, source, sourceBuffer).program)) % 1_000_000_007; };
    let warmupMs = 0, warmupCycles = 0;
    while (warmupCycles < 5 || warmupMs < config.warmupMs) {
      const start = performance.now(); run(); warmupMs += performance.now() - start; warmupCycles++;
    }
    const cyclesPerRound = Math.max(1, Math.min(100_000, Math.round(10 / Math.max(warmupMs / warmupCycles, 0.0001))));
    if (global.gc) global.gc();
    const samples = [];
    const deadline = performance.now() + config.timeMs;
    while (samples.length < config.minRounds || performance.now() < deadline) {
      const start = performance.now();
      for (let i = 0; i < cyclesPerRound; i++) run();
      samples.push((performance.now() - start) / cyclesPerRound);
    }
    results.push({ name, bytes: sourceBuffer.length, nodeCount,
      bestMs: Math.min(...samples), medianRoundMs: median(samples),
      warmupCycles, warmupMs, rounds: samples.length, cyclesPerRound });
    process.stderr.write(`${config.variant} ${config.scenario} ${name}: ${results.at(-1).bestMs.toFixed(4)} ms\n`);
  }
  process.stdout.write(JSON.stringify({ results, checksum }));
}

function median(numbers) {
  const values = [...numbers].sort((a, b) => a - b), middle = Math.floor(values.length / 2);
  return values.length % 2 ? values[middle] : (values[middle - 1] + values[middle]) / 2;
}

function summarize(options, runs) {
  return SCENARIOS.map(scenario => ({ scenario, files: options.files.map(name => {
    const values = (variant, metric = "bestMs") => runs.filter(run => run.scenario === scenario && run.variant === variant)
      .map(run => run.results.find(result => result.name === name)[metric]);
    const eager = values("eager"), lazy = values("lazy");
    const details = runs[0].results.find(result => result.name === name);
    return { name, bytes: details.bytes, nodeCount: details.nodeCount,
      eagerMs: median(eager), lazyMs: median(lazy), ratio: median(lazy) / median(eager),
      eagerBatchMedianMs: median(values("eager", "medianRoundMs")),
      lazyBatchMedianMs: median(values("lazy", "medianRoundMs")),
      eagerRange: [Math.min(...eager), Math.max(...eager)],
      lazyRange: [Math.min(...lazy), Math.max(...lazy)] };
  }) }));
}

function report(metadata, summary) {
  let text = `# Eager / lazy decoder benchmark\n\nNode ${metadata.node}; ${metadata.cpu}; pinned CPU ${metadata.pinnedCpu ?? "none"}; revision ${metadata.revision} with current working-tree decoder changes.\n\n`;
  text += `${metadata.trials} fresh processes per variant and scenario, alternating variant order; at least 5 warmups and ${metadata.warmupMs} ms warmup per fixture, approximately 10 ms batches, at least ${metadata.minRounds} measured batches and ${metadata.timeMs} ms per fixture. Reported time is the median of each process's fastest batch. Trial ranges are included.\n\n`;
  text += "The same pre-parsed native buffer is reused within each fixture. Rust parsing, UTF-8 encoding, input reads, validation, and module loading are outside the timer. Comments are disabled. Buffer setup and creation of a fresh decode result are included each time. Full walk reads every AST field using the same schema keys and checksum for both variants; it does not convert lazy nodes into plain objects.\n\n";
  text += "Scenarios: root reads only Program.type; top-level reads type/start/end of every Program.body entry; full-walk reads all fields recursively. Lower times are better; ratio is lazy / eager.\n\n";
  for (const { scenario, files } of summary) {
    text += `## ${scenario}\n\n| Fixture | Bytes | Raw nodes | Eager ms | Lazy ms | Lazy/eager | Eager trial range ms | Lazy trial range ms |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n`;
    const range = values => values.map(value => value.toFixed(4)).join("–");
    for (const file of files) text += `| ${file.name} | ${file.bytes} | ${file.nodeCount} | ${file.eagerMs.toFixed(4)} | ${file.lazyMs.toFixed(4)} | ${(file.ratio < 0.01 ? file.ratio.toExponential(2) : file.ratio.toFixed(2))}× | ${range(file.eagerRange)} | ${range(file.lazyRange)} |\n`;
    text += "\n";
    if (scenario === "full-walk") {
      text += "### Median of all measured batches\n\nThis includes measured GC and scheduling pauses instead of selecting each process's fastest batch. Values are medians across fresh processes.\n\n| Fixture | Eager ms | Lazy ms | Lazy/eager |\n| --- | ---: | ---: | ---: |\n";
      for (const file of files) text += `| ${file.name} | ${file.eagerBatchMedianMs.toFixed(4)} | ${file.lazyBatchMedianMs.toFixed(4)} | ${(file.lazyBatchMedianMs / file.eagerBatchMedianMs).toFixed(2)}× |\n`;
      text += "\n";
    }
  }
  return text;
}

const isMain = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (isMain && process.argv[2] === "--worker") {
  await worker(JSON.parse(process.argv[3]));
} else if (isMain) {
  const options = parseArgs(process.argv.slice(2));
  mkdirSync(options.output, { recursive: true });
  const runs = [];
  for (let trial = 1; trial <= options.trials; trial++) {
    const variants = trial % 2 ? ["eager", "lazy"] : ["lazy", "eager"];
    for (const scenario of SCENARIOS) for (const variant of variants) {
      console.log(`${scenario}: ${variant} trial ${trial}/${options.trials}`);
      const config = { ...options, scenario, variant, validate: trial === 1 };
      const args = ["--expose-gc", SCRIPT, "--worker", JSON.stringify(config)];
      const child = options.cpu === undefined
        ? spawnSync(process.execPath, args, { encoding: "utf8", maxBuffer: 8 * 1024 * 1024 })
        : spawnSync("taskset", ["-c", String(options.cpu), process.execPath, ...args], { encoding: "utf8", maxBuffer: 8 * 1024 * 1024 });
      if (child.status !== 0) throw new Error(child.stderr || String(child.error));
      const result = JSON.parse(child.stdout);
      assert.equal(Number.isFinite(result.checksum), true);
      runs.push({ trial, scenario, variant, ...result });
      writeFileSync(path.join(options.output, "runs.json"), JSON.stringify(runs, null, 2));
      console.log(child.stderr.trim().split("\n").at(-1));
    }
  }
  const revision = spawnSync("git", ["rev-parse", "HEAD"], { cwd: ROOT, encoding: "utf8" }).stdout.trim();
  const metadata = { node: process.version, cpu: cpus()[0].model, revision,
    trials: options.trials, timeMs: options.timeMs, warmupMs: options.warmupMs, minRounds: options.minRounds, pinnedCpu: options.cpu,
    fixtureHashes: Object.fromEntries(options.files.map(name => [name, createHash("sha256").update(readFileSync(path.join(options.fixtures, name))).digest("hex")])) };
  const summary = summarize(options, runs);
  writeFileSync(path.join(options.output, "results.json"), JSON.stringify({ metadata, summary, runs }, null, 2));
  const markdown = report(metadata, summary);
  writeFileSync(path.join(options.output, "report.md"), markdown);
  console.log(markdown);
}
