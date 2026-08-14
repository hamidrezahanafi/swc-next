# Generator transform benchmark

This is a local diagnostic comparison. It is not a CI Callgrind/Gungraun
result, so the wall-clock numbers should not be used as a merge gate.

## Revisions and environment

- SWC Next: `8671704472d86eb898e0384327972d304575cbfb`
- SWC: `5bf27fd72e4667bac6cc86888b8facb8b91f8077`, from a clean detached worktree
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`, release benchmark profile
- Machine: Apple M5 Pro, 18 cores, 48 GB RAM, arm64 macOS

The benchmark uses a generated script containing 48 generator functions by
default. Each function exercises `yield`, `yield*`, loops, `for-in`, calls,
`switch`, and `try/catch/finally`.

```sh
GENERATOR_BENCH_COUNT=48 \
  cargo bench -p benchmark --bench generator_transform -- generator/48/
```

`GENERATOR_BENCH_COUNT` is part of the Criterion benchmark ID so results from
different corpus sizes are not compared as the same case.

## Measurement boundaries

The benchmark reports these phases separately:

1. parse;
2. resolver for SWC or semantic analysis for SWC Next;
3. generator-only transform;
4. minified codegen;
5. parse + prerequisite + generator-only transform + minified codegen.

Resolver/semantic work is outside the transform timer. The transform timer
calls only SWC's `generator` pass or SWC Next with
`ES2015Options.generator = true`; every other compatibility transform remains
disabled. A startup assertion checks that generator syntax is removed, the
`_ts_generator` call is present, and an arrow-function sentinel is unchanged.

SWC receives a fresh `Globals` and `Helpers` context per compilation. Their
construction is outside the generator-only timer. Both code generators use
minified output; comparing their default pretty-printers is not meaningful
because their line-breaking policies differ.

"End to end" below means only those five measured stages, not a complete ES5
pipeline. In particular, SWC's later hygiene/fixer passes are intentionally not
run. SWC's generator pass can therefore leave textually equal private names
distinguished only by syntax context; the benchmark checks structural lowering
invariants, not byte-for-byte or directly executable output equivalence.

## Default 48-generator result

Input: 34,627 bytes. SWC Next expands from 6,013 to 24,733 flat AST nodes.
Minified output is 71,052 bytes for SWC and 73,884 bytes for SWC Next.

| Phase | SWC median | SWC Next median | SWC Next relative |
| --- | ---: | ---: | ---: |
| Parse | 212.45 us | 73.09 us | 65.6% faster |
| Resolver / semantic | 59.81 us | 41.08 us | 31.3% faster |
| Generator-only transform | 507.16 us | 631.13 us | 24.4% slower |
| Minified codegen | 431.21 us | 365.19 us | 15.3% faster |
| End to end | 1.3632 ms | 1.1171 ms | 18.1% faster |

Although generator transform alone is slower, the measured pipeline is faster
because parse, semantic analysis, and codegen save more time than the transform
loses.

## SWC Next transform breakdown

The 48-generator diagnostic cases measured:

| Component | Median |
| --- | ---: |
| Framework traversal over input with no transforms | 12.33 us |
| Generator enabled but no generator matches | 24.35 us |
| Framework traversal over the lowered tree | 61.46 us |

The generator analysis visitor itself is approximately the difference between
the first two rows, about 12 us. It is not the main cost.

The last row is avoidable in a generator-only run. Generator lowering happens
inside `enter_program`, after which `TraverseAction::Proceed` walks the newly
expanded 24,733-node tree even though shorthand and sticky-regexp transforms
are disabled. This accounts for about 9.7% of SWC Next's transform time.

## Scaling

Each implementation was run separately to prevent one benchmark's untimed
setup from affecting the next benchmark.

| Generator count | SWC | SWC Next | SWC Next relative | SWC / generator | Next / generator |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 10.72 us | 10.87 us | 1.4% slower | 10.72 us | 10.87 us |
| 48 | 507.16 us | 631.13 us | 24.4% slower | 10.57 us | 13.15 us |
| 192 | 2.0245 ms | 4.5569 ms | 125.1% slower | 10.54 us | 23.73 us |

SWC stays almost linear. SWC Next's per-generator time rises with the total
number of generators.

The cross-generator nonlinear work is the current `NameAllocator`:

- a new allocator is created for every generator scope;
- the first `_state` and `_tmp` allocation in each generator calls
  `scan_existing`;
- `scan_existing` iterates every semantic symbol and reference in the whole
  program and repeatedly walks scope ancestors;
- the per-base suffix map only caches within that one generator allocator.

For this corpus, both the number of generator scopes and the size of semantic
data grow together. The resulting work is approximately
`generators * (symbols + references)`, i.e. quadratic for this scaling test.

SWC does less work inside the isolated generator pass for private names: it
creates identifiers with distinct syntax contexts and leaves textual collision
resolution to the later hygiene pass. SWC Next currently performs the scope
scan and textual suffix allocation inside generator lowering because its
codegen does not yet consume hygiene contexts. This gives SWC an advantage in
the deliberately generator-only timer.

## Other structural differences

- SWC Next collects yield facts once with an independent immutable visitor.
  SWC calls `contains_yield` at 24 sites and recursively scans relevant
  subtrees. The single-generator result shows that the independent visitor is
  not inherently slower.
- SWC lowers a function after visiting its original children, so its wrapper
  does not traverse the newly generated state machine again. SWC Next's outer
  transformer currently does.
- The emitted state machines are not identical. For the one-generator fixture,
  SWC Next emits about 3.9% more minified bytes. More generated nodes increase
  lowering and post-transform traversal work, although SWC Next codegen is
  still faster in this benchmark.

## Recommended optimization order

1. Build a semantic used-name index once per analysis phase, keyed by hoist
   scope and canonical pooled name, then keep only `base -> next_suffix` mutable
   state in each allocator. Do not scan all symbols/references per generator.
2. Let the transformer skip traversal after whole-program lowering when no
   node-level ES2015 transforms are enabled.
3. Re-profile the state-machine compiler after removing those two framework
   costs; only then compare case/temp generation strategies with SWC.
