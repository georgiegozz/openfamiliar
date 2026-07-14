import { useMemo, useState } from "react";

const STEPS = [
  "Crear proyecto",
  "Nombre y autor",
  "Licencia",
  "Importar sprites",
  "Asignar estados",
  "Escala y FPS",
  "Personalidad",
  "Probar eventos",
  "Validar",
  "Exportar .familiar",
];

export function App() {
  const [step, setStep] = useState(0);
  const [name, setName] = useState("Mi Familiar");
  const [author, setAuthor] = useState("");
  const [license, setLicense] = useState("CC-BY-4.0");
  const [personality, setPersonality] = useState("Un compañero amable.");
  const manifest = useMemo(
    () => ({
      id:
        name
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, "-")
          .replace(/^-|-$/g, "") || "familiar",
      name,
      version: "0.1.0",
      engine: ">=0.1.0",
      author: author || "unknown",
      license,
      personality: "personality.md",
      states: {
        idle: "assets/idle.webp",
        thinking: "assets/thinking.webp",
        working: "assets/working.webp",
        approval: "assets/approval.webp",
        success: "assets/success.webp",
        error: "assets/error.webp",
      },
    }),
    [name, author, license],
  );

  return (
    <div
      style={{
        fontFamily: "Segoe UI, sans-serif",
        maxWidth: 720,
        margin: "2rem auto",
        padding: 16,
      }}
    >
      <h1>Familiar Creator Studio</h1>
      <p>
        Flujo guiado (scaffold MVP). Exportación real vía CLI{" "}
        <code>familiar pack build</code>.
      </p>
      <ol>
        {STEPS.map((s, i) => (
          <li key={s} style={{ fontWeight: i === step ? 700 : 400 }}>
            {s}
          </li>
        ))}
      </ol>
      <div style={{ display: "grid", gap: 8, marginTop: 16 }}>
        <label>
          Nombre{" "}
          <input value={name} onChange={(e) => setName(e.target.value)} />
        </label>
        <label>
          Autor{" "}
          <input value={author} onChange={(e) => setAuthor(e.target.value)} />
        </label>
        <label>
          Licencia
          <select value={license} onChange={(e) => setLicense(e.target.value)}>
            <option>CC-BY-4.0</option>
            <option>CC0-1.0</option>
            <option>NOASSERTION</option>
          </select>
        </label>
        <label>
          Personalidad{" "}
          <textarea
            value={personality}
            onChange={(e) => setPersonality(e.target.value)}
            rows={4}
          />
        </label>
      </div>
      <pre
        style={{
          background: "#0f172a",
          color: "#e2e8f0",
          padding: 12,
          borderRadius: 8,
          overflow: "auto",
        }}
      >
        {JSON.stringify(manifest, null, 2)}
      </pre>
      <div style={{ display: "flex", gap: 8 }}>
        <button
          type="button"
          disabled={step === 0}
          onClick={() => setStep((s) => s - 1)}
        >
          Anterior
        </button>
        <button
          type="button"
          disabled={step >= STEPS.length - 1}
          onClick={() => setStep((s) => s + 1)}
        >
          Siguiente
        </button>
        <button
          type="button"
          onClick={() =>
            navigator.clipboard.writeText(JSON.stringify(manifest, null, 2))
          }
        >
          Copiar familiar.json
        </button>
      </div>
    </div>
  );
}
