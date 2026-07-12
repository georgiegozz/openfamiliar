/** Thin IPC layer: Tauri when available, browser mock otherwise. */

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Frontend uses camelCase payloads. Rust exposes both direct args and `*_args`
 * commands; we prefer the `*_args` variants for object payloads.
 */
const CMD_MAP: Record<string, string> = {
  set_mascot_state: "set_mascot_state_v2",
  chat: "chat_args",
  authorize_workspace: "authorize_workspace_args",
  preview_workspace: "preview_workspace_args",
  set_security_mode: "set_security_mode_args",
  set_click_through: "set_click_through_args",
};

export async function invokeBackend<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    const mapped = CMD_MAP[cmd] ?? cmd;
    if (mapped.endsWith("_args") || mapped.endsWith("_v2")) {
      return invoke<T>(mapped, { args });
    }
    return invoke<T>(mapped, args);
  }
  return mockInvoke<T>(cmd, args);
}

async function mockInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  switch (cmd) {
    case "set_mascot_state":
    case "set_click_through":
    case "set_security_mode":
      return undefined as T;
    case "chat": {
      const message = String(args?.message ?? "");
      await new Promise((r) => setTimeout(r, 200));
      return `Perrito Tech (web-mock): recibí «${message}». Arranca con Tauri para providers reales.` as T;
    }
    case "authorize_workspace":
      return { ok: true } as T;
    case "preview_workspace":
      return "Preview mock: selecciona archivos tras autorizar en build Tauri." as T;
    default:
      throw new Error(`Unknown command in web mock: ${cmd}`);
  }
}
