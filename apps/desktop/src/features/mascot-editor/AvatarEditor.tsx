import type { MascotVisualState } from "@openfamiliar/mascot-runtime";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { backend, normalizeBackendError } from "../../lib/backend";
import type { LocalMascotInfo } from "../../lib/types";
import { MascotSprite } from "../mascot/MascotSprite";
import { sheetDataUrl, toRuntimeManifest } from "../mascot/localMascotRuntime";
import type { LocalMascotRuntime } from "../../lib/types";
import {
  ALLOWED_LICENSES,
  CELL_SIZE,
  MAX_PERSONALITY_CHARS,
  REQUIRED_STATES,
  SHEET_COLUMNS,
  SHEET_ROWS,
  SHEET_SIZE,
  SHORT_PALETTE,
  assignFrame,
  cellIndex,
  cellOrigin,
  clampFps,
  defaultAnimations,
  isAllowedLicense,
  type EditorAnimation,
  type EditorLicense,
  type EditorState,
} from "./model";

interface AvatarEditorProps {
  localMascots: LocalMascotInfo[];
  onSaved: (info: LocalMascotInfo) => void;
  onNotice: (message: string) => void;
}

const PREVIEW_STATES: MascotVisualState[] = [
  "idle",
  "thinking",
  "success",
  "error",
];

function hexToRgba(hex: string): [number, number, number, number] {
  const value = hex.replace("#", "");
  return [
    Number.parseInt(value.slice(0, 2), 16),
    Number.parseInt(value.slice(2, 4), 16),
    Number.parseInt(value.slice(4, 6), 16),
    255,
  ];
}

function putPixel(
  image: ImageData,
  x: number,
  y: number,
  color: [number, number, number, number],
) {
  if (x < 0 || y < 0 || x >= image.width || y >= image.height) return;
  const i = (y * image.width + x) * 4;
  image.data[i] = color[0];
  image.data[i + 1] = color[1];
  image.data[i + 2] = color[2];
  image.data[i + 3] = color[3];
}

async function canvasPngBytes(canvas: HTMLCanvasElement): Promise<number[]> {
  const blob = await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((value) => {
      if (value) resolve(value);
      else reject(new Error("could not encode the sheet"));
    }, "image/png");
  });
  return Array.from(new Uint8Array(await blob.arrayBuffer()));
}

