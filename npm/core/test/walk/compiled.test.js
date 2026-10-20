import { expect, test } from "@rstest/core";
import { createWalker, walk } from "@swc-next/walker";

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
