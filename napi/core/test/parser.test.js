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

test("parseSync ignores comment punctuation in TypeScript recovery", () => {
  const object = parse("const o = { a /*?*/: 1 };", { lang: "ts" });
  assert.equal(object.body[0].declarations[0].init.properties[0].optional, false);

  const signature = parse("interface I { [k: string /*=*/]: number }", {
    lang: "ts",
  });
  assert.equal(signature.body[0].body.body[0].parameters[0].type, "Identifier");
});

test("parseSync preserves rest-element context during invalid recovery", () => {
  const parameters = parse("function f(...args = []) {}", { lang: "ts" });
  const parameter = parameters.body[0].params[0];
  assert.equal(parameter.type, "RestElement");
  assert.equal(parameter.argument.type, "AssignmentPattern");
  assert.equal(parameter.argument.left.type, "Identifier");
  assert.equal(parameter.argument.right.type, "ArrayExpression");

  for (const source of ["let [...x = y] = z;", "let {...x = y} = z;"]) {
    const declaration = parse(source, { lang: "ts" }).body[0].declarations[0];
    const rest = declaration.id.elements?.[0] ?? declaration.id.properties[0];
    assert.equal(rest.type, "RestElement");
    assert.equal(rest.argument.type, "AssignmentPattern");
  }

  const assignment = parse("[...x = y] = z;", { lang: "ts" });
  const assignmentRest = assignment.body[0].expression.left.elements[0];
  assert.equal(assignmentRest.type, "RestElement");
  assert.equal(assignmentRest.argument.type, "AssignmentPattern");
  assert.equal(assignmentRest.optional, false);
});

test("parseSync keeps script await and yield identifiers before unary operators", () => {
  for (const source of ["await + 1", "yield + 1"]) {
    const expression = parse(source, { lang: "ts" }).body[0].expression;
    assert.equal(expression.type, "BinaryExpression");
    assert.equal(expression.left.type, "Identifier");
  }
});

test("parseSync decodes XHTML entities in JSX text and attributes", () => {
  const expression = parse('<div title="&copy;&reg;">&euro;&Alpha;</div>', {
    lang: "tsx",
  }).body[0].expression;
  assert.equal(expression.openingElement.attributes[0].value.value, "©®");
  assert.equal(expression.children[0].value, "€Α");
});

test("parseSync reports precise postfix JSDoc child spans", () => {
  const type = parse("type T = string?", { lang: "ts" }).body[0].typeAnnotation;
  assert.deepEqual(
    [type.typeAnnotation.start, type.typeAnnotation.end],
    [type.start, type.end - 1],
  );
});
