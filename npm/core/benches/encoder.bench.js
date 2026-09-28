import { encode } from "@swc-next/encoder";
import { prepare, runBenchmark } from "./utils.js";

export default await runBenchmark(import.meta.url, prepare, (bench, cases) => {
  for (const { name, program } of cases) {
    bench.add(name, () => encode(program));
  }
});
