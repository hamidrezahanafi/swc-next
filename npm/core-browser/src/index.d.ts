import type { Lang, Program, SourceType } from "@swc-next/types";
export type * from "@swc-next/types";

import type {
  CodegenOptions,
  CodegenOutput,
  ParserOptions,
} from "./binding.js";
import type { AstMeta } from "@swc-next/decoder";
export type { AstMeta } from "@swc-next/decoder";

export type {
  CommentMode,
  CodegenOptions,
  CodegenOutput,
  ParserOptions,
} from "./binding.js";

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

export interface ParseResult {
  readonly meta: AstMeta;
  readonly program: Program;
  readonly comments: Comment[];
  /** Parser diagnostics and, when semanticErrors is enabled, semantic early errors. Decoded lazily. */
  readonly diagnostics: Diagnostic[];
}

/**
 * Initializes the WebAssembly binding. Call this once before using the browser API.
 * Only the first call performs initialization; subsequent calls are idempotent no-ops.
 */
export default function init(): Promise<void>;

export declare function parseSync(
  source: string,
  options?: ParserOptions | undefined | null,
): ParseResult;

export declare function printSync(
  program: Program,
  options?: CodegenOptions | undefined | null,
  meta?: Partial<AstMeta>,
): CodegenOutput;

export declare function langFromPath(path: string): Lang;
export declare function sourceTypeFromPath(path: string): SourceType;

export { walk, visitorKeys } from "./ast.js";
export type {
  AstNode,
  NodeType,
  WalkContext,
  WalkHandler,
  WalkHook,
  WalkHooks,
  TransformPlugin,
} from "./ast.js";

export interface TransformOptions {
  parser?: ParserOptions;
  codegen?: CodegenOptions;
  /** Each plugin completes a full walk before the next plugin starts. */
  plugins?: readonly import("./ast.js").TransformPlugin[];
}

export interface TransformResult extends CodegenOutput {
  program: import("./ast.js").AstNode & { type: "Program" };
  diagnostics: Diagnostic[];
  meta: AstMeta;
}

/** Throws SyntaxError (with diagnostics) before running plugins on parser errors. */
export declare function transformSync(
  source: string,
  options?: TransformOptions,
): TransformResult;
