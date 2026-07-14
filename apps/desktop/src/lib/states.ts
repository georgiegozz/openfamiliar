export type MascotState =
  | "idle"
  | "listening"
  | "thinking"
  | "working"
  | "waiting_approval"
  | "success"
  | "error"
  | "sleeping"
  | "offline";

export const MASCOT_STATES: MascotState[] = [
  "idle",
  "listening",
  "thinking",
  "working",
  "waiting_approval",
  "success",
  "error",
  "sleeping",
  "offline",
];

export function nextDemoState(current: MascotState): MascotState {
  const order: MascotState[] = [
    "idle",
    "thinking",
    "working",
    "success",
    "idle",
  ];
  const idx = order.indexOf(current);
  if (idx === -1) return "idle";
  return order[(idx + 1) % order.length]!;
}
