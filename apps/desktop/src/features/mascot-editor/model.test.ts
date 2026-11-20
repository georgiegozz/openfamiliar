import assert from "node:assert/strict";
import test from "node:test";
import {
  assignFrame,
  cellIndex,
  cellOrigin,
  clampFps,
  defaultAnimations,
  isAllowedLicense,
} from "./model";

test("clamps editor fps to 1–12", () => {
  assert.equal(clampFps(0), 1);
  assert.equal(clampFps(12), 12);
  assert.equal(clampFps(13), 12);
  assert.equal(clampFps(4.4), 4);
});

test("assigns sheet cells to a state without out-of-range frames", () => {
  const base = defaultAnimations(6);
  assert.deepEqual(base.idle.frames, [0]);
  const next = assignFrame(base, "thinking", 9);
  assert.deepEqual(next.thinking.frames, [0, 9]);
  assert.deepEqual(assignFrame(next, "thinking", 99).thinking.frames, [0, 9]);
  assert.deepEqual(assignFrame(next, "idle", 3, true).idle.frames, [3]);
  assert.equal(cellIndex(3, 1), 7);
  assert.deepEqual(cellOrigin(7), { x: 192, y: 64 });
  assert.equal(isAllowedLicense("CC0-1.0"), true);
  assert.equal(isAllowedLicense("GPL-3.0"), false);
});
