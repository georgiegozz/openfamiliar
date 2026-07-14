import test from "node:test";
import assert from "node:assert/strict";
import { collectStream, type ModelProvider, type ChatRequest } from "./index";

test("collectStream joins deltas", async () => {
  const provider: ModelProvider = {
    id: "mock",
    async validateConfiguration() {
      return { ok: true, message: "ok" };
    },
    async listModels() {
      return [{ id: "m", name: "m" }];
    },
    async *stream() {
      yield { type: "delta", text: "a" };
      yield { type: "delta", text: "b" };
      yield { type: "done" };
    },
    async cancel() {},
  };
  const req: ChatRequest = { model: "m", messages: [], sessionId: "s" };
  assert.equal(await collectStream(provider, req), "ab");
});
