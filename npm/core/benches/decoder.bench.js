import { decode } from "@swc-next/decoder";
import { consume, prepare, runBenchmark } from "./utils.js";

export default await runBenchmark(import.meta.url, prepare, (bench, cases) => {
  for (const { name, source, sourceBytes, buffer } of cases) {
    bench.add(name, () => consume(decode(buffer, source, sourceBytes).program));
  }
});
