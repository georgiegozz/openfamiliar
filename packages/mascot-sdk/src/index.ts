export type MascotStateName =
  | "idle"
  | "thinking"
  | "working"
  | "approval"
  | "success"
  | "error"
  | "listening"
  | "waiting_approval"
  | "sleeping"
  | "offline";

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
}

export interface LoadedPack {
  root: string;
  manifest: FamiliarManifest;
  personalityText: string;
}

export function resolveStateAsset(manifest: FamiliarManifest, state: MascotStateName): string | undefined {
  if (state === "waiting_approval") {
    return manifest.states["approval"] ?? manifest.states["waiting_approval"];
  }
  return manifest.states[state];
}

export function assertNoTraversal(relPath: string): void {
  if (relPath.includes("..") || relPath.startsWith("/") || /^[A-Za-z]:/.test(relPath)) {
    throw new Error(`invalid pack path: ${relPath}`);
  }
}