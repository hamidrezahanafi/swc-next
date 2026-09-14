import {
  Lang,
  parseSync as __parseSync,
  printSync as __printSync,
} from "./binding.js";
import {
  decode_js,
  decode_ts,
  encodeOptions as encodeParserOptions,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeCodegenOptions,
} from "@swc-next/encoder";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, encodeParserOptions(options));
  const lang = options?.lang;
  return lang === Lang.Ts || lang === Lang.Tsx || lang === Lang.Dts
    ? decode_ts(buffer, source)
    : decode_js(buffer, source);
}

export function printSync(program, options) {
  const code = __printSync(encode(program), encodeCodegenOptions(options));
  return { code };
}
