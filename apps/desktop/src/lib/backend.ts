import type {
  AppPreferences,
  BackendError,
  OneShotRequest,
  OneShotResult,
  ProviderStatus,
  SavedPosition,
} from "./types";
import { DEFAULT_PREFERENCES } from "./types";

/** Narrow IPC surface: no generic shell, workspace, provider, or agent commands. */

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function invokeBackend<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }
  return mockInvoke<T>(cmd, args);
}

export const backend = {
  detectCodex: () => invokeBackend<ProviderStatus>("detect_codex"),
  askCodex: (request: OneShotRequest) =>
    invokeBackend<OneShotResult>("ask_codex", { request }),
  cancelCodex: (requestId: string) =>
    invokeBackend<void>("cancel_codex", { requestId }),
  getPreferences: () => invokeBackend<AppPreferences>("get_preferences"),
  updatePreferences: (preferences: AppPreferences) =>
    invokeBackend<AppPreferences>("update_preferences", { preferences }),
  openSettings: () => invokeBackend<void>("open_settings"),
  openQuickAsk: () => invokeBackend<void>("open_quick_ask"),
  setClickThrough: (enabled: boolean) =>
    invokeBackend<AppPreferences>("set_click_through", { enabled }),
  setAlwaysOnTop: (enabled: boolean) =>
    invokeBackend<AppPreferences>("set_always_on_top", { enabled }),
  saveMascotPosition: (position: SavedPosition) =>
    invokeBackend<AppPreferences>("save_mascot_position", { position }),
  resetMascotPosition: () =>
    invokeBackend<AppPreferences>("reset_mascot_position"),
  quit: () => invokeBackend<void>("quit_app"),
};

export function normalizeBackendError(error: unknown): BackendError {
  if (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    "message" in error
  ) {
    return error as BackendError;
  }
  return {
    kind: "unknown",
    message: error instanceof Error ? error.message : String(error),
  };
}

async function mockInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  switch (cmd) {
    case "open_settings":
    case "open_quick_ask":
    case "cancel_codex":
    case "quit_app":
      return undefined as T;
    case "set_click_through":
    case "set_always_on_top":
    case "save_mascot_position":
    case "reset_mascot_position":
    case "update_preferences":
      return (args?.preferences ?? DEFAULT_PREFERENCES) as T;
    case "get_preferences":
      return DEFAULT_PREFERENCES as T;
    case "detect_codex":
      return {
        installed: false,
        authenticated: false,
        compatible: false,
        message:
          "Browser preview: Codex CLI detection requires the Tauri build.",
      } as T;
    case "ask_codex":
      await new Promise((resolve) => setTimeout(resolve, 250));
      return {
        requestId: String(
          (args?.request as OneShotRequest | undefined)?.requestId ?? "preview",
        ),
        answer: "Browser preview only. Run the Tauri build to use Codex CLI.",
      } as T;
    default:
      throw new Error(`Unknown command in web mock: ${cmd}`);
  }
}
