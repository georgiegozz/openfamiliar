export type MascotStateName =
  | "idle"
  | "listening"
  | "thinking"
  | "answering"
  | "working"
  | "approval"
  | "success"
  | "error"
  | "waiting_approval"
  | "sleeping"
  | "wake"
  | "dragging"
  | "offline";

export interface SpriteSheetDefinition {
  asset: string;
  frameWidth: number;
  frameHeight: number;
  columns: number;
  rows: number;
}

export interface SpriteAnimationDefinition {
  frames: number[];
  fps: number;
  loop: boolean;
  interruptible: boolean;
  priority: number;
  fallback?: string;
}

export interface FamiliarManifest {
  id: string;
  name: string;
  version: string;
  engine: string;
  author: string;
  license: string;
  homepage?: string;
  personality: string;
  states: Record<string, string>;
  assetSources?: string[];
  aiGenerated?: boolean;
  spriteSheet?: SpriteSheetDefinition;
  variants?: Record<string, string>;
  animations?: Record<string, SpriteAnimationDefinition>;
}

export interface LoadedPack {
  root: string;
  manifest: FamiliarManifest;
  personalityText: string;
}

export function resolveStateAsset(
  manifest: FamiliarManifest,
  state: MascotStateName,
): string | undefined {
  if (state === "waiting_approval") {
    return manifest.states["approval"] ?? manifest.states["waiting_approval"];
  }
  return manifest.states[state];
}

export function assertNoTraversal(relPath: string): void {
  if (
    relPath.includes("..") ||
    relPath.startsWith("/") ||
    /^[A-Za-z]:/.test(relPath)
  ) {
    throw new Error(`invalid pack path: ${relPath}`);
  }
}
