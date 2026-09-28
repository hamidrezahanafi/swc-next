# JavaScript benchmarks

The PR workflow explicitly selects files using `BENCHMARK_FILES` in
`.github/workflows/js-benchmarks.yml`:

```yaml
BENCHMARK_FILES: >-
  ["encoder.bench.js", "decoder.bench.js"]
```

To add a benchmark file, create a `.bench.js` or `.bench.ts` file in this directory
and append its relative path to that array. To add a case to an existing file,
add a `bench.add()` call. The reporter needs no changes.

```js
import { encodeOptions } from "@swc-next/encoder";

export default function register(bench) {
  bench.add("minify options", () => encodeOptions({ minify: true }));
}
```

TypeScript files can use `import type { Bench } from "tinybench"` and annotate
the parameter as `bench: Bench`. Node 24 strips types; no transpiler is required.
Names are scoped by the relative file path, so two files may use the same case name.

## Preparing inputs

Export an optional `prepare()` function when setup is expensive or needs native
code. Its return value is passed as the registration function's second argument:

```js
export async function prepare() {
  return { values: [1, 2, 3] };
}

export default function register(bench, { values }) {
  bench.add("sum", () => values.reduce((sum, value) => sum + value, 0));
}
```

CI runs preparation before starting CodSpeed. Data is saved using Node's V8
serialization, so return serializable values (including buffers), not functions.
Registration runs during both preparation and measurement; keep setup inside
`prepare()` and measured work inside `bench.add()` callbacks.

`encoder.bench.js` encodes fully materialized ASTs. `decoder.bench.js` decodes
precomputed native parser buffers and traverses all lazy AST fields. They share
JS, TS and Unicode inputs from `utils.js`, which also provides the AST traversal
helper; parsing is preparation only.

## Running

After building the native binding:

```sh
pnpm --filter @swc-next/core build --release
pnpm --filter @swc-next/core bench encoder.bench.js decoder.bench.js
```

To collect simulated instruction counts on Linux, from the repository root:

```sh
export BENCHMARK_FILES='["encoder.bench.js", "decoder.bench.js"]'
node npm/core/benches/run.mjs prepare /tmp/js-bench-inputs
mkdir -p /tmp/js-bench-results
cp /tmp/js-bench-inputs/benchmarks.json /tmp/js-bench-results/
CODSPEED_SKIP_UPLOAD=true codspeed run --mode simulation \
  --profile-folder /tmp/js-bench-results -- \
  node --no-maglev npm/core/benches/run.mjs run /tmp/js-bench-inputs
```

The PR workflow measures base and merge with the same benchmark files and
fixtures. It reports instruction counts, saves raw profiles, and verifies every
registered case against `benchmarks.json`. Changes are informational; build,
execution and missing-measurement errors fail the workflow.

`parser.bench.ts` remains a separate, manually run comparison of several
parser implementations. Run it with `node npm/core/benches/parser.bench.ts`.
