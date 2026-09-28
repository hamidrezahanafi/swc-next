import { decode } from "@swc-next/decoder";

export { prepare } from "./fixtures.mjs";

// Force lazy nodes, arrays and strings. Traversal is part of the measurement.
function consume(value) {
  if (value === null || typeof value !== "object") return 0;
  let nodes = typeof value.type === "string" ? 1 : 0;
  for (const key of Object.keys(value)) nodes += consume(value[key]);
  return nodes;
}

export default function register(bench, cases) {
  for (const { name, source, sourceBytes, buffer } of cases) {
    bench.add(name, () => consume(decode(buffer, source, sourceBytes).program));
  }
}
