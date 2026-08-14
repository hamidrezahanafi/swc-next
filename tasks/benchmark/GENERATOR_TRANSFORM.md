# Generator transform benchmark

This document records a local diagnostic comparison. Wall-clock data is useful
for checking scaling and phase balance, but it is not a CI Callgrind/Gungraun
merge gate.

## Revisions and environment

- SWC Next optimized implementation: based on
  `d181c8f2c17803515b6d3b7f75ce656f1e782efc`
- SWC Next baseline: clean detached `d181c8f2c17803515b6d3b7f75ce656f1e782efc`
- SWC comparison checkout: `5bf27fd72e4667bac6cc86888b8facb8b91f8077`
- Rust: `rustc 1.96.1 (31fca3adb 2026-06-26)`
- Machine: Linux x86_64 VM, 32 online vCPUs, Intel Xeon Platinum 8336C
- Profile: Cargo `bench` release profile, Criterion 100 samples

The generated input exercises `yield`, `yield*`, loops, `for-in`, calls,
`switch`, and `try/catch/finally`. Every run uses generator-only transform
options. A startup assertion verifies that generator syntax disappears,
`_ts_generator` is emitted, and an arrow-function sentinel remains unchanged.

Commands:

```sh
GENERATOR_BENCH_COUNT=1 cargo bench --locked -p benchmark \
  --bench generator_transform -- generator/1/ --noplot
GENERATOR_BENCH_COUNT=48 cargo bench --locked -p benchmark \
  --bench generator_transform -- generator/48/ --noplot
GENERATOR_BENCH_COUNT=192 cargo bench --locked -p benchmark \
  --bench generator_transform -- generator/192/ --noplot
```

## Execution flow

```text
TransformerOptions(generator=true)
  derives YIELD + GENERATOR_FUNCTIONS + FUNCTION_ENV_CAPTURE
       ↓
Semantic single traversal
  canonical names + scopes/symbols/references
  yield counts + generator post-order + captures
  used-name index
       ↓
Generator transform
  consumes semantic facts directly
  reserves generated names in mutable Semantic
  updates capture declaration/reference IDs
  clears yield counts along recorded ancestor paths
       ↓
No node-level transforms enabled
  TraverseAction::Skip; lowered tree is not traversed again
```

## Current phase results

The values below are Criterion point estimates.

| Count | Phase | SWC | SWC Next | SWC Next relative |
| ---: | --- | ---: | ---: | ---: |
| 1 | Parse | 13.255 us | 5.093 us | 61.6% faster |
| 1 | Resolver / generator semantic | 3.719 us | 7.984 us | 2.15x |
| 1 | Generator transform | 33.975 us | 15.806 us | 53.5% faster |
| 1 | Codegen | 31.069 us | 16.563 us | 46.7% faster |
| 1 | End to end | 104.98 us | 50.314 us | 52.1% faster |
| 48 | Parse | 589.13 us | 210.89 us | 64.2% faster |
| 48 | Resolver / generator semantic | 123.18 us | 297.53 us | 2.42x |
| 48 | Generator transform | 1.4041 ms | 663.82 us | 52.7% faster |
| 48 | Codegen | 1.2447 ms | 805.25 us | 35.3% faster |
| 48 | End to end | 3.9449 ms | 1.9777 ms | 49.9% faster |
| 192 | Parse | 2.5320 ms | 845.62 us | 66.6% faster |
| 192 | Resolver / generator semantic | 595.17 us | 1.2892 ms | 2.17x |
| 192 | Generator transform | 5.2221 ms | 2.8281 ms | 45.8% faster |
| 192 | Codegen | 5.2128 ms | 3.3432 ms | 35.9% faster |
| 192 | End to end | 16.058 ms | 8.2282 ms | 48.8% faster |

The resolver and semantic rows are not equivalent algorithms. SWC Next semantic
also builds the requested transform facts and incremental indexes.

Input/output sizes:

| Generators | Source | SWC output | SWC Next output | SWC Next nodes |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 758 B | 1,510 B | 1,569 B | 138 → 528 |
| 48 | 34,627 B | 71,052 B | 73,884 B | 6,013 → 24,733 |
| 192 | 138,635 B | 284,356 B | 295,684 B | 24,013 → 98,893 |

## Optional semantic feature cost

| Generators | features=empty | generator-only | async-only |
| ---: | ---: | ---: | ---: |
| 1 | 4.046 us | 7.984 us | 7.606 us |
| 48 | 147.09 us | 297.53 us | 270.67 us |
| 192 | 675.90 us | 1.2892 ms | 1.2274 ms |

The empty configuration does not allocate the transform-fact side tables.
Generator-only pays for yield counts, post-order generator/capture facts, and
the used-name index. Async-only uses the corresponding await/async facts.

## Same-machine before/after transform comparison

The baseline was measured from a temporary detached worktree at the base
revision with the same compiler, machine, input generator, release target, and
Criterion settings. Only the generator transform cases were run.

| Generators | Baseline | Optimized | Improvement | Baseline / generator | Optimized / generator |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 23.515 us | 15.806 us | 32.8% | 23.52 us | 15.81 us |
| 48 | 1.4285 ms | 663.82 us | 53.5% | 29.76 us | 13.83 us |
| 192 | 10.666 ms | 2.8281 ms | 73.5% | 55.55 us | 14.73 us |

From 48 to 192 generators, input size grows 4x:

- baseline transform grows 7.47x and its per-generator cost grows 86.7%;
- optimized transform grows 4.26x and its per-generator cost grows 6.5%.

This is the central acceptance result: removing per-generator full
symbol/reference scans eliminates the observed quadratic growth. The optimized
curve is close to linear over the requested 1/48/192 range.

## Transform framework breakdown

| Generators | No-op input | Generator enabled, no match | No-op after lowering |
| ---: | ---: | ---: | ---: |
| 1 | 62 ns | 67 ns | 100 ns |
| 48 | 124 ns | 92 ns | 150 ns |
| 192 | 302 ns | 267 ns | 182 ns |

These cases now measure dispatch-only overhead. When shorthand and sticky-regexp
passes are disabled, the transformer returns `TraverseAction::Skip`; it no
longer traverses the expanded lowered tree. Generator analysis is also absent:
yield queries, function order, captures, and used names come from Semantic.

Name reservation and yield-effect commit are included in the generator
transform timer. They are incremental operations, not independently timed
public passes: name lookup is indexed, and commit visits only recorded yield
ancestor paths. A future CI Callgrind harness can split their instruction
counts without adding wall-clock instrumentation to production code.

## Interpretation

- Output snapshots for the pinned Babel generator fixtures remain unchanged.
- Generator transform is faster than the SWC comparison at all three sizes on
  this machine.
- The single-generator case improves, so mutable Semantic and context updates
  did not introduce a fixed-cost regression.
- The feature-empty semantic case remains materially cheaper than requested
  generator/async facts, confirming that option-driven collection is active.
- Local timing can be noisy; instruction-count claims still require the
  repository Gungraun GitHub Actions workflow.
