import { decodeJs } from "./decode-js.js";
import { decodeTs } from "./decode-ts.js";

export { decodeJs, decodeTs };

const FLAGS_OFFSET = 32;
const TYPESCRIPT_FLAG = 1;

export { encodeOptions } from "./options.js";

export function decode(buffer, source, sourceBuffer) {
  const arrayBuffer = buffer instanceof ArrayBuffer ? buffer : buffer.buffer;
  const view = new DataView(arrayBuffer, buffer.byteOffset ?? 0, buffer.byteLength);
  const flags = view.getInt32(FLAGS_OFFSET, true);
  return (flags & TYPESCRIPT_FLAG) !== 0
    ? decodeTs(buffer, source, sourceBuffer)
    : decodeJs(buffer, source, sourceBuffer);
}
