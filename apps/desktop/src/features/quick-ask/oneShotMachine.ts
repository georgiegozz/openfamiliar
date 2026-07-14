export type OneShotPhase =
  "idle" | "editing" | "submitting" | "answered" | "error";

export interface OneShotState {
  phase: OneShotPhase;
  open: boolean;
  question: string;
  answer: string;
  error: string;
  requestId?: string;
}

export type OneShotAction =
  | { type: "open" }
  | { type: "close" }
  | { type: "edit"; value: string }
  | { type: "submit"; requestId: string }
  | { type: "answer"; requestId: string; value: string }
  | { type: "fail"; requestId?: string; value: string }
  | { type: "new" };

export const initialOneShotState: OneShotState = {
  phase: "idle",
  open: false,
  question: "",
  answer: "",
  error: "",
};

export function oneShotReducer(
  state: OneShotState,
  action: OneShotAction,
): OneShotState {
  switch (action.type) {
    case "open":
      return {
        ...state,
        open: true,
        phase: state.phase === "idle" ? "editing" : state.phase,
      };
    case "close":
      return initialOneShotState;
    case "edit":
      return {
        ...state,
        open: true,
        phase: "editing",
        question: action.value,
        answer: "",
        error: "",
        requestId: undefined,
      };
    case "submit":
      return {
        ...state,
        open: true,
        phase: "submitting",
        answer: "",
        error: "",
        requestId: action.requestId,
      };
    case "answer":
      if (state.requestId !== action.requestId) return state;
      return { ...state, phase: "answered", answer: action.value, error: "" };
    case "fail":
      if (
        action.requestId &&
        state.requestId &&
        action.requestId !== state.requestId
      )
        return state;
      return { ...state, phase: "error", answer: "", error: action.value };
    case "new":
      return { ...initialOneShotState, open: true, phase: "editing" };
  }
}
