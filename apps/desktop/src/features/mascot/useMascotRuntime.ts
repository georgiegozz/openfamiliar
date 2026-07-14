import {
  MascotRuntime,
  chooseAmbientAction,
  nextAmbientDelay,
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
): { snapshot: FrameSnapshot; markActivity: () => void } {
  const runtimeRef = useRef(new MascotRuntime());
  const visualStateRef = useRef(visualState);
  const lastActivityRef = useRef(Date.now());
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

  const markActivity = useCallback(() => {
    lastActivityRef.current = Date.now();
    if (visualStateRef.current === "idle") {
      const current = runtimeRef.current.current();
      publish(
        current.animation === "sleeping"
          ? runtimeRef.current.play("wake", true)
          : runtimeRef.current.playState("idle", true),
      );
    }
  }, [publish]);

  useEffect(() => {
    visualStateRef.current = visualState;
    lastActivityRef.current = Date.now();
    publish(runtimeRef.current.playState(visualState, true));
  }, [publish, visualState]);

  useEffect(() => {
    const runtime = runtimeRef.current;
    if (!options.enabled || options.reduceMotion || !documentVisible) {
      runtime.pause();
      publish(runtime.current());
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
  }, [documentVisible, options.enabled, options.reduceMotion, publish]);

  useEffect(() => {
    const handleVisibility = () => setDocumentVisible(!document.hidden);
    document.addEventListener("visibilitychange", handleVisibility);
    return () =>
      document.removeEventListener("visibilitychange", handleVisibility);
  }, []);

  useEffect(() => {
    if (!options.enabled || options.reduceMotion || visualState !== "idle")
      return;
    let ambientTimer = 0;
    let sleepTimer = 0;

    const scheduleAmbient = () => {
      ambientTimer = window.setTimeout(() => {
        if (
          visualStateRef.current === "idle" &&
          Date.now() - lastActivityRef.current <
            runtimeRef.current.manifest.sleepAfterMs
        ) {
          publish(runtimeRef.current.play(chooseAmbientAction(Math.random())));
        }
        scheduleAmbient();
      }, nextAmbientDelay(Math.random()));
    };

    scheduleAmbient();
    sleepTimer = window.setInterval(() => {
      if (
        visualStateRef.current === "idle" &&
        Date.now() - lastActivityRef.current >=
          runtimeRef.current.manifest.sleepAfterMs
      ) {
        publish(runtimeRef.current.playState("sleeping", true));
      }
    }, 15_000);

    return () => {
      window.clearTimeout(ambientTimer);
      window.clearInterval(sleepTimer);
    };
  }, [options.enabled, options.reduceMotion, publish, visualState]);

  return { snapshot, markActivity };
}
