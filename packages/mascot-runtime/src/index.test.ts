import assert from "node:assert/strict";
import test from "node:test";
import { MascotRuntime, PERRITO_TECH_RUNTIME } from "./index";

test("advances frames at configured pixel-art fps", () => {
  const runtime = new MascotRuntime();
  runtime.play("thinking", true);
  assert.equal(runtime.current().frame, 2);
  assert.equal(runtime.tick(500).frame, 9);
});

test("non-interruptible success falls back to idle", () => {
  const runtime = new MascotRuntime();
  runtime.play("success", true);
  runtime.tick(1_000);
  assert.equal(runtime.current().animation, "idle");
});

test("pause freezes the frame", () => {
  const runtime = new MascotRuntime();
  runtime.pause();
  const before = runtime.current();
  const after = runtime.tick(10_000);
  assert.deepEqual(after, before);
});

test("idle remains visually static without an event", () => {
  const runtime = new MascotRuntime();
  const before = runtime.current();
  const after = runtime.tick(60_000);
  assert.equal(before.animation, "idle");
  assert.equal(before.frame, 0);
  assert.equal(after.animation, "idle");
  assert.equal(after.frame, 0);
});

test("friendly stable states use the reviewed collar-free frames", () => {
  assert.deepEqual(PERRITO_TECH_RUNTIME.animations.idle?.frames, [0]);
  assert.deepEqual(PERRITO_TECH_RUNTIME.animations.answering?.frames, [15]);
  assert.deepEqual(PERRITO_TECH_RUNTIME.animations.approval?.frames, [3]);
});
