import {
  MascotRuntime,
  type FrameSnapshot,
  type MascotVisualState,
} from "@openfamiliar/mascot-runtime";
import { useCallback, useEffect, useRef, useState } from "react";

interface RuntimeOptions {
  enabled: boolean;
  reduceMotion: boolean;
}

export function useMascotRuntime(
  visualState: MascotVisualState,
  options: RuntimeOptions,
): FrameSnapshot {
  const runtimeRef = useRef(new MascotRuntime());
  const [documentVisible, setDocumentVisible] = useState(
    () => !document.hidden,
  );
  const [snapshot, setSnapshot] = useState(() => runtimeRef.current.current());

  const publish = useCallback((next: FrameSnapshot) => {
    setSnapshot((current) =>
      current.animation === next.animation && current.frame === next.frame
        ? current
        : next,
    );
  }, []);

  useEffect(() => {
    publish(runtimeRef.current.playState(visualState, true));
  }, [publish, visualState]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    const definition = runtime.manifest.animations[snapshot.animation];
    if (!options.enabled || options.reduceMotion || !documentVisible) {
      runtime.pause();
      publish(runtime.current());
      return;
    }
    if (!definition || definition.frames.length <= 1) {
      runtime.pause();
      return;
    }

    runtime.resume();
    let lastTick = performance.now();
    const interval = window.setInterval(() => {
      const now = performance.now();
      publish(runtime.tick(Math.min(250, now - lastTick)));
      lastTick = now;
    }, 100);
    return () => window.clearInterval(interval);
  }, [
    documentVisible,
    options.enabled,
    options.reduceMotion,
    publish,
    snapshot.animation,
  ]);

  useEffect(() => {
    const handleVisibility = () => setDocumentVisible(!document.hidden);
    document.addEventListener("visibilitychange", handleVisibility);
    return () =>
      document.removeEventListener("visibilitychange", handleVisibility);
  }, []);

  return snapshot;
}
