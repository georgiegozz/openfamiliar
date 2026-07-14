import assert from "node:assert/strict";
import test from "node:test";
import { MascotRuntime, chooseAmbientAction, nextAmbientDelay } from "./index";

test("advances frames at configured pixel-art fps", () => {
  const runtime = new MascotRuntime();
  runtime.play("blink", true);
  assert.equal(runtime.current().frame, 0);
  assert.equal(runtime.tick(125).frame, 2);
});

test("non-interruptible success falls back to idle", () => {
  const runtime = new MascotRuntime();
  runtime.play("success", true);
  runtime.tick(500);
  assert.equal(runtime.current().animation, "idle");
});

test("pause freezes the frame", () => {
  const runtime = new MascotRuntime();
  runtime.pause();
  const before = runtime.current();
  const after = runtime.tick(10_000);
  assert.deepEqual(after, before);
});

test("ambient selection and delay are bounded", () => {
  assert.equal(chooseAmbientAction(0), "breathe");
  assert.equal(chooseAmbientAction(1), "special-idle");
  assert.equal(nextAmbientDelay(0), 4_000);
  assert.equal(nextAmbientDelay(1), 12_000);
});
