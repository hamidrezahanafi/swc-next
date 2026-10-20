import { expect, test } from "@rstest/core";
import { createWalker, visit, walk } from "../index.js";

const id = (name) => ({ type: "Identifier", name });
const program = () => ({
  type: "FunctionDeclaration",
  id: id("f"),
  params: [],
  body: { type: "BlockStatement", body: [{ type: "DebuggerStatement" }] },
});

test("compiled walkers snapshot hooks and isolate invocation state", () => {
  const pair = {
    enter(node, ctx) {
      ctx.state.push("typed");
    },
  };
  const hooks = {
    Function(node, ctx) {
      ctx.state.push("alias");
    },
    FunctionDeclaration: pair,
    DebuggerStatement(node, ctx) {
      ctx.state.push(node.type);
      ctx.remove();
    },
  };
  const walker = createWalker(hooks);
  hooks.Function = pair.enter = () => {
    throw new Error("hooks were read again");
  };
  const first = program();
  const second = program();
  const firstState = ["first"];
  const secondState = ["second"];
  expect(walker.walk(first, firstState)).toBe(first);
  walker.walk(second, secondState);
  expect(first.body.body).toEqual([]);
  expect(second.body.body).toEqual([]);
  expect(firstState).toEqual(["first", "alias", "typed", "DebuggerStatement"]);
  expect(secondState).toEqual(["second", "alias", "typed", "DebuggerStatement"]);
});

test("compiled walkers support reentrancy and recover after hook failures", () => {
  const outerState = [];
  const innerState = [];
  const walker = createWalker({
    Identifier(node, ctx) {
      if (node.name === "throw") throw new Error("hook failure");
      ctx.state.push(node.name);
      if (node.name === "outer") {
        walker.walk(id("inner"), innerState);
        expect(ctx.node).toBe(node);
        expect(ctx.state).toBe(outerState);
        ctx.state.push("resumed");
      }
    },
  });
  walker.walk(id("outer"), outerState);
  expect(outerState).toEqual(["outer", "resumed"]);
  expect(innerState).toEqual(["inner"]);
  expect(() => walker.walk(id("throw"), [])).toThrow("hook failure");
  const afterState = [];
  walker.walk(id("after"), afterState);
  expect(afterState).toEqual(["after"]);
});

test("read-only visits preserve global, alias and concrete hook order", () => {
  const ast = program();
  ast.body.body = [];
  const original = structuredClone(ast);
  const seen = [];
  const record = (phase) => (node, state) => {
    state.push(`${phase}:${node.type}`);
  };
  expect(
    visit(
      ast,
      {
        enter: record("enter"),
        Function: { enter: record("alias:enter"), leave: record("alias:leave") },
        FunctionDeclaration: {
          enter: record("typed:enter"),
          leave: record("typed:leave"),
        },
        leave: record("leave"),
      },
      seen,
    ),
  ).toBe(ast);
  expect(seen).toEqual([
    "enter:FunctionDeclaration",
    "alias:enter:FunctionDeclaration",
    "typed:enter:FunctionDeclaration",
    "enter:Identifier",
    "leave:Identifier",
    "enter:BlockStatement",
    "leave:BlockStatement",
    "typed:leave:FunctionDeclaration",
    "alias:leave:FunctionDeclaration",
    "leave:FunctionDeclaration",
  ]);
  expect(ast).toEqual(original);
});

test("list edits dispatch inserted nodes and stop subsequent traversal", () => {
  const first = { type: "VariableDeclarator", id: id("first"), init: null };
  const last = { type: "VariableDeclarator", id: id("last"), init: null };
  const inserted = {
    type: "ExpressionStatement",
    expression: { type: "Literal", value: "nested" },
  };
  const ast = { type: "VariableDeclaration", declarations: [first, null, last] };
  const seen = [];
  walk(ast, {
    VariableDeclarator(node, ctx) {
      seen.push(node.id.name);
      if (node === first) {
        ctx.insertBefore(id("before"));
        ctx.insertAfter(inserted);
        ctx.remove();
      }
    },
    ExpressionStatement() {
      seen.push("inserted");
    },
    Literal(node) {
      seen.push(node.value);
    },
  });
  expect(seen).toEqual(["first", "inserted", "nested", "last"]);
  expect(ast.declarations).toEqual([id("before"), inserted, null, last]);

  const stopped = [];
  walk(ast, {
    enter(node) {
      stopped.push(node.type);
    },
    ExpressionStatement(node, ctx) {
      ctx.stop();
    },
    VariableDeclaration: {
      leave() {
        stopped.push("leave");
      },
    },
  });
  expect(stopped).toEqual(["VariableDeclaration", "Identifier", "ExpressionStatement"]);
});

test("read-only visits traverse mixed lists and ignore non-child fields", () => {
  const ast = {
    type: "TemplateLiteral",
    quasis: [
      { type: "TemplateElement", value: { raw: "binding", cooked: "binding" } },
      null,
      { type: "Literal", value: "fallback" },
    ],
    metadata: id("ignored"),
  };
  const original = structuredClone(ast);
  const seen = [];
  visit(ast, {
    TemplateElement(node) {
      seen.push(node.value.raw);
    },
    Literal(node) {
      seen.push(node.value);
    },
    Identifier(node) {
      seen.push(node.name);
    },
  });
  expect(seen).toEqual(["binding", "fallback"]);
  expect(ast).toEqual(original);
});
