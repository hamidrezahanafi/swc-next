import assert from "node:assert/strict";
import test from "node:test";
import { createWalker, visit, walk } from "../index.js";

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

test("unobserved parents select leave hooks after a child changes their type", () => {
  const ast = {
    type: "BinaryExpression",
    left: id("left"),
    right: id("right"),
  };
  const seen = [];
  walk(ast, {
    Identifier(node, ctx) {
      seen.push(node.name);
      ctx.parent.type = "LogicalExpression";
    },
    LogicalExpression: {
      leave(node, ctx) {
        assert.equal(ctx.node, ast);
        assert.deepEqual(ctx.ancestors(), []);
        seen.push("leave");
      },
    },
  });
  assert.deepEqual(seen, ["left", "right", "leave"]);
});

test("compiled hooks snapshot alias and concrete pairs without freezing caller objects", () => {
  const seen = [];
  const pair = {
    enter() {
      seen.push("typed");
    },
  };
  const hooks = {
    Function() {
      seen.push("alias");
    },
    FunctionDeclaration: pair,
  };
  const walker = createWalker(hooks);
  pair.enter = () => assert.fail("pair was read again");
  hooks.Function = () => assert.fail("alias was read again");
  walker.walk({
    type: "FunctionDeclaration",
    params: [],
    body: { type: "BlockStatement", body: [] },
  });
  assert.deepEqual(seen, ["alias", "typed"]);
});

test("supported type membership rejects inherited object names", () => {
  assert.throws(() => walk({ type: "toString" }, {}), TypeError);
  assert.throws(() => walk({ type: "constructor" }, {}), TypeError);
  assert.throws(() => walk({ type: "__proto__" }, {}), TypeError);
  assert.throws(() => createWalker({ toString() {} }), TypeError);
  const hooks = Object.create(null);
  hooks.Identifier = (node, ctx) => {
    ctx.state.push(node.name);
  };
  const state = [];
  walk(id("valid"), hooks, state);
  assert.deepEqual(state, ["valid"]);
});

test("list edits preserve inserted type dispatch and stop behavior", () => {
  const first = { type: "TemplateElement", value: { raw: "first", cooked: "first" } };
  const last = { type: "TemplateElement", value: { raw: "last", cooked: "last" } };
  const ast = { type: "TemplateLiteral", quasis: [first, null, last] };
  const seen = [];
  walk(ast, {
    TemplateElement(node, ctx) {
      seen.push(node.value.raw);
      if (node === first) {
        ctx.insertBefore(id("before"));
        ctx.insertAfter({
          type: "ExpressionStatement",
          expression: { type: "Literal", value: "nested" },
        });
        ctx.remove();
      }
    },
    ExpressionStatement(node, ctx) {
      seen.push("after");
      assert.equal(ctx.parent, ast);
      assert.equal(ctx.key, "quasis");
      assert.equal(ctx.index, 1);
      assert.deepEqual(ctx.ancestors(), [ast]);
    },
    Literal(node) {
      seen.push(node.value);
    },
  });
  assert.deepEqual(seen, ["first", "after", "nested", "last"]);
  assert.equal(ast.quasis[0].name, "before");
  assert.equal(ast.quasis[2], null);
  assert.equal(ast.quasis[3], last);
  walk(ast, {
    ExpressionStatement(node, ctx) {
      ctx.stop();
    },
    TemplateElement() {
      assert.fail("later list elements are not visited after stop");
    },
    TemplateLiteral: {
      leave() {
        assert.fail("pending leave is not called after stop");
      },
    },
  });
});

test("list elements retain generic types and single discriminator reads", () => {
  let reads = 0;
  const child = {
    get type() {
      reads++;
      assert.equal(reads, 1);
      return "Literal";
    },
    value: "changed",
  };
  const root = { type: "TemplateLiteral", quasis: [child] };
  assert.equal(walk(root, {}), root);
  assert.equal(reads, 1);
  let invalidReads = 0;
  const invalid = {
    get type() {
      invalidReads++;
      return undefined;
    },
  };
  assert.throws(
    () => walk({ type: "TemplateLiteral", quasis: [invalid] }, {}),
    TypeError,
  );
  assert.equal(invalidReads, 1);
});

test("read-only visitors retain state and mixed list element traversal", () => {
  const ast = {
    type: "TemplateLiteral",
    quasis: [
      { type: "TemplateElement", value: { raw: "binding", cooked: "binding" } },
      null,
      { type: "Literal", value: "fallback" },
    ],
  };
  const original = structuredClone(ast);
  const seen = [];
  assert.equal(
    visit(
      ast,
      {
        TemplateElement(node, state) {
          state.push(node.value.raw);
        },
        Identifier(node, state) {
          state.push(node.name);
        },
        Literal(node, state) {
          state.push(node.value);
        },
      },
      seen,
    ),
    ast,
  );
  assert.deepEqual(seen, ["binding", "fallback"]);
  assert.deepEqual(ast, original);
});
