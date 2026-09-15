import type { Program } from "@swc-next/ast";

export interface CodegenOptions {
  asciiOnly?: boolean;
  minify?: boolean;
}

export interface CodegenOutput {
  code: string;
}

export declare function printSync(
  program: Program,
  options?: CodegenOptions | undefined | null,
): CodegenOutput;
