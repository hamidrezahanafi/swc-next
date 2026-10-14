import assert from "node:assert/strict";
import test from "node:test";
import { createWalker, visit } from "../index.js";

const id = (name) => ({ type: "Identifier", name });
const program = () => ({
  type: "Program",
  body: [{ type: "DebuggerStatement" }],
});

test("compiled walkers snapshot hooks and isolate each invocation", () => {
  let retained;
  const hooks = {
    DebuggerStatement(node, ctx) {
      ctx.state.push(node.type);
      retained = ctx;
      ctx.remove();
    },
  };
  const walker = createWalker(hooks);
  hooks.DebuggerStatement = () => assert.fail("hooks were not snapshotted");
  const first = program();
  const state = [];
  assert.equal(walker.walk(first, state), first);
  assert.deepEqual(first.body, []);
  assert.deepEqual(state, ["DebuggerStatement"]);
  assert.throws(() => retained.node, /only valid/);
  const second = program();
  walker.walk(second, state);
  assert.deepEqual(second.body, []);
  assert.equal(state.length, 2);
});

test("a compiled walker is reentrant and recovers after hook failures", () => {
  let outer;
  const walker = createWalker({
    Identifier(node, ctx) {
      if (node.name === "throw") throw new Error("hook failure");
      ctx.state.push(node.name);
      if (node.name === "outer") {
        outer = ctx;
        walker.walk(id("inner"), []);
        assert.equal(ctx.node, node);
      } else assert.notEqual(ctx, outer);
    },
  });
  walker.walk(id("outer"), []);
  assert.throws(() => walker.walk(id("throw"), []), /hook failure/);
  const state = [];
  walker.walk(id("after"), state);
  assert.deepEqual(state, ["after"]);
});

test("read-only visits preserve global, alias and concrete hook order", () => {
  const ast = {
    type: "FunctionDeclaration",
    id: id("f"),
    params: [],
    body: { type: "BlockStatement", body: [] },
  };
  const seen = [];
  assert.equal(
    visit(
      ast,
      {
        enter(node, state) {
          state.push(`enter:${node.type}`);
        },
        Function: {
          enter(node, state) {
            state.push("alias:enter");
          },
          leave(node, state) {
            state.push("alias:leave");
          },
        },
        FunctionDeclaration: {
          enter(node, state) {
            state.push("typed:enter");
          },
          leave(node, state) {
            state.push("typed:leave");
          },
        },
        leave(node, state) {
          state.push(`leave:${node.type}`);
        },
      },
      seen,
    ),
    ast,
  );
  assert.deepEqual(seen, [
    "enter:FunctionDeclaration",
    "alias:enter",
    "typed:enter",
    "enter:Identifier",
    "leave:Identifier",
    "enter:BlockStatement",
    "leave:BlockStatement",
    "typed:leave",
    "alias:leave",
    "leave:FunctionDeclaration",
  ]);
});

test("read-only traversal collects supported fields and rejects invalid callbacks", async () => {
  const ast = {
    type: "ArrayExpression",
    elements: [id("a"), null, id("b")],
    metadata: id("ignored"),
  };
  const original = structuredClone(ast);
  const seen = [];
  visit(
    ast,
    {
      Identifier(node, state) {
        state.push(node.name);
      },
    },
    seen,
  );
  assert.deepEqual(seen, ["a", "b"]);
  assert.deepEqual(ast, original);
  assert.throws(() => visit(ast, { Typo() {} }), /Unknown AST hook/);
  assert.throws(
    () =>
      visit(ast, {
        Identifier() {
          return 1;
        },
      }),
    /return undefined/,
  );
  assert.throws(
    () =>
      visit(ast, {
        async Identifier() {
          throw new Error("async failure");
        },
      }),
    /synchronous/,
  );
  await new Promise((resolve) => setImmediate(resolve));
  assert.throws(() => visit({ type: "Unknown" }, {}), /Unsupported AST/);
});
