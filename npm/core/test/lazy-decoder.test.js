import assert from "node:assert/strict";
import test from "node:test";
import { inspect } from "node:util";

import { decode as decodeEager, encodeOptions } from "@swc-next/decoder";
import { decode, decodeJs, decodeTs } from "@swc-next/decoder/lazy";
import { parseSync as parseBuffer } from "../src-js/binding.js";
import { printSync } from "../src-js/index.js";

function parseRaw(source, options) {
  const sourceBuffer = new TextEncoder().encode(source);
  const buffer = parseBuffer(sourceBuffer, source.length, encodeOptions(options));
  return { buffer, sourceBuffer };
}

function parseLazy(source, options) {
  const { buffer, sourceBuffer } = parseRaw(source, options);
  return decode(buffer, source, sourceBuffer);
}

function materialize(value) {
  if (value === null || typeof value !== "object" || value instanceof RegExp) return value;
  if (Array.isArray(value)) return Array.from(value, materialize);
  const object = typeof value.toJSON === "function" ? value.toJSON() : value;
  return Object.fromEntries(Object.entries(object).map(([key, entry]) => [key, materialize(entry)]));
}

function assertMatchesEager(source, options) {
  const { buffer, sourceBuffer } = parseRaw(source, options);
  const result = decode(buffer, source, sourceBuffer);
  assert.deepEqual(materialize(result), decodeEager(buffer, source, sourceBuffer));
}

test("lazy decoder constructs only accessed nodes and caches their identity", () => {
  const source = "first(); second();";
  const { buffer, sourceBuffer } = parseRaw(source);
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const nodeCount = view.getInt32(0, true);
  const kinds = 40 + nodeCount * 8;
  const originalKinds = Uint8Array.from(buffer.subarray(kinds, kinds + nodeCount));
  for (let index = 0; index < nodeCount; index++) {
    if (view.getInt32(40 + index * 8, true) === source.indexOf("second")) {
      buffer[kinds + index] = 255;
    }
  }

  const result = decodeJs(buffer, source, sourceBuffer);
  const program = result.program;
  const body = program.body;
  assert.equal(program.type, "Program");
  assert.equal(Object.getPrototypeOf(Object.getPrototypeOf(program)), Object.prototype);
  assert.throws(() => new program.constructor(), Error);
  assert.equal(result.program, program);
  assert.equal(program.body, body);
  assert.equal(body.length, 2);
  assert.deepEqual(Reflect.ownKeys(body), ["0", "1", "length"]);
  assert.equal(body[0].expression.callee.name, "first");
  assert.equal(body[0], body[0]);
  assert.equal(body[0].expression, body[0].expression);
  assert.throws(() => body[1], RangeError);

  buffer.set(originalKinds, kinds);
  assert.equal(body[1].expression.callee.name, "second");
  assert.equal(body[1], [...body][1]);
});

test("lazy parameter lists defer rest nodes and default expressions", () => {
  const source = "function f(x = missing(), ...rest) {}";
  const { buffer, sourceBuffer } = parseRaw(source);
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const nodeCount = view.getInt32(0, true);
  const kinds = 40 + nodeCount * 8;
  for (let index = 0; index < nodeCount; index++) {
    const start = view.getInt32(40 + index * 8, true);
    if (start === source.indexOf("missing") || start === source.indexOf("...")) {
      buffer[kinds + index] = 255;
    }
  }

  const params = decode(buffer, source, sourceBuffer).program.body[0].params;
  assert.equal(params.length, 2);
  assert.equal(params[0].type, "AssignmentPattern");
  assert.equal(params[0].left.name, "x");
  assert.throws(() => params[0].right, RangeError);
  assert.throws(() => params[1], RangeError);
});

test("lazy decoder reads scalar fields only on access", () => {
  const source = "42;";
  const { buffer, sourceBuffer } = parseRaw(source);
  const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  const nodeCount = view.getInt32(0, true);
  const payloads = (40 + nodeCount * 9 + 3) & ~3;
  let literalIndex = -1;
  for (let index = 0; index < nodeCount; index++) {
    if (view.getInt32(40 + index * 8 + 4, true) === 2) literalIndex = index;
  }
  assert.notEqual(literalIndex, -1);

  const literal = decode(buffer, source, sourceBuffer).program.body[0].expression;
  view.setFloat64(payloads + literalIndex * 16, 7, true);
  assert.equal(literal.type, "Literal");
  assert.equal(literal.value, 7);
  view.setFloat64(payloads + literalIndex * 16, 9, true);
  assert.equal(literal.value, 9);
  assert.equal(literal.raw, "42");
});

