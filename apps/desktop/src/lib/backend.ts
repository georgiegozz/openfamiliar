import type {
  AppPreferences,
  BackendError,
  EditorMascot,
  EditorSaveRequest,
  LocalMascotInfo,
  LocalMascotRuntime,
  OneShotRequest,
  OneShotResult,
  ProviderSecretStatus,
  ProviderStatus,
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
  saveMascotPosition: () =>
    invokeBackend<AppPreferences>("save_mascot_position"),
  setMascotExpanded: (expanded: boolean) =>
    invokeBackend<void>("set_mascot_expanded", { expanded }),
  resetMascotPosition: () =>
    invokeBackend<AppPreferences>("reset_mascot_position"),
  quit: () => invokeBackend<void>("quit_app"),
  importMascotImage: (request: { name: string; data: number[] }) =>
    invokeBackend<LocalMascotInfo>("import_mascot_image", { request }),
  listLocalMascots: () =>
    invokeBackend<LocalMascotInfo[]>("list_local_mascots"),
  getMascotSheetPng: (id: string) =>
    invokeBackend<string | null>("get_mascot_sheet_png", { id }),
  getLocalMascotRuntime: (id: string) =>
    invokeBackend<LocalMascotRuntime>("get_local_mascot_runtime", { id }),
  setActiveMascot: (id: string) =>
    invokeBackend<AppPreferences>("set_active_mascot", { id }),
  saveEditorMascot: (request: EditorSaveRequest) =>
    invokeBackend<LocalMascotInfo>("save_editor_mascot", { request }),
  getEditorMascot: (id: string) =>
    invokeBackend<EditorMascot>("get_editor_mascot", { id }),
  setProviderSecret: (providerId: string, secret: string) =>
    invokeBackend<ProviderSecretStatus>("set_provider_secret", {
      request: { providerId, secret },
    }),
  providerSecretConfigured: (providerId: string) =>
    invokeBackend<ProviderSecretStatus>("provider_secret_configured", {
      providerId,
    }),
  deleteProviderSecret: (providerId: string) =>
    invokeBackend<ProviderSecretStatus>("delete_provider_secret", {
      providerId,
    }),
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
    case "set_mascot_expanded":
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
    case "list_local_mascots":
      return [] as T;
    case "import_mascot_image":
      return { id: "preview-mascot", name: "Preview" } as T;
    case "get_mascot_sheet_png":
      return null as T;
    case "get_local_mascot_runtime":
      throw new Error("Browser preview has no local mascot runtime.");
    case "set_active_mascot":
      return {
        ...DEFAULT_PREFERENCES,
        activeMascotId: String(args?.id ?? "perrito-tech"),
      } as T;
    case "save_editor_mascot":
      return { id: "preview-mascot", name: "Preview" } as T;
    case "get_editor_mascot":
      throw new Error("Browser preview has no editor pack.");
    case "set_provider_secret":
    case "provider_secret_configured":
    case "delete_provider_secret":
      return { providerId: "anthropic", configured: false } as T;
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
