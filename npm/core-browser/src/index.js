import {
  Lang,
  parseSync as __parseSync,
  printSync as __printSync,
} from "./swc-next-core.browser.mjs";
import {
  decode_js,
  decode_ts,
  encodeOptions as encodeParserOptions,
} from "@swc-next/decoder";
import {
  encode,
  encodeOptions as encodeCodegenOptions,
} from "@swc-next/encoder";

export {
  default,
  CommentMode,
  Lang,
  SourceType,
  langFromPath,
  sourceTypeFromPath,
} from "./swc-next-core.browser.mjs";

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
