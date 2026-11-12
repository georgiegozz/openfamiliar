import {
  preloadSpriteSheet,
  type MascotRuntimeManifest,
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
  displayName?: string;
  customSheetUrl?: string;
  customManifest?: MascotRuntimeManifest;
}

export function MascotSprite({
  state,
  scale,
  palette,
  animationsEnabled,
  reduceMotion,
  displayName,
  customSheetUrl,
  customManifest,
}: MascotSpriteProps) {
  const bundledSheet = {
    teal: spriteSheetUrl,
    midnight: midnightSpriteSheetUrl,
    burgundy: burgundySpriteSheetUrl,
  }[palette];
  const selectedSheet = customSheetUrl ?? bundledSheet;
  const frameWidth = customManifest?.sheet.frameWidth ?? MASCOT_FRAME_SIZE;
  const frameHeight = customManifest?.sheet.frameHeight ?? MASCOT_FRAME_SIZE;
  const columns = customManifest?.sheet.columns ?? 4;
  useEffect(() => {
    void preloadSpriteSheet(selectedSheet).catch(() => undefined);
  }, [selectedSheet]);
  const snapshot = useMascotRuntime(
    state,
    {
      enabled: animationsEnabled,
      reduceMotion,
    },
    customManifest,
  );
  const column = snapshot.frame % columns;
  const row = Math.floor(snapshot.frame / columns);
  const label = displayName ?? "Perrito Tech";

  return (
    <div
      className="mascot-sprite-viewport"
      data-animation={snapshot.animation}
      style={{
        width: frameWidth * scale,
        height: frameHeight * scale,
      }}
      aria-label={`${label}: ${state}`}
      role="img"
    >
      <div
        className="mascot-sprite-frame"
        style={{
          backgroundImage: `url(${selectedSheet})`,
          backgroundPosition: `${-column * frameWidth}px ${-row * frameHeight}px`,
          backgroundSize: `${columns * frameWidth}px ${(customManifest?.sheet.rows ?? 4) * frameHeight}px`,
          transform: `scale(${scale})`,
          width: frameWidth,
          height: frameHeight,
        }}
      />
    </div>
  );
}
