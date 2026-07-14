export type MascotVisualState =
  | "idle"
  | "listening"
  | "thinking"
  | "answering"
  | "success"
  | "error"
  | "sleeping"
  | "wake"
  | "dragging";

export type AmbientAction =
  | "breathe"
  | "blink"
  | "look-left"
  | "look-right"
  | "ear-twitch"
  | "special-idle";

export interface SpriteSheetManifest {
  asset: string;
  frameWidth: number;
  frameHeight: number;
  columns: number;
  rows: number;
}

export interface AnimationDefinition {
  frames: readonly number[];
  fps: number;
  loop: boolean;
  interruptible: boolean;
  priority: number;
  fallback?: string;
}

export interface MascotRuntimeManifest {
  id: string;
  sheet: SpriteSheetManifest;
  animations: Readonly<Record<string, AnimationDefinition>>;
  states: Readonly<Record<MascotVisualState, string>>;
  ambient: Readonly<Record<AmbientAction, string>>;
  sleepAfterMs: number;
}

export interface FrameSnapshot {
  animation: string;
  frame: number;
  completed: boolean;
}

export const PERRITO_TECH_RUNTIME: MascotRuntimeManifest = {
  id: "perrito-tech",
  sheet: {
    asset: "assets/perrito-tech-spritesheet.png",
    frameWidth: 96,
    frameHeight: 96,
    columns: 4,
    rows: 4,
  },
  animations: {
    idle: {
      frames: [0, 1],
      fps: 6,
      loop: true,
      interruptible: true,
      priority: 0,
    },
    breathe: {
      frames: [0, 1, 0],
      fps: 6,
      loop: false,
      interruptible: true,
      priority: 1,
    },
    blink: {
      frames: [0, 2, 0],
      fps: 8,
      loop: false,
      interruptible: true,
      priority: 1,
    },
    "look-left": {
      frames: [0, 3, 3, 0],
      fps: 6,
      loop: false,
      interruptible: true,
      priority: 1,
    },
    "look-right": {
      frames: [0, 4, 4, 0],
      fps: 6,
      loop: false,
      interruptible: true,
      priority: 1,
    },
    thinking: {
      frames: [5, 0, 5],
      fps: 6,
      loop: true,
      interruptible: true,
      priority: 3,
    },
    answering: {
      frames: [6, 1, 6],
      fps: 8,
      loop: true,
      interruptible: true,
      priority: 3,
    },
    success: {
      frames: [7, 15, 7],
      fps: 8,
      loop: false,
      interruptible: false,
      priority: 5,
      fallback: "idle",
    },
    error: {
      frames: [8, 8, 0],
      fps: 6,
      loop: false,
      interruptible: false,
      priority: 5,
      fallback: "idle",
    },
    sleeping: {
      frames: [9],
      fps: 2,
      loop: true,
      interruptible: true,
      priority: 2,
    },
    wake: {
      frames: [10, 0],
      fps: 8,
      loop: false,
      interruptible: false,
      priority: 4,
      fallback: "idle",
    },
    dragging: {
      frames: [11],
      fps: 8,
      loop: true,
      interruptible: true,
      priority: 4,
    },
    listening: {
      frames: [12, 0],
      fps: 6,
      loop: true,
      interruptible: true,
      priority: 2,
    },
    "ear-twitch": {
      frames: [13, 0],
      fps: 8,
      loop: false,
      interruptible: true,
      priority: 1,
    },
    "special-idle": {
      frames: [14, 0],
      fps: 6,
      loop: false,
      interruptible: true,
      priority: 1,
    },
  },
  states: {
    idle: "idle",
    listening: "listening",
    thinking: "thinking",
    answering: "answering",
    success: "success",
    error: "error",
    sleeping: "sleeping",
    wake: "wake",
    dragging: "dragging",
  },
  ambient: {
    breathe: "breathe",
    blink: "blink",
    "look-left": "look-left",
    "look-right": "look-right",
    "ear-twitch": "ear-twitch",
    "special-idle": "special-idle",
  },
  sleepAfterMs: 5 * 60_000,
};

export class MascotRuntime {
  readonly manifest: MascotRuntimeManifest;
  private animation = "idle";
  private frameIndex = 0;
  private elapsedMs = 0;
  private paused = false;

  constructor(manifest: MascotRuntimeManifest = PERRITO_TECH_RUNTIME) {
    this.manifest = manifest;
  }

  play(animation: string, force = false): FrameSnapshot {
    const next = this.definition(animation);
    const current = this.definition(this.animation);
    const canInterrupt =
      current.interruptible || next.priority >= current.priority;
    if (!force && !canInterrupt) return this.snapshot(false);
    this.animation = animation in this.manifest.animations ? animation : "idle";
    this.frameIndex = 0;
    this.elapsedMs = 0;
    return this.snapshot(false);
  }

  playState(state: MascotVisualState, force = false): FrameSnapshot {
    return this.play(this.manifest.states[state] ?? "idle", force);
  }

  pause(): void {
    this.paused = true;
  }

  resume(): void {
    this.paused = false;
  }

  tick(deltaMs: number): FrameSnapshot {
    if (this.paused || deltaMs <= 0) return this.snapshot(false);
    const definition = this.definition(this.animation);
    const frameDuration = 1_000 / Math.min(12, Math.max(1, definition.fps));
    this.elapsedMs += deltaMs;
    let completed = false;
    while (this.elapsedMs >= frameDuration) {
      this.elapsedMs -= frameDuration;
      this.frameIndex += 1;
      if (this.frameIndex >= definition.frames.length) {
        completed = true;
        if (definition.loop) {
          this.frameIndex = 0;
        } else if (definition.fallback) {
          this.animation = definition.fallback;
          this.frameIndex = 0;
          this.elapsedMs = 0;
          break;
        } else {
          this.frameIndex = definition.frames.length - 1;
          break;
        }
      }
    }
    return this.snapshot(completed);
  }

  current(): FrameSnapshot {
    return this.snapshot(false);
  }

  private definition(animation: string): AnimationDefinition {
    return (
      this.manifest.animations[animation] ?? this.manifest.animations.idle!
    );
  }

  private snapshot(completed: boolean): FrameSnapshot {
    const definition = this.definition(this.animation);
    return {
      animation: this.animation,
      frame: definition.frames[this.frameIndex] ?? definition.frames[0] ?? 0,
      completed,
    };
  }
}

export function chooseAmbientAction(randomValue: number): AmbientAction {
  const actions: AmbientAction[] = [
    "breathe",
    "blink",
    "look-left",
    "look-right",
    "ear-twitch",
    "special-idle",
  ];
  const index = Math.min(
    actions.length - 1,
    Math.max(0, Math.floor(randomValue * actions.length)),
  );
  return actions[index]!;
}

export function nextAmbientDelay(
  randomValue: number,
  minimumMs = 4_000,
  spreadMs = 8_000,
): number {
  const normalized = Math.min(1, Math.max(0, randomValue));
  return Math.round(minimumMs + normalized * spreadMs);
}

export function preloadSpriteSheet(assetUrl: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve();
    image.onerror = () =>
      reject(new Error("Mascot spritesheet could not be preloaded."));
    image.src = assetUrl;
  });
}
