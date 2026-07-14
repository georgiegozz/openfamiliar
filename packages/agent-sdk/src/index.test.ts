import test from "node:test";
import assert from "node:assert/strict";
import type { AgentEvent } from "./index";

test("event shape", () => {
  const e: AgentEvent = { type: "session.started", sessionId: "1" };
  assert.equal(e.type, "session.started");
});
