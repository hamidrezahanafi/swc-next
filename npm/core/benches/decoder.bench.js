import { decode } from "@swc-next/decoder";
import { consume } from "./utils.js";

export { prepare } from "./utils.js";

export default function register(bench, cases) {
  for (const { name, source, sourceBytes, buffer } of cases) {
    bench.add(name, () => consume(decode(buffer, source, sourceBytes).program));
  }
}
