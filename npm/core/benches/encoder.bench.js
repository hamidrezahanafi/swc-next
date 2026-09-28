import { encode } from "@swc-next/encoder";

export { prepare } from "./utils.js";

export default function register(bench, cases) {
  for (const { name, program } of cases) {
    bench.add(name, () => encode(program));
  }
}
