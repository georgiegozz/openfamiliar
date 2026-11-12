import {
  MascotRuntime,
  type FrameSnapshot,
  type MascotRuntimeManifest,
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
  manifest?: MascotRuntimeManifest,
): FrameSnapshot {
  const runtimeRef = useRef(new MascotRuntime(manifest));
  const manifestId = manifest?.id ?? "perrito-tech";
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
    runtimeRef.current = new MascotRuntime(manifest);
    publish(runtimeRef.current.playState(visualState, true));
  }, [manifest, manifestId, publish, visualState]);

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
    manifestId,
  ]);

  useEffect(() => {
    const handleVisibility = () => setDocumentVisible(!document.hidden);
    document.addEventListener("visibilitychange", handleVisibility);
    return () =>
      document.removeEventListener("visibilitychange", handleVisibility);
  }, []);

  return snapshot;
}
