import {
  preloadSpriteSheet,
  type MascotVisualState,
} from "@openfamiliar/mascot-runtime";
import { useEffect } from "react";
import spriteSheetUrl from "../../../../../mascots/perrito-tech/assets/perrito-tech-spritesheet.png?url";
import { useMascotRuntime } from "./useMascotRuntime";

interface MascotSpriteProps {
  state: MascotVisualState;
  scale: 1 | 2 | 3;
  animationsEnabled: boolean;
  reduceMotion: boolean;
  onActivity?: () => void;
}

export function MascotSprite({
  state,
  scale,
  animationsEnabled,
  reduceMotion,
  onActivity,
}: MascotSpriteProps) {
  useEffect(() => {
    void preloadSpriteSheet(spriteSheetUrl).catch(() => undefined);
  }, []);
  const { snapshot, markActivity } = useMascotRuntime(state, {
    enabled: animationsEnabled,
    reduceMotion,
  });
  const column = snapshot.frame % 4;
  const row = Math.floor(snapshot.frame / 4);

  return (
    <div
      className="mascot-sprite-viewport"
      data-animation={snapshot.animation}
      style={{ width: 96 * scale, height: 96 * scale }}
      onPointerEnter={() => {
        markActivity();
        onActivity?.();
      }}
      aria-label={`Perrito Tech: ${state}`}
      role="img"
    >
      <div
        className="mascot-sprite-frame"
        style={{
          backgroundImage: `url(${spriteSheetUrl})`,
          backgroundPosition: `${-column * 96}px ${-row * 96}px`,
          transform: `scale(${scale})`,
        }}
      />
    </div>
  );
}
