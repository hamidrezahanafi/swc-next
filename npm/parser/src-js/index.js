import { parse as __parse, parseSync as __parseSync } from "./binding.js";
import { decode, encodeOptions } from "@swc-next/decoder";

const textEncoder = new TextEncoder();
const canTransferToFixedLength =
  typeof ArrayBuffer.prototype.transferToFixedLength === "function";

export * from "./binding.js";

export async function parse(source, options) {
  const sourceBytes = encodeSource(source);
  const buffer = await __parse(sourceBytes, source.length, encodeOptions(options));
  return decode(buffer, source, sourceBytes);
}

function encodeSource(source) {
  const sourceBuffer = new Uint8Array(source.length * 3);
  const { written } = textEncoder.encodeInto(source, sourceBuffer);
  // Lazy decoder getters retain sourceBytes, so drop the unused capacity of
  // the encodeInto buffer instead of keeping it alive through a subarray.
  const sourceBytes = canTransferToFixedLength
    ? new Uint8Array(sourceBuffer.buffer.transferToFixedLength(written))
    : sourceBuffer.slice(0, written);
  return sourceBytes;
}

export function parseSync(source, options) {
  const sourceBytes = encodeSource(source);
  const buffer = __parseSync(sourceBytes, source.length, encodeOptions(options));
  return decode(buffer, source, sourceBytes);
}
