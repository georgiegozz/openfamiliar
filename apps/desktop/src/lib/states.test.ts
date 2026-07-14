import assert from "node:assert/strict";
import test from "node:test";
import { nextDemoState } from "./states";

test("demo state cycle", () => {
  assert.equal(nextDemoState("idle"), "thinking");
  assert.equal(nextDemoState("thinking"), "working");
  assert.equal(nextDemoState("success"), "idle");
});
