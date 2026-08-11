import assert from "node:assert/strict";
import test from "node:test";

import { parseSync } from "../src-js/index.js";

function parse(source, options) {
  return parseSync(source, { sourceType: "script", ...options }).program;
}

test("parseSync preserves decoded string-pool values", () => {
  const identifier = `_${"\u0816".repeat(100)}`;
  const escaped = "\\u0816".repeat(100);
  const program = parse(`const _${escaped} = "\ufeffvalue";`);

  const declaration = program.body[0].declarations[0];
  assert.equal(declaration.id.name, identifier);
  assert.equal(declaration.init.value, "\ufeffvalue");
});

test("parseSync normalizes template raw line endings", () => {
  const program = parse("const value = `a\r\nb\rc`;");
  const template = program.body[0].declarations[0].init;

  assert.equal(template.quasis[0].value.raw, "a\nb\nc");
  assert.equal(template.quasis[0].value.cooked, "a\nb\nc");
});

test("parseSync preserves TypeScript ESTree template raw line endings", () => {
  const program = parse("`a\rb`", { lang: "ts" });
  const template = program.body[0].expression;

  assert.equal(template.quasis[0].value.raw, "a\rb");
  assert.equal(template.quasis[0].value.cooked, "a\nb");
});

test("parseSync preserves parenthesized assignment targets", () => {
  const program = parse("(a) = 1; (b) += 1; (c)++; for ((d) in obj) {}");

  assert.equal(program.body[0].expression.left.type, "ParenthesizedExpression");
  assert.equal(program.body[1].expression.left.type, "ParenthesizedExpression");
  assert.equal(program.body[2].expression.argument.type, "ParenthesizedExpression");
  assert.equal(program.body[3].left.type, "ParenthesizedExpression");
});

test("parseSync keeps Annex B call-expression targets for semantic analysis", () => {
  const program = parse(
    "f() = 1; f() += 1; ++f(); f()++; for (f() in obj) {} for (f() of obj) {}",
  );

  assert.equal(program.body.length, 6);
  assert.equal(program.body[0].expression.left.type, "CallExpression");
  assert.equal(program.body[1].expression.left.type, "CallExpression");
  assert.equal(program.body[2].expression.argument.type, "CallExpression");
  assert.equal(program.body[3].expression.argument.type, "CallExpression");
  assert.equal(program.body[4].left.type, "CallExpression");
  assert.equal(program.body[5].left.type, "CallExpression");
});

test("parseSync represents overflowing numeric values as Infinity", () => {
  const program = parse("1e400;");
  assert.equal(program.body[0].expression.value, Infinity);
});