test("lazy node arrays support normal array reads, holes, and rest elements", () => {
  const program = parseLazy("const [a,,...rest] = value;").program;
  const elements = program.body[0].declarations[0].id.elements;
  assert.equal(Array.isArray(elements), true);
  assert.equal(elements.length, 3);
  assert.equal(elements[1], null);
  assert.equal(elements[2].type, "RestElement");
  assert.equal(elements[2].argument.name, "rest");
  assert.deepEqual(elements.map(node => node?.type), ["Identifier", undefined, "RestElement"]);
  assert.deepEqual([...elements.keys()], [0, 1, 2]);
  assert.equal(elements.slice(-1)[0], elements[2]);
  assert.equal(elements.at(-1), elements[2]);
  assert.equal(elements[3], undefined);
  assert.equal(elements["01"], undefined);
  assert.throws(() => { elements[0] = null; }, TypeError);
  assert.throws(() => { elements.length = 0; }, TypeError);
  assert.throws(() => { delete elements[0]; }, TypeError);
  assert.throws(() => Object.freeze(elements), TypeError);
});

test("lazy decoder matches eager JavaScript projections", () => {
  assertMatchesEager('"use strict"; function f(x = 1, ...rest) { return x; }');
  assertMatchesEager("let {a = 1, ...rest} = value;");
  assertMatchesEager("class C extends B { static x = 1; accessor y; get z() { return this.x; } }");
  assertMatchesEager("try {} catch ({message}) {}");
  assertMatchesEager("0xFF; 1_000n; /foo/gi; null; true;");
  assertMatchesEager('/* 😀 */ const 文 = "é\\uD800";');
  assertMatchesEager("`hello ${value} 😀`;");
  assertMatchesEager('<X a="é">hello {value}</X>;', { lang: "jsx" });
});

test("lazy decoder matches eager TypeScript projections", () => {
  const options = { lang: "ts" };
  assertMatchesEager("function f(this: C, x?: number, y: string = '', ...rest: string[]): void {}", options);
  assertMatchesEager("const {a, ...rest}: T = value;", options);
  assertMatchesEager("try {} catch (error: unknown) {}", options);
  assertMatchesEager("abstract class C<T> extends B<T> implements I { abstract f(x: number): void; declare x: string; }", options);
  assertMatchesEager("type F = (this: C, x: number) => string;", options);
  assertMatchesEager("interface I { f(this: C, ...args: string[]): void; (this: C, x: number): void; [key: string]: number; }", options);
  assertMatchesEager("type M<T> = { readonly [K in keyof T as K]?: T[K] };", options);
  assertMatchesEager("namespace X {} declare namespace N; declare global { var x: string; }", options);
  assertMatchesEager("", options);
  assertMatchesEager("const x = <X<string> />;", { lang: "tsx" });
});

test("lazy decoder preserves comments, diagnostics, serialization, and codegen", () => {
  assertMatchesEager("/* 😀 */ const = 1;");

  const source = "// comment\nconst answer = 42;";
  const { buffer, sourceBuffer } = parseRaw(source, { lang: "ts" });
  const result = decodeTs(buffer, source, sourceBuffer);
  assert.equal(result.comments, result.comments);
  assert.equal(result.diagnostics, result.diagnostics);
  assert.deepEqual(JSON.parse(JSON.stringify(result)), JSON.parse(JSON.stringify(decodeEager(buffer, source, sourceBuffer))));
  assert.equal(printSync(result.program).code, "const answer = 42;\n");
});

test("lazy nodes and node arrays expose named inspection output", () => {
  const program = parseLazy("let x = 1;").program;
  assert.match(inspect(program, { depth: 1 }), /^Program \{/);
  assert.match(inspect(program.body), /^NodeArray\(1\)/);
});

test("lazy decoder supports unaligned buffers and ArrayBuffer input", () => {
  const source = "let x: number = 1;";
  const { buffer, sourceBuffer } = parseRaw(source, { lang: "ts" });
  const unaligned = new Uint8Array(new ArrayBuffer(buffer.byteLength + 1), 1);
  unaligned.set(buffer);
  assert.deepEqual(materialize(decode(unaligned, source, sourceBuffer)), decodeEager(buffer, source, sourceBuffer));
  assert.deepEqual(materialize(decode(Uint8Array.from(buffer).buffer, source, sourceBuffer)), decodeEager(buffer, source, sourceBuffer));
});
