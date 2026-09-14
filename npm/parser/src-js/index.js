import { Lang, parseSync as __parseSync } from "./binding.js";
import { decode_js, decode_ts, encodeOptions } from "@swc-next/decoder";

export * from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, encodeOptions(options));
  const lang = options?.lang;
  return lang === Lang.Ts || lang === Lang.Tsx || lang === Lang.Dts
    ? decode_ts(buffer, source)
    : decode_js(buffer, source);
}