export function AvatarEditor({
  localMascots,
  onSaved,
  onNotice,
}: AvatarEditorProps) {
  const sheetRef = useRef<HTMLCanvasElement>(null);
  const cellRef = useRef<HTMLCanvasElement>(null);
  const [existingId, setExistingId] = useState<string>("");
  const [name, setName] = useState("Mi familiar");
  const [license, setLicense] = useState<EditorLicense>("CC0-1.0");
  const [personality, setPersonality] = useState(
    "Un familiar amistoso de escritorio.",
  );
  const [fps, setFps] = useState(4);
  const [animations, setAnimations] = useState(() => defaultAnimations(4));
  const [activeState, setActiveState] = useState<EditorState>("idle");
  const [selectedCell, setSelectedCell] = useState(0);
  const [tool, setTool] = useState<"pencil" | "eraser">("pencil");
  const [color, setColor] = useState<string>(SHORT_PALETTE[2]);
  const [previewUrl, setPreviewUrl] = useState<string>();
  const [previewState, setPreviewState] = useState<MascotVisualState>("idle");
  const [busy, setBusy] = useState(false);
  const drawing = useRef(false);

  const refreshPreview = useCallback(() => {
    const canvas = sheetRef.current;
    if (!canvas) return;
    setPreviewUrl(canvas.toDataURL("image/png"));
    const cell = cellRef.current;
    const sheetCtx = canvas.getContext("2d");
    const cellCtx = cell?.getContext("2d");
    if (!cell || !sheetCtx || !cellCtx) return;
    const origin = cellOrigin(selectedCell);
    cellCtx.imageSmoothingEnabled = false;
    cellCtx.clearRect(0, 0, CELL_SIZE, CELL_SIZE);
    cellCtx.drawImage(
      canvas,
      origin.x,
      origin.y,
      CELL_SIZE,
      CELL_SIZE,
      0,
      0,
      CELL_SIZE,
      CELL_SIZE,
    );
  }, [selectedCell]);

  useEffect(() => {
    const canvas = sheetRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.imageSmoothingEnabled = false;
    refreshPreview();
  }, [refreshPreview]);

  const paintAt = useCallback(
    (clientX: number, clientY: number, target: HTMLCanvasElement) => {
      const sheet = sheetRef.current;
      if (!sheet) return;
      const sheetCtx = sheet.getContext("2d");
      if (!sheetCtx) return;
      const rect = target.getBoundingClientRect();
      const localX = Math.floor(
        ((clientX - rect.left) / rect.width) * CELL_SIZE,
      );
      const localY = Math.floor(
        ((clientY - rect.top) / rect.height) * CELL_SIZE,
      );
      const origin = cellOrigin(selectedCell);
      const image = sheetCtx.getImageData(0, 0, SHEET_SIZE, SHEET_SIZE);
      const rgba =
        tool === "eraser" ? ([0, 0, 0, 0] as const) : hexToRgba(color);
      putPixel(image, origin.x + localX, origin.y + localY, [...rgba]);
      sheetCtx.putImageData(image, 0, 0);
      refreshPreview();
    },
    [color, refreshPreview, selectedCell, tool],
  );

  const loadExisting = async (id: string) => {
    if (!id) {
      setExistingId("");
      setName("Mi familiar");
      setLicense("CC0-1.0");
      setPersonality("Un familiar amistoso de escritorio.");
      setAnimations(defaultAnimations(fps));
      const canvas = sheetRef.current;
      const ctx = canvas?.getContext("2d");
      ctx?.clearRect(0, 0, SHEET_SIZE, SHEET_SIZE);
      refreshPreview();
      return;
    }
    setBusy(true);
    try {
      const packed = await backend.getEditorMascot(id);
      setExistingId(packed.runtime.id);
      setName(packed.runtime.name);
      setLicense(
        isAllowedLicense(packed.license) ? packed.license : "NOASSERTION",
      );
      setPersonality(packed.personality.slice(0, MAX_PERSONALITY_CHARS));
      const next = defaultAnimations(fps);
      for (const state of REQUIRED_STATES) {
        const source = packed.runtime.animations[state];
        if (source?.frames.length) {
          next[state] = {
            frames: source.frames.filter((frame) => frame >= 0 && frame <= 15),
            fps: clampFps(source.fps),
            loop: source.loop,
          };
        }
      }
      setAnimations(next);
      const canvas = sheetRef.current;
      const ctx = canvas?.getContext("2d");
      if (canvas && ctx) {
        const image = new Image();
        await new Promise<void>((resolve, reject) => {
          image.onload = () => resolve();
          image.onerror = () => reject(new Error("could not load the sheet"));
          image.src = sheetDataUrl(packed.runtime.sheetPngBase64);
        });
        ctx.imageSmoothingEnabled = false;
        ctx.clearRect(0, 0, SHEET_SIZE, SHEET_SIZE);
        ctx.drawImage(image, 0, 0, SHEET_SIZE, SHEET_SIZE);
        refreshPreview();
      }
    } catch (error) {
      onNotice(normalizeBackendError(error).message);
    } finally {
      setBusy(false);
    }
  };

  const save = async () => {
    const canvas = sheetRef.current;
    if (!canvas) return;
    setBusy(true);
    try {
      const data = await canvasPngBytes(canvas);
      const info = await backend.saveEditorMascot({
        existingId: existingId || undefined,
        name,
        license,
        personality: personality.slice(0, MAX_PERSONALITY_CHARS),
        data,
        fps: clampFps(fps),
        animations,
      });
      setExistingId(info.id);
      onSaved(info);
      onNotice(`Familiar guardado: ${info.name}.`);
    } catch (error) {
      onNotice(normalizeBackendError(error).message);
    } finally {
      setBusy(false);
    }
  };

  const previewRuntime = useMemo(() => {
    if (!previewUrl) return undefined;
    const runtime: LocalMascotRuntime = {
      id: existingId || "editor-preview",
      name,
      sheetPngBase64: previewUrl.replace(/^data:image\/png;base64,/, ""),
      frameWidth: CELL_SIZE,
      frameHeight: CELL_SIZE,
      columns: SHEET_COLUMNS,
      rows: SHEET_ROWS,
      animations: animations as Record<string, EditorAnimation>,
    };
    return toRuntimeManifest(runtime);
  }, [animations, existingId, name, previewUrl]);

  const applyFps = (value: number) => {
    const next = clampFps(value);
    setFps(next);
    setAnimations((current) => {
      const updated = { ...current };
      for (const state of REQUIRED_STATES) {
        updated[state] = { ...updated[state], fps: next };
      }
      return updated;
    });
  };

  return (
    <div className="avatar-editor">
      <h3>Editor de avatar</h3>
      <p className="avatar-editor-note">
        Crea un familiar 4×4 (celdas de 64×64) y asígnalas a los estados. La
        personalidad se guarda en el pack y no se envía a ningún modelo en este
        PR.
      </p>
      <label className="settings-field">
        <span>Editar un familiar local</span>
        <select
          value={existingId}
          disabled={busy}
          onChange={(event) => void loadExisting(event.target.value)}
        >
          <option value="">Nuevo familiar</option>
          {localMascots.map((mascot) => (
            <option key={mascot.id} value={mascot.id}>
              {mascot.name}
            </option>
          ))}
        </select>
      </label>
      <label className="settings-field">
        <span>Nombre</span>
        <input
          value={name}
          maxLength={80}
          onChange={(event) => setName(event.target.value)}
        />
      </label>
      <label className="settings-field">
        <span>Licencia</span>
        <select
          value={license}
          onChange={(event) =>
            setLicense(
              isAllowedLicense(event.target.value)
                ? event.target.value
                : "NOASSERTION",
            )
          }
        >
          {ALLOWED_LICENSES.map((item) => (
            <option key={item} value={item}>
              {item}
            </option>
          ))}
        </select>
      </label>
      <label className="settings-field">
        <span>Personalidad (solo local, no se usa en prompts)</span>
        <textarea
          rows={4}
          maxLength={MAX_PERSONALITY_CHARS}
          value={personality}
          onChange={(event) => setPersonality(event.target.value)}
        />
      </label>
      <label className="settings-field">
        <span>FPS ({fps})</span>
        <input
          type="range"
          min={1}
          max={12}
          value={fps}
          onChange={(event) => applyFps(Number(event.target.value))}
        />
      </label>
      <div className="avatar-editor-layout">
        <div>
          <span className="settings-field-label">Hoja 4×4</span>
          <div
            className="avatar-sheet-grid"
            role="grid"
            aria-label="Celdas de la hoja"
          >
            {Array.from({ length: SHEET_ROWS * SHEET_COLUMNS }, (_, frame) => (
              <button
                key={frame}
                type="button"
                className={
                  frame === selectedCell
                    ? "avatar-cell avatar-cell-active"
                    : "avatar-cell"
                }
                onClick={() => {
                  setSelectedCell(frame);
                  setAnimations((current) =>
                    assignFrame(current, activeState, frame, true),
                  );
                }}
                aria-label={`Celda ${frame}`}
                aria-pressed={frame === selectedCell}
              >
                {frame}
              </button>
            ))}
          </div>
          <canvas
            ref={sheetRef}
            width={SHEET_SIZE}
            height={SHEET_SIZE}
            className="avatar-sheet-canvas"
            aria-hidden="true"
          />
        </div>
        <div>
          <span className="settings-field-label">
            Pintar celda {selectedCell}
          </span>
          <div className="avatar-tools">
            <button
              type="button"
              className={
                tool === "pencil"
                  ? "secondary-button is-active"
                  : "secondary-button"
              }
              onClick={() => setTool("pencil")}
            >
              Lápiz
            </button>
            <button
              type="button"
              className={
                tool === "eraser"
                  ? "secondary-button is-active"
                  : "secondary-button"
              }
              onClick={() => setTool("eraser")}
            >
              Goma
            </button>
          </div>
          <div className="avatar-palette" role="listbox" aria-label="Paleta">
            {SHORT_PALETTE.map((swatch) => (
              <button
                key={swatch}
                type="button"
                className={
                  swatch === color
                    ? "avatar-swatch avatar-swatch-active"
                    : "avatar-swatch"
                }
                style={{ backgroundColor: swatch }}
                aria-label={swatch}
                aria-pressed={swatch === color}
                onClick={() => {
                  setColor(swatch);
                  setTool("pencil");
                }}
              />
            ))}
          </div>
          <canvas
            ref={cellRef}
            className="avatar-cell-canvas"
            width={CELL_SIZE}
            height={CELL_SIZE}
            aria-label="Lienzo de la celda seleccionada"
            onPointerDown={(event) => {
              drawing.current = true;
              event.currentTarget.setPointerCapture(event.pointerId);
              paintAt(event.clientX, event.clientY, event.currentTarget);
            }}
            onPointerMove={(event) => {
              if (!drawing.current) return;
              paintAt(event.clientX, event.clientY, event.currentTarget);
            }}
            onPointerUp={() => {
              drawing.current = false;
            }}
          />
          <p className="avatar-editor-note">
            El lienzo de celda pinta sobre la hoja oculta de 256×256. Clic en
            una celda de la cuadrícula también la asigna al estado activo.
          </p>
        </div>
      </div>
      <label className="settings-field">
        <span>Estado a asignar</span>
        <select
          value={activeState}
          onChange={(event) =>
            setActiveState(event.target.value as EditorState)
          }
        >
          {REQUIRED_STATES.map((state) => (
            <option key={state} value={state}>
              {state} ({animations[state].frames.join(", ")})
            </option>
          ))}
        </select>
      </label>
      <div className="avatar-preview">
        <span className="settings-field-label">Vista previa</span>
        <div className="avatar-preview-states">
          {PREVIEW_STATES.map((state) => (
            <button
              key={state}
              type="button"
              className="secondary-button"
              onClick={() => setPreviewState(state)}
            >
              {state}
            </button>
          ))}
        </div>
        {previewUrl && previewRuntime ? (
          <MascotSprite
            state={previewState}
            scale={2}
            palette="teal"
            animationsEnabled
            reduceMotion={false}
            displayName={name}
            customSheetUrl={previewUrl}
            customManifest={previewRuntime}
          />
        ) : null}
      </div>
      <button
        type="button"
        className="primary-button"
        disabled={busy || name.trim().length === 0}
        onClick={() => void save()}
      >
        Guardar familiar
      </button>
    </div>
  );
}
