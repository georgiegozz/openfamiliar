import assert from "node:assert/strict";
import test from "node:test";
import { sheetDataUrl, toRuntimeManifest } from "./localMascotRuntime";
import type { LocalMascotRuntime } from "../../lib/types";

test("maps a local pack runtime to the mascot runtime manifest", () => {
  const dto: LocalMascotRuntime = {
    id: "buddy-pal",
    name: "Buddy Pal",
    sheetPngBase64: "ZmFrZQ",
    frameWidth: 64,
    frameHeight: 64,
    columns: 4,
    rows: 4,
    animations: {
      idle: { frames: [0], fps: 1, loop: true },
      working: { frames: [2, 9], fps: 2, loop: true },
      success: { frames: [10, 0], fps: 3, loop: false },
    },
  };
  const manifest = toRuntimeManifest(dto);
  assert.equal(manifest.id, "buddy-pal");
  assert.equal(manifest.sheet.columns, 4);
  const thinking = manifest.animations.thinking;
  const listening = manifest.animations.listening;
  assert.ok(thinking);
  assert.ok(listening);
  assert.deepEqual(thinking.frames, [2, 9]);
  assert.equal(manifest.states.thinking, "thinking");
  assert.equal(manifest.states.listening, "listening");
  assert.deepEqual(listening.frames, [0]);
  assert.equal(
    sheetDataUrl(dto.sheetPngBase64),
    "data:image/png;base64,ZmFrZQ",
  );
});
