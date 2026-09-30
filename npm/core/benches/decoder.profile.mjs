#!/usr/bin/env node
// Separate allocation/GC cost from repeated field access on the same AST.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { Session } from "node:inspector";
import { cpus } from "node:os";
import path from "node:path";
import { performance, PerformanceObserver } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import { collectKeys, makeWalker } from "./decoder.bench.mjs";

const SCRIPT = fileURLToPath(import.meta.url);
const ROOT = path.resolve(path.dirname(SCRIPT), "../../..");
function runChild(config) {
  const args = ["--expose-gc", SCRIPT, "--worker", JSON.stringify(config)];
  const child = config.cpu === undefined
    ? spawnSync(process.execPath, args, { encoding: "utf8", maxBuffer: 8 * 1024 * 1024 })
    : spawnSync("taskset", ["-c", String(config.cpu), process.execPath, ...args], { encoding: "utf8", maxBuffer: 8 * 1024 * 1024 });
  if (child.status !== 0) throw new Error(child.stderr || String(child.error));
  return child.stdout;
}
const median = values => {
  const sorted = [...values].sort((a, b) => a - b), middle = sorted.length >> 1;
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

async function worker(config) {
  const { decode: eager, encodeOptions } = await import(pathToFileURL(path.join(ROOT, "packages/decoder/index.js")));
  const { decode: lazy } = await import(pathToFileURL(config.lazyModule ?? path.join(ROOT, "packages/decoder/lazy/index.js")));
  const { parseSync, langFromPath, sourceTypeFromPath } = await import(pathToFileURL(path.join(ROOT, "npm/core/src-js/binding.js")));
  const file = path.join(ROOT, "tasks/benchmark/files", config.file);
  const source = readFileSync(file, "utf8"), sourceBuffer = new TextEncoder().encode(source);
  const buffer = parseSync(sourceBuffer, source.length, encodeOptions({
    lang: langFromPath(file), sourceType: sourceTypeFromPath(file), preserveParens: true, comments: "none",
  }));
  let reference = eager(buffer, source, sourceBuffer).program;
  const keysByType = new Map();
  collectKeys(reference, keysByType);
  for (const [type, keys] of keysByType) keysByType.set(type, [...keys]);
  const walk = makeWalker(keysByType), expected = walk(reference);
  reference = undefined;
  const decode = config.variant === "eager" ? eager : lazy;
  let tree = decode(buffer, source, sourceBuffer).program;
  assert.equal(walk(tree), expected);
  tree = undefined;
  // Unwind the validation call stack before measuring retained heap.
  await new Promise(resolve => setImmediate(resolve));
  global.gc();
  if (config.scenario === "retained") {
    const baseline = process.memoryUsage().heapUsed;
    tree = decode(buffer, source, sourceBuffer).program;
    await new Promise(resolve => setImmediate(resolve));
    global.gc();
    const rootBytes = process.memoryUsage().heapUsed - baseline;
    assert.equal(walk(tree), expected);
    await new Promise(resolve => setImmediate(resolve));
    global.gc();
    const fullBytes = process.memoryUsage().heapUsed - baseline;
    // Use the tree after GC so its liveness does not depend on optimizer decisions.
    process.stdout.write(JSON.stringify({ rootBytes, fullBytes, checksum: walk(tree) }));
    return;
  }
  if (config.scenario === "cached-walk") {
    tree = decode(buffer, source, sourceBuffer).program;
    assert.equal(walk(tree), expected);
  }
  let checksum = 0;
  const run = config.scenario === "cached-walk" ? () => { checksum += walk(tree); }
    : config.scenario === "decode" ? () => { checksum += decode(buffer, source, sourceBuffer).program.type.length; }
    : () => { checksum += walk(decode(buffer, source, sourceBuffer).program); };
  let warmupMs = 0, warmupCycles = 0;
  while (warmupCycles < 5 || warmupMs < config.warmupMs) {
    const start = performance.now(); run(); warmupMs += performance.now() - start; warmupCycles++;
  }
  global.gc();
  const gcEntries = [];
  const observer = new PerformanceObserver(list => gcEntries.push(...list.getEntries()));
  observer.observe({ entryTypes: ["gc"] });
  let session;
  const post = (method, params = {}) => new Promise((resolve, reject) => session.post(method, params, (error, result) => error ? reject(error) : resolve(result)));
  if (config.profile) {
    session = new Session(); session.connect();
    await post("Profiler.enable");
    await post("Profiler.setSamplingInterval", { interval: 1000 });
    await post("Profiler.start");
  }
  const cyclesPerRound = Math.max(1, Math.min(100_000, Math.round(10 / Math.max(warmupMs / warmupCycles, 0.0001))));
  const samples = [], start = performance.now(), deadline = start + config.timeMs;
  while (samples.length < config.rounds || performance.now() < deadline) {
    const batchStart = performance.now();
    for (let i = 0; i < cyclesPerRound; i++) run();
    const batchEnd = performance.now();
    samples.push({ start: batchStart, end: batchEnd, ms: (batchEnd - batchStart) / cyclesPerRound });
  }
  const end = performance.now();
  if (config.profile) {
    const { profile } = await post("Profiler.stop");
    session.disconnect();
    writeFileSync(config.profile, JSON.stringify(profile));
  }
  await new Promise(resolve => setImmediate(resolve));
  await new Promise(resolve => setImmediate(resolve));
  observer.disconnect();
  const measuredGc = gcEntries.filter(entry => entry.startTime >= start && entry.startTime < end);
  const gcMs = measuredGc.reduce((sum, entry) => sum + entry.duration, 0);
  process.stdout.write(JSON.stringify({ bestMs: Math.min(...samples.map(sample => sample.ms)),
    medianMs: median(samples.map(sample => sample.ms)), elapsedMs: end - start,
    gcMs, gcPercent: gcMs / (end - start) * 100, gcCount: measuredGc.length,
    rounds: samples.length, cyclesPerRound, samples, checksum }));
}

if (process.argv[2] === "--worker") {
  await worker(JSON.parse(process.argv[3]));
} else {
  const options = { file: "typescript.js", output: path.join(ROOT, "target/decoder-benchmark/profile"),
    trials: 3, rounds: 20, warmupMs: 500, timeMs: 1000 };
  for (let i = 2; i < process.argv.length; i += 2) {
    const key = process.argv[i], value = process.argv[i + 1];
    if (value === undefined) throw new Error(`Missing value for ${key}`);
    if (key === "--file") options.file = value;
    else if (key === "--output") options.output = path.resolve(value);
    else if (key === "--lazy-module") options.lazyModule = path.resolve(value);
    else if (key === "--trials") options.trials = Number(value);
    else if (key === "--rounds") options.rounds = Number(value);
    else if (key === "--warmup-ms") options.warmupMs = Number(value);
    else if (key === "--time-ms") options.timeMs = Number(value);
    else if (key === "--cpu") options.cpu = Number(value);
    else throw new Error(`Unknown option: ${key}`);
  }
  assert.equal(path.basename(options.file), options.file);
  for (const key of ["trials", "rounds"]) assert.ok(Number.isInteger(options[key]) && options[key] > 0);
  for (const key of ["warmupMs", "timeMs"]) assert.ok(Number.isFinite(options[key]) && options[key] >= 0);
  if (options.cpu !== undefined) assert.ok(Number.isInteger(options.cpu) && options.cpu >= 0);
  mkdirSync(options.output, { recursive: true });
  const runs = [];
  for (let trial = 1; trial <= options.trials; trial++) {
    for (const scenario of ["decode", "full-walk", "cached-walk", "retained"]) {
      for (const variant of trial % 2 ? ["eager", "lazy"] : ["lazy", "eager"]) {
        const config = { ...options, scenario, variant };
        const result = JSON.parse(runChild(config));
        runs.push({ trial, scenario, variant, ...result });
        writeFileSync(path.join(options.output, "runs.json"), JSON.stringify(runs, null, 2));
        console.log(`${scenario} ${variant} trial ${trial}: ${scenario === "retained" ? (result.fullBytes / 2 ** 20).toFixed(1) + " MiB" : result.bestMs.toFixed(1) + " / " + result.medianMs.toFixed(1) + " ms; GC " + result.gcPercent.toFixed(1) + "%"}`);
      }
    }
  }
  for (const scenario of ["full-walk", "cached-walk"]) for (const variant of ["eager", "lazy"]) {
    const profile = path.join(options.output, `${variant}-${scenario}.cpuprofile`);
    const config = { ...options, variant, scenario, profile, rounds: 10, timeMs: 5000 };
    writeFileSync(path.join(options.output, `${variant}-${scenario}-profile-run.json`), runChild(config));
    console.log(`CPU profile saved: ${profile}`);
  }
  const summary = ["decode", "full-walk", "cached-walk", "retained"].map(scenario => ({ scenario,
    variants: ["eager", "lazy"].map(variant => {
      const subset = runs.filter(run => run.variant === variant && run.scenario === scenario);
      const metrics = scenario === "retained" ? ["rootBytes", "fullBytes"] : ["bestMs", "medianMs", "gcPercent"];
      return { variant, ...Object.fromEntries(metrics.map(key => [key, median(subset.map(run => run[key]))])) };
    }) }));
  const hashFiles = [path.join("tasks/benchmark/files", options.file),
    "packages/decoder/decode-js.js", "packages/decoder/decode-ts.js",
    "packages/decoder/lazy/decode-js.js", "packages/decoder/lazy/decode-ts.js", "packages/decoder/lazy/node-array.js"];
  const metadata = { ...options, node: process.version, cpuModel: cpus()[0].model,
    revision: spawnSync("git", ["rev-parse", "HEAD"], { cwd: ROOT, encoding: "utf8" }).stdout.trim(),
    hashes: Object.fromEntries(hashFiles.map(file => [file, createHash("sha256").update(readFileSync(path.join(ROOT, file))).digest("hex")])) };
  writeFileSync(path.join(options.output, "results.json"), JSON.stringify({ metadata, summary, runs }, null, 2));
  console.log(JSON.stringify(summary, null, 2));
}
