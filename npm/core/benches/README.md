# JavaScript benchmarks

After building the native binding, each benchmark is directly executable:

```sh
pnpm --filter @swc-next/core build --release
node npm/core/benches/encoder.bench.js
node npm/core/benches/decoder.bench.js
```

The package scripts `bench:codspeed:encoder` and `bench:codspeed:decoder` run
these same files. `pnpm --filter @swc-next/core bench:parser` runs the separate
parser comparison in `parser.bench.ts`.

## Adding benchmarks

The PR workflow explicitly selects files using `BENCHMARK_FILES` in
`.github/workflows/js-benchmarks.yml`:

```yaml
BENCHMARK_FILES: >-
  ["encoder.bench.js", "decoder.bench.js"]
```

To add a file, create a `.bench.js` or `.bench.ts` file in this directory and
append its relative path to that array. Use `runBenchmark` from `utils.js`:

```js
import { encodeOptions } from "@swc-next/encoder";
import { runBenchmark } from "./utils.js";

export default await runBenchmark(
  import.meta.url,
  () => ({ minify: true }),
  (bench, options) => {
    bench.add("minify options", () => encodeOptions(options));
  },
);
```

The second argument prepares inputs, which are passed to the registration
callback. Use `() => undefined` if no preparation is needed. Inputs must be
serializable with Node's V8 serializer, including buffers but excluding functions.
Keep setup in preparation and measured work inside `bench.add()` callbacks.

To extend an existing file, add a `bench.add()` call. Names are scoped by the
relative file path. TypeScript files use Node 24's native type stripping.

`encoder.bench.js` encodes fully materialized ASTs. `decoder.bench.js` decodes
precomputed native parser buffers and traverses all lazy AST fields. They share
JS, TS and Unicode inputs and the AST traversal helper from `utils.js`.

## Execution mode

`runBenchmark` selects its behavior from environment variables:

- No `BENCHMARK_INPUT_DIR` or `CODSPEED_ENV`: prepare inputs and run local Tinybench timings.
- `BENCHMARK_INPUT_DIR` set: prepare and save inputs without measuring.
- `CODSPEED_ENV` set by the CodSpeed runner: load saved inputs from
  `BENCHMARK_INPUT_DIR` and measure with CodSpeed. Native parsing is excluded.

`run.mjs` runs the files in `BENCHMARK_FILES` and writes the expected case list
when preparing inputs. To collect simulated instruction counts on Linux:

```sh
export BENCHMARK_FILES='["encoder.bench.js", "decoder.bench.js"]'
export BENCHMARK_INPUT_DIR=/tmp/js-bench-inputs
node npm/core/benches/run.mjs
mkdir -p /tmp/js-bench-results
cp "$BENCHMARK_INPUT_DIR/benchmarks.json" /tmp/js-bench-results/
CODSPEED_SKIP_UPLOAD=true codspeed run --mode simulation \
  --profile-folder /tmp/js-bench-results -- \
  node --no-maglev npm/core/benches/run.mjs
```

The PR workflow measures base and merge with the same benchmark files and
fixtures. It reports instruction counts, saves raw profiles, and verifies every
registered case against `benchmarks.json`. Changes are informational; build,
execution and missing-measurement errors fail the workflow.
