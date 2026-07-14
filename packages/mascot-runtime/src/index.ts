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
    frameWidth: 64,
    frameHeight: 64,
    columns: 4,
    rows: 4,
  },
  animations: {
    idle: {
      frames: [0],
      fps: 1,
      loop: true,
      interruptible: true,
      priority: 0,
    },
    thinking: {
      frames: [2, 9],
      fps: 2,
      loop: true,
      interruptible: true,
      priority: 3,
    },
    answering: {
      frames: [15],
      fps: 1,
      loop: true,
      interruptible: true,
      priority: 3,
    },
    approval: {
      frames: [3],
      fps: 1,
      loop: true,
      interruptible: true,
      priority: 3,
    },
    success: {
      frames: [10, 0],
      fps: 3,
      loop: false,
      interruptible: false,
      priority: 5,
      fallback: "idle",
    },
    error: {
      frames: [5, 0],
      fps: 2,
      loop: false,
      interruptible: false,
      priority: 5,
      fallback: "idle",
    },
    sleeping: {
      frames: [6],
      fps: 1,
      loop: true,
      interruptible: true,
      priority: 2,
    },
    wake: {
      frames: [7, 0],
      fps: 3,
      loop: false,
      interruptible: false,
      priority: 4,
      fallback: "idle",
    },
    dragging: {
      frames: [8],
      fps: 1,
      loop: true,
      interruptible: true,
      priority: 4,
    },
    listening: {
      frames: [1],
      fps: 1,
      loop: true,
      interruptible: true,
      priority: 2,
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
