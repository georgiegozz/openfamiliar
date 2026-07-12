import { useCallback, useEffect, useMemo, useState } from "react";
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
  const [panelOpen, setPanelOpen] = useState(true);
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

  const provider = useMemo(
    () => PROVIDERS.find((p) => p.id === providerId) ?? PROVIDERS[0]!,
    [providerId],
  );

  useEffect(() => {
    setModel(provider.model);
  }, [provider]);

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
      setTimeout(() => void applyState("idle"), 1200);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
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
    } catch (e) {
      setContextPreview(e instanceof Error ? e.message : String(e));
    }
  };

  const toggleClickThrough = async () => {
    const next = !clickThrough;
    setClickThrough(next);
    await invokeBackend("set_click_through", { enabled: next });
  };

  return (
    <div className="app">
      <div
        className="mascot-stage"
        data-tauri-drag-region
        onDoubleClick={() => setPanelOpen((v) => !v)}
        title="Doble clic: panel · Arrastrar: mover"
      >
        {speech ? <div className="mascot-bubble">{speech}</div> : null}
        <div className={`mascot-face ${state}`}>
          <span className="ear left" />
          <span className="ear right" />
          <div className="eyes">
            <span className="eye" />
            <span className="eye" />
          </div>
          <div className="nose" />
        </div>
        <div className="state-chip">{state}</div>
      </div>

      <div className="toolbar">
        <button type="button" className="secondary" onClick={() => setPanelOpen((v) => !v)}>
          {panelOpen ? "Ocultar panel" : "Chat / Config"}
        </button>
        <button type="button" className="secondary" onClick={() => void toggleClickThrough()}>
          Click-through: {clickThrough ? "ON" : "OFF"}
        </button>
      </div>

      <div className={`panel ${panelOpen ? "" : "hidden"}`}>
        <div className="row">
          <label>
            Provider{" "}
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
          <label>
            Model{" "}
            <input value={model} onChange={(e) => setModel(e.target.value)} disabled={busy} />
          </label>
        </div>
        <div className="row">
          <label>
            Mode{" "}
            <select
              value={mode}
              onChange={(e) => {
                const m = e.target.value as typeof mode;
                setMode(m);
                void invokeBackend("set_security_mode", { mode: m });
              }}
            >
              <option value="chat">Chat</option>
              <option value="read_only">Read-only</option>
              <option value="agent">Agent</option>
            </select>
          </label>
          <label>
            State{" "}
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
        <div className="row">
          <input
            style={{ flex: 1 }}
            placeholder="Ruta workspace (autorización explícita)"
            value={workspacePath}
            onChange={(e) => setWorkspacePath(e.target.value)}
          />
          <button type="button" className="secondary" onClick={() => void authorizeWorkspace()}>
            Autorizar
          </button>
        </div>
        <div className="context-preview">{contextPreview}</div>
        <div className="messages">
          {messages.map((m, i) => (
            <div key={i} className={`msg ${m.role}`}>
              <strong>{m.role}:</strong> {m.content}
            </div>
          ))}
        </div>
        <textarea
          placeholder="Pregunta a Perrito Tech…"
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
        <div className="row">
          <button type="button" onClick={() => void sendChat()} disabled={busy || !input.trim()}>
            {busy ? "…" : "Enviar"}
          </button>
          <button
            type="button"
            className="secondary"
            onClick={() => {
              setMessages([]);
              setSpeech("Listo.");
              void applyState("idle");
            }}
          >
            Limpiar
          </button>
          <span className="state-chip">{isTauri() ? "tauri" : "web-mock"}</span>
        </div>
      </div>
    </div>
  );
}
