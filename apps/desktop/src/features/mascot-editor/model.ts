export const REQUIRED_STATES = [
  "idle",
  "thinking",
  "working",
  "approval",
  "success",
  "error",
] as const;

export type EditorState = (typeof REQUIRED_STATES)[number];

export const SHEET_SIZE = 256;
export const CELL_SIZE = 64;
export const SHEET_COLUMNS = 4;
export const SHEET_ROWS = 4;
export const MAX_FRAME = SHEET_COLUMNS * SHEET_ROWS - 1;
export const MIN_EDITOR_FPS = 1;
export const MAX_EDITOR_FPS = 12;
export const MAX_PERSONALITY_CHARS = 8192;

export const ALLOWED_LICENSES = [
  "CC0-1.0",
  "CC-BY-4.0",
  "Apache-2.0",
  "NOASSERTION",
] as const;

export type EditorLicense = (typeof ALLOWED_LICENSES)[number];

export const SHORT_PALETTE = [
  "#000000",
  "#ffffff",
  "#7a4a32",
  "#c48a5a",
  "#f0d2b4",
  "#3d6b4f",
  "#7c9c6b",
  "#2f5d8a",
  "#6fa0c8",
  "#8b2d2d",
  "#d98a3a",
  "#6b4c9a",
  "#4a4a4a",
  "#9aa0a6",
  "#e8c84a",
  "#2a2a2a",
] as const;

export interface EditorAnimation {
  frames: number[];
  fps: number;
  loop: boolean;
}

export function clampFps(value: number): number {
  if (!Number.isFinite(value)) return MIN_EDITOR_FPS;
  return Math.min(MAX_EDITOR_FPS, Math.max(MIN_EDITOR_FPS, Math.round(value)));
}

export function cellIndex(column: number, row: number): number {
  return row * SHEET_COLUMNS + column;
}

export function cellOrigin(frame: number): { x: number; y: number } {
  const index = Math.max(0, Math.min(MAX_FRAME, frame));
  return {
    x: (index % SHEET_COLUMNS) * CELL_SIZE,
    y: Math.floor(index / SHEET_COLUMNS) * CELL_SIZE,
  };
}

export function defaultAnimations(
  fps: number,
): Record<EditorState, EditorAnimation> {
  const clamped = clampFps(fps);
  return Object.fromEntries(
    REQUIRED_STATES.map((state) => [
      state,
      {
        frames: [0],
        fps: clamped,
        loop: state !== "success" && state !== "error",
      },
    ]),
  ) as Record<EditorState, EditorAnimation>;
}

export function assignFrame(
  animations: Record<EditorState, EditorAnimation>,
  state: EditorState,
  frame: number,
  replace = false,
): Record<EditorState, EditorAnimation> {
  if (frame < 0 || frame > MAX_FRAME) return animations;
  const current = animations[state];
  const frames = replace
    ? [frame]
    : current.frames.includes(frame)
      ? current.frames
      : [...current.frames, frame];
  return {
    ...animations,
    [state]: { ...current, frames },
  };
}

export function isAllowedLicense(value: string): value is EditorLicense {
  return (ALLOWED_LICENSES as readonly string[]).includes(value);
}
