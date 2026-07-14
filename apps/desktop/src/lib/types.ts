import type {
  OneShotRequest,
  OneShotResult,
  ProviderStatus,
} from "@openfamiliar/provider-sdk";

export type { OneShotRequest, OneShotResult, ProviderStatus };

export interface SavedPosition {
  x: number;
  y: number;
  monitorName?: string;
  scaleFactor: number;
}

export interface AppPreferences {
  mascotPosition?: SavedPosition;
  scale: 1 | 2 | 3;
  alwaysOnTop: boolean;
  clickThrough: boolean;
  animationsEnabled: boolean;
  reduceMotion: boolean;
  language: "es-MX" | "en-US";
  launchAtStartup: boolean;
  codexPath?: string;
  timeoutSeconds: number;
}

export interface BackendError {
  kind:
    | "codex_not_installed"
    | "not_authenticated"
    | "incompatible_version"
    | "rate_limit"
    | "timeout"
    | "cancelled"
    | "invalid_output"
    | "output_too_large"
    | "process_failed"
    | "invalid_request"
    | "unknown";
  message: string;
}

export const DEFAULT_PREFERENCES: AppPreferences = {
  scale: 2,
  alwaysOnTop: true,
  clickThrough: false,
  animationsEnabled: true,
  reduceMotion: false,
  language: "es-MX",
  launchAtStartup: false,
  timeoutSeconds: 120,
};
