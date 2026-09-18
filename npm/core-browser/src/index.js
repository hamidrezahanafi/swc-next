import {
  parse as __parse,
  parseSync as __parseSync,
  printSync as __printSync,
} from "./swc-next-core.browser.mjs";
import {
  decode,
  encodeOptions as encodeParserOptions,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeCodegenOptions,
} from "@swc-next/encoder";

const textEncoder = new TextEncoder();
const canTransferToFixedLength =
  typeof ArrayBuffer.prototype.transferToFixedLength === "function";

export {
  default,
  CommentMode,
  Lang,
  SourceType,
  langFromPath,
  sourceTypeFromPath,
} from "./swc-next-core.browser.mjs";

export async function parse(source, options) {
  const sourceBytes = encodeSource(source);
  const buffer = await __parse(sourceBytes, source.length, encodeParserOptions(options));
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
  const buffer = __parseSync(sourceBytes, source.length, encodeParserOptions(options));
  return decode(buffer, source, sourceBytes);
}

export function printSync(program, options) {
  const code = __printSync(encode(program), encodeCodegenOptions(options));
  return { code };
}
