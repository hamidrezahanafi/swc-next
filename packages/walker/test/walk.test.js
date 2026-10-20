import { expect, test } from "@rstest/core";
import { walk, visitorKeys } from "@swc-next/walker";

const id = (name) => ({ type: "Identifier", name });
const statement = (name) => ({ type: "ExpressionStatement", expression: id(name) });
const program = (...names) => ({ type: "Program", body: names.map(statement) });
const bodyNames = (ast) => ast.body.map((node) => node.expression.name);

test("dispatches every supported node type", () => {
  for (const type of Object.keys(visitorKeys)) {
    const node = { type };
    const seen = [];
    expect(walk(node, { [type]: (current) => { seen.push(current); } })).toBe(node);
    expect(seen).toEqual([node]);
  }
});

test("visits schema children with state and parent context", () => {
  const ast = program("first", "second");
  ast.metadata = id("not-a-child");
  const state = { names: [] };

  expect(walk(ast, {
    Identifier(node, ctx) {
      ctx.state.names.push(node.name);
      expect(ctx.parent).toBe(ast.body[ctx.state.names.length - 1]);
      expect(ctx.key).toBe("expression");
      expect(ctx.index).toBeNull();
      expect(ctx.ancestors()).toEqual([ast, ctx.parent]);
    },
  }, state)).toBe(ast);
  expect(state.names).toEqual(["first", "second"]);
});

test("orders global, alias and concrete hooks", () => {
  const ast = { type: "FunctionExpression", params: [], body: { type: "BlockStatement", body: [] } };
  const seen = [];
  walk(ast, {
    enter(node) { if (node === ast) seen.push("global enter"); },
    Function: {
      enter() { seen.push("alias enter"); },
      leave() { seen.push("alias leave"); },
    },
    FunctionExpression: {
      enter() { seen.push("typed enter"); },
      leave() { seen.push("typed leave"); },
    },
    leave(node) { if (node === ast) seen.push("global leave"); },
  });
  expect(seen).toEqual([
    "global enter", "alias enter", "typed enter",
    "typed leave", "alias leave", "global leave",
  ]);
});

test("replacement visits new children and selects leave hooks for the new type", () => {
  const ast = program("before");
  const seen = [];
  walk(ast, {
    Identifier(node, ctx) {
      seen.push(node.name);
      if (node.name === "before") {
        ctx.replace({ type: "UnaryExpression", operator: "!", prefix: true, argument: id("after") });
      }
    },
    UnaryExpression: {
      enter() { seen.push("unexpected re-entry"); },
      leave() { seen.push("unary leave"); },
    },
  });
  expect(seen).toEqual(["before", "after", "unary leave"]);

  seen.length = 0;
  walk(ast, {
    Identifier(node) { seen.push(node.name); },
    UnaryExpression: { leave(_, ctx) { ctx.replace(id("final")); } },
  });
  expect(seen).toEqual(["after"]);
  expect(ast.body[0].expression).toEqual(id("final"));
});

test("skip retains leave while remove skips children and leave", () => {
  const ast = program("skip", "remove", "keep");
  const seen = [];
  walk(ast, {
    ExpressionStatement: {
      enter(node, ctx) {
        if (node.expression.name === "skip") ctx.skip();
        if (node.expression.name === "remove") ctx.remove();
      },
      leave(node) { seen.push(`leave ${node.expression.name}`); },
    },
    Identifier(node) { seen.push(node.name); },
  });
  expect(seen).toEqual(["leave skip", "keep", "leave keep"]);
  expect(bodyNames(ast)).toEqual(["skip", "keep"]);
});

test("insertions preserve order and visit only nodes inserted after the current node", () => {
  const ast = program("current", "tail");
  const seen = [];
  walk(ast, {
    ExpressionStatement(node, ctx) {
      if (node.expression.name !== "current") return;
      ctx.insertBefore([statement("before1"), statement("before2")]);
      expect(ctx.index).toBe(2);
      ctx.insertAfter(statement("after1"));
      ctx.insertAfter(statement("after2"));
    },
    Identifier(node) { seen.push(node.name); },
  });
  expect(bodyNames(ast)).toEqual(["before1", "before2", "current", "after1", "after2", "tail"]);
  expect(seen).toEqual(["current", "after1", "after2", "tail"]);
});

test("stop retains edits and suppresses siblings and pending leave hooks", () => {
  const ast = program("first", "second");
  const leaves = [];
  walk(ast, {
    Identifier(node, ctx) {
      node.name = "changed";
      ctx.stop();
    },
    leave(node) { leaves.push(node.type); },
  });
  expect(bodyNames(ast)).toEqual(["changed", "second"]);
  expect(leaves).toEqual([]);
});

test("contexts are reused, nested walks are isolated and saved contexts expire", () => {
  const ast = program("one", "two");
  const contexts = [];
  const ancestors = [];
  walk(ast, {
    Identifier(node, ctx) {
      contexts.push(ctx);
      ancestors.push(ctx.ancestors());
      walk(program("nested"), {
        Identifier(_, nested) {
          expect(nested).not.toBe(ctx);
          expect(ctx.node).toBe(node);
        },
      });
      expect(ctx.node).toBe(node);
    },
  });
  expect(contexts[0]).toBe(contexts[1]);
  expect(ancestors).toEqual([[ast, ast.body[0]], [ast, ast.body[1]]]);
  expect(() => contexts[0].parent).toThrow(/only valid/);
  expect(() => contexts[0].replace(id("bad"))).toThrow(/only valid/);

  let failedContext;
  expect(() => walk(ast, {
    Program(_, ctx) {
      failedContext = ctx;
      throw new Error("hook failure");
    },
  })).toThrow("hook failure");
  expect(() => failedContext.node).toThrow(/only valid/);
});

test("rejects invalid callbacks, nodes and structural edits", () => {
  expect(() => walk(program(), { Typo() {} })).toThrow(/Unknown AST hook/);
  expect(() => walk({ type: "Unknown" }, {})).toThrow(/Unsupported AST/);
  expect(() => walk(program(), { Program: async () => {} })).toThrow(/synchronous/);
  expect(() => walk(program(), { Program: () => false })).toThrow(/return undefined/);
  expect(() => walk(program(), { Program(_, ctx) { ctx.remove(); } })).toThrow(/root/);
  expect(() => walk(program("a"), { Identifier(_, ctx) { ctx.remove(); } })).toThrow(/array element/);
  expect(() => walk(program(), { leave(_, ctx) { ctx.skip(); } })).toThrow(/enter/);
  expect(() => walk(program("a"), {
    Identifier(_, ctx) {
      ctx.parent.expression = id("changed");
      ctx.replace(id("bad"));
    },
  })).toThrow(/container/);
});
