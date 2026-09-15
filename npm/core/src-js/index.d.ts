import type { Comment, Program } from "@swc-next/ast";

export type * as ast from "@swc-next/ast";

import type {
  CodegenOptions,
  CodegenOutput,
  ParserOptions,
} from "./binding.js";

export * from "./binding.js";

export type { Comment } from "@swc-next/ast";

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

export interface ParseResult {
  readonly program: Program;
  readonly comments: Comment[];
  /** Parser diagnostics, decoded lazily. Empty when parsing succeeds without diagnostics. */
  readonly diagnostics: Diagnostic[];
}

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): ParseResult;

export declare function printSync(
  program: Program,
  options?: CodegenOptions | undefined | null,
): CodegenOutput;
