import { useEffect, useMemo, useState } from "react";
import { AvatarEditor } from "../features/mascot-editor/AvatarEditor";
import { backend, isTauri, normalizeBackendError } from "../lib/backend";
import {
  BUNDLED_MASCOT_ID,
  DEFAULT_PREFERENCES,
  type AppPreferences,
  type LocalMascotInfo,
  type ProviderId,
  type ProviderStatus,
} from "../lib/types";

const TABS = [
  "General",
  "Codex",
  "Proveedores",
  "Apariencia",
  "Comportamiento",
  "Privacidad",
  "Acerca de",
] as const;
type Tab = (typeof TABS)[number];

export function SettingsWindow() {
  const [tab, setTab] = useState<Tab>("General");
  const [preferences, setPreferences] =
    useState<AppPreferences>(DEFAULT_PREFERENCES);
  const [provider, setProvider] = useState<ProviderStatus>();
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [localMascots, setLocalMascots] = useState<LocalMascotInfo[]>([]);
  const [importError, setImportError] = useState("");
  const [secretDraft, setSecretDraft] = useState("");
  const [anthropicConfigured, setAnthropicConfigured] = useState(false);

  const providerLabel = useMemo(() => {
    if (!provider) return "Sin comprobar";
    if (!provider.installed) return "Proveedor no encontrado";
    if (!provider.compatible) return "Versión incompatible";
    if (!provider.authenticated) return "Inicio de sesión requerido";
    return `Disponible${provider.version ? ` · ${provider.version}` : ""}`;
  }, [provider]);

  useEffect(() => {
    void backend.getPreferences().then(setPreferences);
    void backend
      .listLocalMascots()
      .then(setLocalMascots)
      .catch(() => setLocalMascots([]));
    if (!isTauri()) {
      void backend.detectProvider().then(setProvider);
      return;
    }
    let disposed = false;
    const cleanups: Array<() => void> = [];
    void (async () => {
      const [{ listen }, { getCurrentWindow }] = await Promise.all([
        import("@tauri-apps/api/event"),
        import("@tauri-apps/api/window"),
      ]);
      if (disposed) return;
      cleanups.push(
        await listen<string>("settings:section", ({ payload }) => {
          if (payload === "about") setTab("Acerca de");
        }),
      );
      const currentWindow = getCurrentWindow();
      if (await currentWindow.isVisible()) {
        void backend.detectProvider().then(setProvider);
      }
      cleanups.push(
        await currentWindow.onFocusChanged(({ payload }) => {
          if (payload) void backend.detectProvider().then(setProvider);
        }),
      );
    })();
    return () => {
      disposed = true;
      for (const cleanup of cleanups) cleanup();
    };
  }, []);

  const update = <K extends keyof AppPreferences>(
    key: K,
    value: AppPreferences[K],
  ) => {
    setPreferences((current) => ({ ...current, [key]: value }));
    setNotice("");
  };

  const save = async () => {
    setBusy(true);
    try {
      setPreferences(await backend.updatePreferences(preferences));
      setNotice("Ajustes guardados.");
    } catch (error) {
      setNotice(normalizeBackendError(error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="settings-window">
      <aside>
        <div className="settings-brand">
          <span className="brand-mark">OF</span>
          <div>
            <strong>OpenFamiliar</strong>
            <small>Windows MVP 0.1</small>
          </div>
        </div>
        <nav>
          {TABS.map((item) => (
            <button
              className={tab === item ? "active" : ""}
              key={item}
              onClick={() => setTab(item)}
            >
              {item}
            </button>
          ))}
        </nav>
      </aside>

      <section className="settings-content">
        <header>
          <span className="eyebrow">Preferencias locales</span>
          <h1>{tab}</h1>
        </header>

        {tab === "General" ? (
          <div className="settings-section">
            <Toggle
              label="Iniciar con Windows"
              description="Registra OpenFamiliar para el usuario actual."
              checked={preferences.launchAtStartup}
              onChange={(value) => update("launchAtStartup", value)}
            />
            <label className="settings-field">
              <span>Idioma</span>
              <select
                value={preferences.language}
                onChange={(event) =>
                  update(
                    "language",
                    event.target.value as AppPreferences["language"],
                  )
                }
              >
                <option value="es-MX">Español (México)</option>
                <option value="en-US">English (United States)</option>
              </select>
            </label>
            <button
              className="secondary-button align-start"
              onClick={() => void backend.resetMascotPosition()}
            >
              Restablecer posición del familiar
            </button>
          </div>
        ) : null}

        {tab === "Codex" ? (
          <div className="settings-section">
            <div
              className={`status-card ${provider?.authenticated && provider.compatible ? "ok" : "warning"}`}
            >
              <strong>{providerLabel}</strong>
              <p>{provider?.message ?? "Comprobando la instalación local…"}</p>
            </div>
            <label className="settings-field">
              <span>Ruta opcional a codex.exe</span>
              <input
                value={preferences.codexPath ?? ""}
                placeholder="Vacío: buscar en PATH"
                onChange={(event) =>
                  update("codexPath", event.target.value || undefined)
                }
              />
            </label>
            <label className="settings-field">
              <span>Timeout por pregunta: {preferences.timeoutSeconds} s</span>
              <input
                type="range"
                min="10"
                max="300"
                step="10"
                value={preferences.timeoutSeconds}
                onChange={(event) =>
                  update("timeoutSeconds", Number(event.target.value))
                }
              />
            </label>
            <button
              className="secondary-button align-start"
              onClick={() => void backend.detectProvider().then(setProvider)}
            >
              Comprobar de nuevo
            </button>
          </div>
        ) : null}

        {tab === "Proveedores" ? (
          <div className="settings-section">
            <label className="settings-field">
              <span>Proveedor one-shot activo</span>
              <select
                value={preferences.activeProviderId}
                onChange={(event) =>
                  update("activeProviderId", event.target.value as ProviderId)
                }
              >
                <option value="codex">Codex CLI</option>
                <option value="anthropic">Anthropic (Claude)</option>
                <option value="opencode">OpenCode CLI</option>
                <option value="agy">Antigravity CLI</option>
              </select>
              <small>
                Codex, OpenCode y Antigravity usan la sesión de su propio CLI.
                OpenFamiliar no lee esas credenciales. Anthropic guarda la API
                key solo en el almacén de credenciales del sistema, nunca en
                preferences.json. El webview no hace fetch al proveedor.
                OpenCode corre `opencode run` en un directorio temporal vacío
                con un `opencode.json` que deniega herramientas (no hay flag
                oficial `--deny`; una regla global del operador podría
                prevalecer). Antigravity corre `agy -p --sandbox` y confía en el
                soft-deny headless; nunca se pasa
                `--dangerously-skip-permissions`.
              </small>
            </label>
            <label className="settings-field">
              <span>Modelo de Anthropic (no secreto)</span>
              <input
                value={preferences.anthropicModel}
                maxLength={80}
                onChange={(event) =>
                  update("anthropicModel", event.target.value)
                }
              />
              <small>
                Predeterminado: claude-sonnet-4-5. Se envía como id de modelo,
                no es un secreto.
              </small>
            </label>
            <label className="settings-field">
              <span>API key de Anthropic</span>
              <input
                type="password"
                autoComplete="off"
                value={secretDraft}
                placeholder={
                  anthropicConfigured
                    ? "Configurada en el llavero del sistema"
                    : "Se envía a Rust y se borra de esta ventana"
                }
                onChange={(event) => setSecretDraft(event.target.value)}
              />
              <small>
                Estado:{" "}
                {anthropicConfigured ? "sí, configurada" : "no configurada"}.
                Tras guardar, el valor desaparece de la interfaz y no vuelve por
                IPC.
              </small>
            </label>
            <div className="avatar-tools">
              <button
                type="button"
                className="primary-button"
                disabled={busy || secretDraft.trim().length === 0}
                onClick={() => {
                  const secret = secretDraft;
                  setSecretDraft("");
                  setBusy(true);
                  void backend
                    .setProviderSecret("anthropic", secret)
                    .then((status) => {
                      setAnthropicConfigured(status.configured);
                      setNotice("La key quedó en el llavero del sistema.");
                    })
                    .catch((error) =>
                      setNotice(normalizeBackendError(error).message),
                    )
                    .finally(() => setBusy(false));
                }}
              >
                Guardar en el llavero
              </button>
              <button
                type="button"
                className="secondary-button"
                disabled={busy || !anthropicConfigured}
                onClick={() => {
                  setBusy(true);
                  setSecretDraft("");
                  void backend
                    .deleteProviderSecret("anthropic")
                    .then((status) => {
                      setAnthropicConfigured(status.configured);
                      setNotice("Key eliminada del llavero.");
                    })
                    .catch((error) =>
                      setNotice(normalizeBackendError(error).message),
                    )
                    .finally(() => setBusy(false));
                }}
              >
                Eliminar key
              </button>
            </div>
          </div>
        ) : null}

        {tab === "Apariencia" ? (
          <div className="settings-section">
            <label className="settings-field">
              <span>Familiar activo</span>
              <select
                value={preferences.activeMascotId}
                onChange={(event) => {
                  const id = event.target.value;
                  void backend
                    .setActiveMascot(id)
                    .then((next) => {
                      setPreferences(next);
                      setNotice(
                        id === BUNDLED_MASCOT_ID
                          ? "Perrito Tech es el familiar predeterminado."
                          : "Familiar local activado.",
                      );
                    })
                    .catch((error) =>
                      setNotice(normalizeBackendError(error).message),
                    );
                }}
              >
                <option value={BUNDLED_MASCOT_ID}>
                  Perrito Tech (predeterminado)
                </option>
                {localMascots.map((mascot) => (
                  <option key={mascot.id} value={mascot.id}>
                    {mascot.name}
                  </option>
                ))}
              </select>
              <small>
                Los familiares importados viven en los datos de la aplicación y
                se validan como paquetes no confiables. Perrito Tech sigue
                siendo el predeterminado.
              </small>
            </label>
            <label className="settings-field">
              <span>Importar imagen</span>
              <input
                type="file"
                accept="image/png,image/jpeg,.png,.jpg,.jpeg"
                disabled={busy}
                onChange={(event) => {
                  const input = event.currentTarget;
                  const file = input.files?.[0];
                  input.value = "";
                  if (!file) return;
                  setBusy(true);
                  setImportError("");
                  void (async () => {
                    try {
                      const buffer = new Uint8Array(await file.arrayBuffer());
                      const name =
                        file.name.replace(/\.[^.]+$/, "") || "mi-familiar";
                      const info = await backend.importMascotImage({
                        name,
                        data: Array.from(buffer),
                      });
                      const listed = await backend.listLocalMascots();
                      setLocalMascots(listed);
                      const next = await backend.setActiveMascot(info.id);
                      setPreferences(next);
                      setNotice(`Importado y activado: ${info.name}.`);
                    } catch (error) {
                      const message = normalizeBackendError(error).message;
                      setImportError(message);
                      setNotice(message);
                    } finally {
                      setBusy(false);
                    }
                  })();
                }}
              />
              <small>
                PNG o JPEG local. Se convierte a una hoja 256×256 en pixel art y
                se guarda solo en esta máquina. No se envía a la red.
              </small>
              {importError ? <small role="alert">{importError}</small> : null}
            </label>
            <label className="settings-field">
              <span>Escala pixel-perfect</span>
              <select
                value={preferences.scale}
                onChange={(event) =>
                  update("scale", Number(event.target.value) as 1 | 2 | 3)
                }
              >
                <option value="1">1× (64 px)</option>
                <option value="2">2× (128 px, recomendado)</option>
                <option value="3">3× (192 px)</option>
              </select>
            </label>
            <label className="settings-field">
              <span>Color de acentos</span>
              <select
                value={preferences.mascotPalette}
                onChange={(event) =>
                  update(
                    "mascotPalette",
                    event.target.value as AppPreferences["mascotPalette"],
                  )
                }
              >
                <option value="teal">Teal</option>
                <option value="midnight">Azul medianoche</option>
                <option value="burgundy">Borgoña</option>
              </select>
              <small>
                Cambia sólo pequeños props de estado; el perro permanece natural
                y sin collar.
              </small>
            </label>
            <Toggle
              label="Animaciones"
              description="Anima sólo eventos activos; el idle permanece inmóvil."
              checked={preferences.animationsEnabled}
              onChange={(value) => update("animationsEnabled", value)}
            />
            <Toggle
              label="Reducir movimiento"
              description="Mantiene poses estáticas aunque las animaciones estén activadas."
              checked={preferences.reduceMotion}
              onChange={(value) => update("reduceMotion", value)}
            />
            <AvatarEditor
              localMascots={localMascots}
              onSaved={(info) => {
                void backend.listLocalMascots().then(setLocalMascots);
                setPreferences((current) => ({
                  ...current,
                  activeMascotId: info.id,
                }));
              }}
              onNotice={setNotice}
            />
          </div>
        ) : null}

        {tab === "Comportamiento" ? (
          <div className="settings-section">
            <Toggle
              label="Siempre visible"
              description="Mantiene a Perrito Tech sobre otras ventanas."
              checked={preferences.alwaysOnTop}
              onChange={(value) => update("alwaysOnTop", value)}
            />
            <Toggle
              label="Click-through"
              description="Ignora el puntero. Se desactiva al abrir Preguntar desde la bandeja."
              checked={preferences.clickThrough}
              onChange={(value) => update("clickThrough", value)}
            />
          </div>
        ) : null}

        {tab === "Privacidad" ? (
          <div className="settings-section prose-section">
            <h2>Contrato del MVP</h2>
            <p>
              Cada pregunta inicia un proceso efímero de Codex CLI en modo
              read-only. OpenFamiliar no conserva preguntas, respuestas ni
              historial.
            </p>
            <p>
              Los logs locales contienen solamente eventos operativos y
              categorías de error; nunca prompt, stdout ni stderr.
            </p>
            <p>
              No hay acceso genérico a shell, proveedor remoto, API key,
              workspace ni modo agente.
            </p>
          </div>
        ) : null}

        {tab === "Acerca de" ? (
          <div className="settings-section prose-section">
            <h2>OpenFamiliar 0.1.0</h2>
            <p>
              Perrito Tech es un familiar pixel-art original para Windows. El
              código se distribuye bajo Apache-2.0 y el arte bajo CC BY 4.0.
            </p>
            <p>
              Codex CLI es una dependencia externa: debe estar instalado y
              autenticado por el operador.
            </p>
          </div>
        ) : null}

        <footer>
          <span className="settings-notice" role="status">
            {notice}
          </span>
          <button
            className="primary-button"
            disabled={busy}
            onClick={() => void save()}
          >
            {busy ? "Guardando…" : "Guardar"}
          </button>
        </footer>
      </section>
    </main>
  );
}

function Toggle({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className="toggle-row">
      <span>
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  );
}
