import type { MascotVisualState } from "@openfamiliar/mascot-runtime";
import {
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { MascotSprite } from "../features/mascot/MascotSprite";
import { AnswerView } from "../features/quick-ask/AnswerView";
import {
  initialOneShotState,
  oneShotReducer,
} from "../features/quick-ask/oneShotMachine";
import { backend, isTauri, normalizeBackendError } from "../lib/backend";
import {
  DEFAULT_PREFERENCES,
  type AppPreferences,
  type ProviderStatus,
} from "../lib/types";

function createRequestId(): string {
  return (
    globalThis.crypto?.randomUUID?.() ??
    `${Date.now()}-${Math.random().toString(16).slice(2)}`
  );
}

export function MascotWindow() {
  const [ask, dispatch] = useReducer(oneShotReducer, initialOneShotState);
  const [preferences, setPreferences] =
    useState<AppPreferences>(DEFAULT_PREFERENCES);
  const [codexStatus, setCodexStatus] = useState<ProviderStatus>();
  const [copied, setCopied] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const dragStart = useRef<{ x: number; y: number }>();
  const dragging = useRef(false);
  const [isDragging, setIsDragging] = useState(false);
  const suppressClick = useRef(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const visualState: MascotVisualState = useMemo(() => {
    if (isDragging) return "dragging";
    if (ask.phase === "submitting") return "thinking";
    if (ask.phase === "answered") return "success";
    if (ask.phase === "error") return "error";
    if (ask.open) return "listening";
    return "idle";
  }, [ask.open, ask.phase, isDragging]);

  const openAsk = useCallback(() => {
    void backend.setMascotExpanded(true).then(() => {
      dispatch({ type: "open" });
      setMenuOpen(false);
      window.setTimeout(() => inputRef.current?.focus(), 0);
    });
  }, []);

  useEffect(() => {
    void backend.getPreferences().then(setPreferences);
    void backend.detectCodex().then(setCodexStatus);
    if (!isTauri()) return;
    let disposed = false;
    const cleanups: Array<() => void> = [];
    void (async () => {
      const [{ listen }, { getCurrentWindow }] = await Promise.all([
        import("@tauri-apps/api/event"),
        import("@tauri-apps/api/window"),
      ]);
      if (disposed) return;
      cleanups.push(await listen("quick-ask:open", openAsk));
      const currentWindow = getCurrentWindow();
      let saveTimer = 0;
      cleanups.push(
        await currentWindow.onMoved(({ payload }) => {
          window.clearTimeout(saveTimer);
          saveTimer = window.setTimeout(() => {
            void backend.saveMascotPosition();
          }, 250);
        }),
      );
      cleanups.push(() => window.clearTimeout(saveTimer));
    })();
    return () => {
      disposed = true;
      for (const cleanup of cleanups) cleanup();
    };
  }, [openAsk]);

  const submit = useCallback(async () => {
    const prompt = ask.question.trim();
    if (!prompt || ask.phase === "submitting") return;
    const requestId = createRequestId();
    dispatch({ type: "submit", requestId });
    try {
      const result = await backend.askCodex({
        requestId,
        prompt,
        timeoutSeconds: preferences.timeoutSeconds,
      });
      dispatch({ type: "answer", requestId, value: result.answer });
    } catch (error) {
      dispatch({
        type: "fail",
        requestId,
        value: normalizeBackendError(error).message,
      });
    }
  }, [ask.phase, ask.question, preferences.timeoutSeconds]);

  const closeAsk = useCallback(() => {
    if (ask.phase === "submitting" && ask.requestId)
      void backend.cancelCodex(ask.requestId);
    dispatch({ type: "close" });
    void backend.setMascotExpanded(menuOpen);
  }, [ask.phase, ask.requestId, menuOpen]);

  useEffect(() => {
    if (!ask.open) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      closeAsk();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [ask.open, closeAsk]);

  const startWindowDrag = useCallback(async () => {
    if (!isTauri()) return;
    dragging.current = true;
    setIsDragging(true);
    suppressClick.current = true;
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().startDragging();
    } finally {
      dragging.current = false;
      setIsDragging(false);
      window.setTimeout(() => {
        suppressClick.current = false;
      }, 80);
    }
  }, []);

  return (
    <main
      className="mascot-window"
      onPointerUp={() => (dragStart.current = undefined)}
    >
      <section
        className={`quick-ask-card ${ask.open ? "is-open" : ""}`}
        aria-hidden={!ask.open}
      >
        <header>
          <div>
            <span className="eyebrow">Perrito Tech</span>
            <h1>Pregunta rápida a Codex</h1>
          </div>
          <button
            className="icon-button"
            onClick={closeAsk}
            aria-label="Cerrar"
          >
            ×
          </button>
        </header>

        {codexStatus &&
        (!codexStatus.installed ||
          !codexStatus.compatible ||
          !codexStatus.authenticated) ? (
          <div className="onboarding-card">
            <strong>Codex CLI requiere atención</strong>
            <p>{codexStatus.message}</p>
            <button
              className="secondary-button"
              onClick={() => void backend.openSettings()}
            >
              Abrir ajustes
            </button>
          </div>
        ) : null}

        {ask.phase === "answered" ? (
          <div className="result-area">
            <AnswerView answer={ask.answer} />
            <div className="button-row">
              <button
                className="secondary-button"
                onClick={() => {
                  void navigator.clipboard.writeText(ask.answer).then(() => {
                    setCopied(true);
                    window.setTimeout(() => setCopied(false), 1200);
                  });
                }}
              >
                {copied ? "Copiado" : "Copiar"}
              </button>
              <button
                className="primary-button"
                onClick={() => dispatch({ type: "new" })}
              >
                Nueva pregunta
              </button>
            </div>
          </div>
        ) : (
          <>
            <textarea
              ref={inputRef}
              value={ask.question}
              maxLength={8000}
              placeholder="Escribe una pregunta independiente…"
              disabled={ask.phase === "submitting"}
              onChange={(event) =>
                dispatch({ type: "edit", value: event.target.value })
              }
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  void submit();
                }
              }}
            />
            {ask.phase === "error" ? (
              <p className="error-message">{ask.error}</p>
            ) : null}
            <div className="button-row">
              <span className="privacy-note">
                Sin historial · sesión efímera · read-only
              </span>
              {ask.phase === "submitting" ? (
                <button className="danger-button" onClick={closeAsk}>
                  Cancelar
                </button>
              ) : (
                <button
                  className="primary-button"
                  disabled={
                    !ask.question.trim() ||
                    Boolean(
                      codexStatus &&
                      (!codexStatus.installed ||
                        !codexStatus.compatible ||
                        !codexStatus.authenticated),
                    )
                  }
                  onClick={() => void submit()}
                >
                  Preguntar
                </button>
              )}
            </div>
          </>
        )}
      </section>

      <div className="mascot-anchor">
        <div
          className="mascot-click-zone"
          role="button"
          tabIndex={0}
          aria-label="Abrir pregunta rápida a Codex"
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              if (ask.open) closeAsk();
              else openAsk();
            }
          }}
          onContextMenu={(event) => {
            event.preventDefault();
            const next = !menuOpen;
            setMenuOpen(next);
            void backend.setMascotExpanded(next || ask.open);
          }}
          onClick={() => {
            if (!suppressClick.current) {
              if (ask.open) closeAsk();
              else openAsk();
            }
          }}
          title="Clic: preguntar · Clic derecho: menú"
        >
          <MascotSprite
            state={visualState}
            scale={preferences.scale}
            palette={preferences.mascotPalette}
            animationsEnabled={preferences.animationsEnabled}
            reduceMotion={preferences.reduceMotion}
          />
        </div>
        <button
          className="mascot-drag-handle"
          aria-label="Mover Perrito Tech"
          title="Arrastrar para mover"
          onClick={(event) => event.stopPropagation()}
          onPointerDown={(event) => {
            event.stopPropagation();
            if (event.button !== 0) return;
            dragStart.current = { x: event.clientX, y: event.clientY };
            event.currentTarget.setPointerCapture(event.pointerId);
          }}
          onPointerMove={(event) => {
            event.stopPropagation();
            if (!dragStart.current || dragging.current) return;
            const distance = Math.hypot(
              event.clientX - dragStart.current.x,
              event.clientY - dragStart.current.y,
            );
            if (distance >= 4) void startWindowDrag();
          }}
          onPointerUp={(event) => {
            event.stopPropagation();
            dragStart.current = undefined;
          }}
        >
          <span aria-hidden="true">•••</span>
        </button>
      </div>

      {menuOpen ? (
        <nav className="mini-menu" aria-label="Menú de Perrito Tech">
          <button onClick={openAsk}>Preguntar</button>
          <button
            onClick={() => {
              setMenuOpen(false);
              void backend
                .setMascotExpanded(false)
                .then(() => backend.openSettings());
            }}
          >
            Ajustes
          </button>
          <button
            onClick={() =>
              void backend.setClickThrough(true).then((next) => {
                setPreferences(next);
                setMenuOpen(false);
                void backend.setMascotExpanded(false);
              })
            }
          >
            Activar click-through
          </button>
        </nav>
      ) : null}
    </main>
  );
}
