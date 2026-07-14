import assert from "node:assert/strict";
import test from "node:test";
import { initialOneShotState, oneShotReducer } from "./oneShotMachine";

test("new question clears previous one-shot content", () => {
  const answered = {
    ...initialOneShotState,
    open: true,
    phase: "answered" as const,
    question: "old",
    answer: "old answer",
    requestId: "id",
  };
  const next = oneShotReducer(answered, { type: "new" });
  assert.equal(next.question, "");
  assert.equal(next.answer, "");
  assert.equal(next.requestId, undefined);
});

test("ignores a stale response", () => {
  const pending = oneShotReducer(
    { ...initialOneShotState, question: "q" },
    { type: "submit", requestId: "current" },
  );
  assert.equal(
    oneShotReducer(pending, { type: "answer", requestId: "stale", value: "x" }),
    pending,
  );
});
