import type { Program } from "@swc-next/ast";

/** Encode an ESTree Program into the binary format consumed by SWC Next crates. */
export declare function encode(program: Program): Uint8Array;
