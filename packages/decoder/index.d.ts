export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}

export interface DiagnosticLabel {
  message: string;
  /** UTF-16 offset in the source. */
  start: number;
  /** UTF-16 offset in the source. */
  end: number;
}

export interface Diagnostic extends DiagnosticLabel {
  severity: "error" | "warning" | "hint" | "info";
  help?: string;
  labels: DiagnosticLabel[];
}

export interface DecodeResult {
  readonly program: any;
  readonly comments: Comment[];
  readonly diagnostics: Diagnostic[];
}

/** Decode a JavaScript or JSX parser buffer into ESTree. */
export declare function decode_js(buffer: Uint8Array | ArrayBuffer, source: string): DecodeResult;

/** Decode a TypeScript, TSX, or declaration-file parser buffer into ESTree. */
export declare function decode_ts(buffer: Uint8Array | ArrayBuffer, source: string): DecodeResult;
