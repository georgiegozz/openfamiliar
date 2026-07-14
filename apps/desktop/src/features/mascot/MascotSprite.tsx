import {
  preloadSpriteSheet,
  type MascotVisualState,
} from "@openfamiliar/mascot-runtime";
import { useEffect } from "react";
import spriteSheetUrl from "../../../../../mascots/perrito-tech/assets/perrito-tech-spritesheet.png?url";
import burgundySpriteSheetUrl from "../../../../../mascots/perrito-tech/assets/variants/perrito-tech-burgundy.png?url";
import midnightSpriteSheetUrl from "../../../../../mascots/perrito-tech/assets/variants/perrito-tech-midnight.png?url";
import type { MascotPalette } from "../../lib/types";
import { useMascotRuntime } from "./useMascotRuntime";

export const MASCOT_FRAME_SIZE = 64;

interface MascotSpriteProps {
  state: MascotVisualState;
  scale: 1 | 2 | 3;
  palette: MascotPalette;
  animationsEnabled: boolean;
  reduceMotion: boolean;
}

export function MascotSprite({
  state,
  scale,
  palette,
  animationsEnabled,
  reduceMotion,
}: MascotSpriteProps) {
  const selectedSheet = {
    teal: spriteSheetUrl,
    midnight: midnightSpriteSheetUrl,
    burgundy: burgundySpriteSheetUrl,
  }[palette];
  useEffect(() => {
    void preloadSpriteSheet(selectedSheet).catch(() => undefined);
  }, [selectedSheet]);
  const snapshot = useMascotRuntime(state, {
    enabled: animationsEnabled,
    reduceMotion,
  });
  const column = snapshot.frame % 4;
  const row = Math.floor(snapshot.frame / 4);

  return (
    <div
      className="mascot-sprite-viewport"
      data-animation={snapshot.animation}
      style={{
        width: MASCOT_FRAME_SIZE * scale,
        height: MASCOT_FRAME_SIZE * scale,
      }}
      aria-label={`Perrito Tech: ${state}`}
      role="img"
    >
      <div
        className="mascot-sprite-frame"
        style={{
          backgroundImage: `url(${selectedSheet})`,
          backgroundPosition: `${-column * MASCOT_FRAME_SIZE}px ${-row * MASCOT_FRAME_SIZE}px`,
          transform: `scale(${scale})`,
        }}
      />
    </div>
  );
}
