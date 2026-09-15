import type { Comment, Program } from "@swc-next/ast";

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
  readonly program: Program;
  readonly comments: Comment[];
  /** Parser diagnostics, decoded lazily. Empty when parsing succeeds without diagnostics. */
  readonly diagnostics: Diagnostic[];
}

/** Decode a buffer produced by an SWC Next parser crate into ESTree. */
export declare function decode(
  buffer: Uint8Array,
  source: string,
): DecodeResult;
