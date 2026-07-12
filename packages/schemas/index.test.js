import test from "node:test";
import assert from "node:assert/strict";
import { validateManifestShape } from "./index.js";

test("valid minimal manifest", () => {
  const r = validateManifestShape({
    id: "x",
    name: "X",
    version: "0.1.0",
    engine: ">=0.1.0",
    author: "a",
    license: "CC0-1.0",
    personality: "personality.md",
    states: {
      idle: "a", thinking: "b", working: "c", approval: "d", success: "e", error: "f",
    },
  });
  assert.equal(r.ok, true);
});

test("rejects missing license", () => {
  const r = validateManifestShape({ id: "x", name: "X", version: "0.1.0", engine: ">=0.1.0", author: "a", personality: "p", states: {} });
  assert.equal(r.ok, false);
});