import assert from "node:assert/strict";
import test from "node:test";

import { parseSync } from "../src-js/index.js";

function parse(source, options) {
  return parseSync(source, { sourceType: "script", ...options }).program;
}

test("parseSync preserves parenthesized assignment targets", () => {
  const program = parse(
    "(a) = 1; ((b)) += 1; ++(c); ((d))--; for ((e) in obj) {} for ((f) of obj) {}",
  );

  assert.equal(program.body.length, 6);
  assert.equal(program.body[0].expression.left.type, "ParenthesizedExpression");
  assert.equal(program.body[1].expression.left.type, "ParenthesizedExpression");
  assert.equal(
    program.body[1].expression.left.expression.type,
    "ParenthesizedExpression",
  );
  assert.equal(
    program.body[2].expression.argument.type,
    "ParenthesizedExpression",
  );
  assert.equal(
    program.body[3].expression.argument.type,
    "ParenthesizedExpression",
  );
  assert.equal(
    program.body[3].expression.argument.expression.type,
    "ParenthesizedExpression",
  );
  assert.equal(program.body[4].left.type, "ParenthesizedExpression");
  assert.equal(program.body[5].left.type, "ParenthesizedExpression");
});

test("parseSync preserves Annex B call-expression assignment targets", () => {
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
