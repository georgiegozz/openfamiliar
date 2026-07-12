import test from "node:test";
import assert from "node:assert/strict";
import { assertNoTraversal, resolveStateAsset } from "./index.ts";

test("resolve approval alias", () => {
  const m = {
    id: "p", name: "P", version: "0.1.0", engine: ">=0.1.0", author: "a", license: "CC0-1.0",
    personality: "p.md",
    states: { idle: "i", thinking: "t", working: "w", approval: "ap", success: "s", error: "e" },
  };
  assert.equal(resolveStateAsset(m, "waiting_approval"), "ap");
});

test("blocks traversal", () => {
  assert.throws(() => assertNoTraversal("../x"));
});