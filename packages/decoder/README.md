# `@swc-next/decoder`

Decodes the binary output of an SWC Next parser crate into an ESTree-compatible program, comments, and parser diagnostics.

`decode_js` assumes a JavaScript or JSX buffer; `decode_ts` assumes a TypeScript, TSX, or declaration-file buffer. The caller selects the decoder from the parser language.

The decoders are generated into separate `decode_js.js` and `decode_ts.js` modules, each with its own copy of the helpers. The handwritten `index.js` re-exports both decoders and `encodeOptions`; `index.d.ts` is also maintained by hand.

The binary format is internal and versioned with the Rust crates. Depend on the exact package version that matches the `swc_next_*` crate versions used by your native binding.

```js
import { decode_js, decode_ts, encodeOptions } from "@swc-next/decoder";
import { Lang, parseSync as __parseSync } from "./binding.js";

export function parseSync(source, options) {
  const buffer = __parseSync(source, encodeOptions(options));
  const lang = options?.lang;
  return lang === Lang.Ts || lang === Lang.Tsx || lang === Lang.Dts
    ? decode_ts(buffer, source)
    : decode_js(buffer, source);
}
```
