import type {
  AnimationDefinition,
  MascotRuntimeManifest,
  MascotVisualState,
} from "@openfamiliar/mascot-runtime";
import type { LocalMascotRuntime } from "../../lib/types";

const STATES: MascotVisualState[] = [
  "idle",
  "listening",
  "thinking",
  "answering",
  "success",
  "error",
  "sleeping",
  "wake",
  "dragging",
];

const STATE_ALIASES: Record<string, string[]> = {
  thinking: ["thinking", "working"],
  answering: ["answering", "working"],
  success: ["success", "approval"],
  listening: ["listening", "idle"],
  sleeping: ["sleeping", "idle"],
  wake: ["wake", "idle"],
  dragging: ["dragging", "idle"],
};

function pickAnimation(
  animations: LocalMascotRuntime["animations"],
  name: string,
): AnimationDefinition {
  const aliases = STATE_ALIASES[name] ?? [name, "idle"];
  let source = animations[name] ?? animations.idle;
  for (const alias of aliases) {
    if (animations[alias]) {
      source = animations[alias];
      break;
    }
  }
  source ??= { frames: [0], fps: 1, loop: true };
  return {
    frames: source.frames.length > 0 ? source.frames : [0],
    fps: source.fps > 0 ? source.fps : 1,
    loop: source.loop,
    interruptible: true,
    priority: 0,
  };
}

export function toRuntimeManifest(
  dto: LocalMascotRuntime,
): MascotRuntimeManifest {
  const animations: Record<string, AnimationDefinition> = {};
  for (const state of STATES) {
    animations[state] = pickAnimation(dto.animations, state);
  }
  for (const [name, def] of Object.entries(dto.animations)) {
    if (!animations[name]) {
      animations[name] = pickAnimation(
        { [name]: def, ...dto.animations },
        name,
      );
    }
  }
  const states = Object.fromEntries(
    STATES.map((state) => [state, animations[state] ? state : "idle"]),
  ) as MascotRuntimeManifest["states"];
  return {
    id: dto.id,
    sheet: {
      asset: "assets/spritesheet.png",
      frameWidth: dto.frameWidth,
      frameHeight: dto.frameHeight,
      columns: dto.columns,
      rows: dto.rows,
    },
    animations,
    states,
  };
}

export function sheetDataUrl(base64: string): string {
  return `data:image/png;base64,${base64}`;
}
