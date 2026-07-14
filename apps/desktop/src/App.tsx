import { useCallback, useEffect, useMemo, useState, useRef } from "react";
import type { MascotState } from "./lib/states";
import { MASCOT_STATES } from "./lib/states";
import { invokeBackend, isTauri } from "./lib/backend";

type ChatMsg = { role: "user" | "assistant" | "error"; content: string };

const PROVIDERS = [
  { id: "mock", label: "Mock (offline demo)", model: "mock-model" },
  { id: "ollama-local", label: "Ollama local", model: "llama3.2" },
  { id: "openai-compatible", label: "OpenAI-compatible", model: "gpt-4o-mini" },
  { id: "gemini-native", label: "Gemini native", model: "gemini-2.0-flash" },
];

export function App() {
  const [state, setState] = useState<MascotState>("idle");
  const [panelOpen, setPanelOpen] = useState(false); // Starts collapsed (floating mascot only)
  const [speech, setSpeech] = useState("¡Hola! Soy Perrito Tech.");
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<ChatMsg[]>([]);
  const [providerId, setProviderId] = useState("mock");
  const [model, setModel] = useState("mock-model");
  const [mode, setMode] = useState<"chat" | "read_only" | "agent">("chat");
  const [busy, setBusy] = useState(false);
  const [workspacePath, setWorkspacePath] = useState("");
  const [contextPreview, setContextPreview] = useState("Sin workspace autorizado.");
  const [clickThrough, setClickThrough] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);

  const provider = useMemo(
    () => PROVIDERS.find((p) => p.id === providerId) ?? PROVIDERS[0]!,
    [providerId],
  );

  useEffect(() => {
    setModel(provider.model);
  }, [provider]);

  // Handle window resizing dynamically based on panel status
  useEffect(() => {
    if (panelOpen) {
      invokeBackend("resize_window", { width: 380, height: 680 }).catch(console.error);
    } else {
      invokeBackend("resize_window", { width: 240, height: 280 }).catch(console.error);
    }
  }, [panelOpen]);

  // Auto-scroll chat messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const applyState = useCallback(async (next: MascotState) => {
    setState(next);
    await invokeBackend("set_mascot_state", { state: next });
  }, []);

  const sendChat = async () => {
    const text = input.trim();
    if (!text || busy) return;
    setInput("");
    setMessages((m) => [...m, { role: "user", content: text }]);
    setBusy(true);
    await applyState("thinking");
    setSpeech("Pensando…");
    try {
      const reply = await invokeBackend<string>("chat", {
        providerId,
        model,
        message: text,
        maxTokens: 512,
      });
      setMessages((m) => [...m, { role: "assistant", content: reply }]);
      setSpeech(reply.slice(0, 140));
      await applyState("success");
      setTimeout(() => void applyState("idle"), 1500);
    } catch (e) {
      let msg = "Error al responder";
      if (e instanceof Error) {
        msg = e.message;
      } else if (typeof e === "object" && e !== null && "message" in e) {
        msg = String((e as { message: unknown }).message);
      } else {
        msg = String(e);
      }
      setMessages((m) => [...m, { role: "error", content: msg }]);
      setSpeech("Error al responder");
      await applyState("error");
    } finally {
      setBusy(false);
    }
  };

  const authorizeWorkspace = async () => {
    if (!workspacePath.trim()) return;
    try {
      await invokeBackend("authorize_workspace", {
        id: "default",
        path: workspacePath.trim(),
      });
      const preview = await invokeBackend<string>("preview_workspace", {
        id: "default",
        paths: [],
      });
      setContextPreview(preview);
      setMode("read_only");
      await invokeBackend("set_security_mode", { mode: "read_only" });
      setSpeech("Workspace autorizado (read-only)");
      await applyState("success");
      setTimeout(() => void applyState("idle"), 1200);
    } catch (e) {
      setContextPreview(e instanceof Error ? e.message : String(e));
      await applyState("error");
    }
  };

  const toggleClickThrough = async () => {
    const next = !clickThrough;
    setClickThrough(next);
    await invokeBackend("set_click_through", { enabled: next });
  };

  return (
    <div className="app">
      {/* Mascot Stage */}
      <div
        className="mascot-stage"
        data-tauri-drag-region
        onClick={() => setPanelOpen((v) => !v)}
        title="Clic: Abrir/Cerrar Chat · Arrastrar: Mover"
      >
        {speech ? (
          <div className="mascot-bubble" onClick={(e) => e.stopPropagation()}>
            {speech}
          </div>
        ) : null}
        
        <div className={`mascot-avatar-container ${state}`}>
          <img src="/perrito.png" className="mascot-img" alt="Perrito Tech" />
          <div className={`mascot-status-glow ${state}`} />
        </div>
        
        <div className={`state-badge ${state}`}>{state}</div>
      </div>

      {/* Toolbar */}
      <div className="toolbar">
        <button
          type="button"
          className={`secondary ${panelOpen ? "active" : ""}`}
          onClick={() => setPanelOpen((v) => !v)}
        >
          {panelOpen ? "Minimizar" : "Chat & Ajustes"}
        </button>
        <button
          type="button"
          className={`secondary ${clickThrough ? "active" : ""}`}
          onClick={() => void toggleClickThrough()}
          title="Permite hacer clic a través del fondo de la ventana"
        >
          Fantasma: {clickThrough ? "ON" : "OFF"}
        </button>
      </div>

      {/* Main Glassmorphic Panel */}
      <div className={`panel ${panelOpen ? "" : "hidden"}`}>
        <div className="panel-header">
          <h3>Ajustes de Perrito Tech</h3>
        </div>
        
        <div className="row">
          <label className="field-group">
            <span>Proveedor</span>
            <select
              value={providerId}
              onChange={(e) => setProviderId(e.target.value)}
              disabled={busy}
            >
              {PROVIDERS.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.label}
                </option>
              ))}
            </select>
          </label>
          <label className="field-group">
            <span>Modelo</span>
            <input value={model} onChange={(e) => setModel(e.target.value)} disabled={busy} />
          </label>
        </div>

        <div className="row">
          <label className="field-group">
            <span>Modo de Seguridad</span>
            <select
              value={mode}
              onChange={(e) => {
                const m = e.target.value as typeof mode;
                setMode(m);
                void invokeBackend("set_security_mode", { mode: m });
              }}
            >
              <option value="chat">Chat (Sin acceso a archivos)</option>
              <option value="read_only">Lectura (Read-only)</option>
              <option value="agent">Agente (Full autocontrol)</option>
            </select>
          </label>
          <label className="field-group">
            <span>Estado Forzado</span>
            <select
              value={state}
              onChange={(e) => void applyState(e.target.value as MascotState)}
            >
              {MASCOT_STATES.map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div className="row input-action">
          <input
            style={{ flex: 1 }}
            placeholder="Ruta absoluta al workspace..."
            value={workspacePath}
            onChange={(e) => setWorkspacePath(e.target.value)}
          />
          <button type="button" className="action-btn" onClick={() => void authorizeWorkspace()}>
            Autorizar
          </button>
        </div>
        
        <div className="context-preview-box">
          <span className="section-label">Contexto del Workspace:</span>
          <div className="context-preview">{contextPreview}</div>
        </div>

        <div className="chat-section">
          <span className="section-label">Chat en Vivo:</span>
          <div className="messages">
            {messages.length === 0 ? (
              <div className="empty-chat">No hay mensajes. ¡Pregúntame algo!</div>
            ) : (
              messages.map((m, i) => (
                <div key={i} className={`msg ${m.role}`}>
                  <span className="role-label">{m.role === "user" ? "Tú" : "Perrito"}:</span>
                  <p>{m.content}</p>
                </div>
              ))
            )}
            <div ref={messagesEndRef} />
          </div>
        </div>

        <div className="chat-input-area">
          <textarea
            placeholder="Escribe algo aquí para Perrito Tech..."
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                void sendChat();
              }
            }}
            disabled={busy}
          />
          <div className="chat-actions">
            <button type="button" className="primary-btn" onClick={() => void sendChat()} disabled={busy || !input.trim()}>
              {busy ? "Enviando..." : "Enviar"}
            </button>
            <button
              type="button"
              className="clear-btn"
              onClick={() => {
                setMessages([]);
                setSpeech("Listo.");
                void applyState("idle");
              }}
            >
              Limpiar
            </button>
            <span className="environment-tag">{isTauri() ? "Tauri Native" : "Web Mock"}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
