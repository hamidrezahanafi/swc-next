import { decodeJs } from "./decode-js.js";
import { decodeTs } from "./decode-ts.js";

export { decodeJs, decodeTs };
export { encodeOptions } from "../options.js";

/** Decode using field getters and arrays that construct nodes on access. */
export function decode(buffer, source, sourceBuffer) {
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const view = new DataView(arrayBuffer, buffer.byteOffset ?? 0, buffer.byteLength);
  return (view.getInt32(32, true) & 1) !== 0
    ? decodeTs(buffer, source, sourceBuffer)
    : decodeJs(buffer, source, sourceBuffer);
}
