# Manual humano — seguimiento post-implementación

Documento **versionado** con las acciones que solo puedes hacer tú (humano):
cuentas, arte final, beta real, publicación, legal de marcas y pruebas en tu máquina.

El tracking privado de decisiones diarias vive en `.private/` (gitignored).

---

## 1. Inmediato (hoy / esta semana)

| # | Acción | Por qué | Hecho |
|---|--------|---------|-------|
| 1 | Confirmar remote `https://github.com/georgiegozz/openfamiliar.git` y que el repo siga **privado** | Push de fundación | ☐ |
| 2 | Instalar/verificar: Node 20+, pnpm, Rust stable, WebView2, **VS Build Tools 2022 + C++** (`link.exe`) | Compilar crates y Tauri (el scaffold falló sin MSVC en la máquina) | ☐ |
| 3 | Ejecutar `.\scripts\bootstrap.ps1` en una shell nueva (PATH de cargo) | Dependencias locales | ☐ |
| 4 | `cargo test --workspace` y `pnpm test` | Validar fundación | ☐ |
| 5 | Proteger `main` en GitHub: PR required, sin push directo si lo deseas | Gobernanza del plan | ☐ |
| 6 | Activar secret scanning / push protection en el repo privado | Política de secretos | ☐ |
| 7 | Crear API keys de desarrollo (Gemini / OpenAI-compatible / xAI) **solo en Credential Manager o `.env` local** | Probar providers reales | ☐ |
| 8 | Instalar Ollama y un modelo pequeño (`llama3.2` o similar) | Provider local | ☐ |

## 2. Arte y marca (antes de capturas públicas)

| # | Acción | Notas | Hecho |
|---|--------|-------|-------|
| 9 | Reemplazar placeholders WebP de Perrito Tech por arte final | Licencia **CC-BY-4.0**, autor y origen en `mascots/perrito-tech` | ☐ |
| 10 | Añadir sonidos opcionales bajo `sounds/` con licencia clara | No usar assets de Clawd / All Rights Reserved | ☐ |
| 11 | Iconos de app (ICO/PNG multi-size) para instalador Windows | Sustituir `apps/desktop/src-tauri/icons` | ☐ |
| 12 | Búsqueda formal de nombre: GitHub orgs, npm, crates.io, dominios, marcas | Plan § revisión de nombre | ☐ |
| 13 | Decidir dominio `openfamiliar.dev` (u otro) y si se registra | Schema URL en manifiestos | ☐ |

## 3. Producto desktop (pruebas humanas)

| # | Acción | Criterio | Hecho |
|---|--------|----------|-------|
| 14 | `pnpm --filter @openfamiliar/desktop tauri dev` | Ventana transparente + always-on-top | ☐ |
| 15 | Arrastrar mascota; doble monitor; DPI 100%/125%/150% | Posición estable | ☐ |
| 16 | Click-through ON: el escritorio recibe clics | No bloquea UI ajena | ☐ |
| 17 | System tray: hide on close, show, quit | Cierre limpio | ☐ |
| 18 | Chat mock → success state | Streaming simulado | ☐ |
| 19 | Chat Ollama real | Respuesta local | ☐ |
| 20 | Chat OpenAI-compatible con key real | Sin key en logs | ☐ |
| 21 | Chat Gemini con key de desarrollador | No cookies / no sesión web | ☐ |
| 22 | Autorizar `examples/` como workspace | Preview de árbol sin `.env` | ☐ |
| 23 | Intentar leer `.env` sintético de prueba | Debe bloquearse | ☐ |
| 24 | Dejar la app 8h en idle | RAM &lt; ~150 MB objetivo; sin leaks obvios | ☐ |
| 25 | Sleep/wake Windows | Recupera UI | ☐ |

## 4. Packs y CLI

| # | Acción | Hecho |
|---|--------|-------|
| 26 | `cargo run -p familiar-cli -- pack validate mascots/perrito-tech` | ☐ |
| 27 | `familiar pack build` y reimportar pack | ☐ |
| 28 | Crear mascota propia con Creator Studio + CLI | ☐ |
| 29 | Probar pack con `../` malicioso (debe fallar) | ☐ |
| 30 | Probar pack con `.js` o `.exe` (debe fallar) | ☐ |

## 5. Agentes y Permission Broker

| # | Acción | Hecho |
|---|--------|-------|
| 31 | Instalar al menos dos CLIs (p.ej. Gemini CLI + Codex u OpenCode) | ☐ |
| 32 | Modo Agent: lanzar tarea y ver pantallazo de aprobación | ☐ |
| 33 | Denegar y verificar que no ejecuta | ☐ |
| 34 | Allow session y reintento | ☐ |
| 35 | Revisar `audit.jsonl` en data dir | ☐ |
| 36 | Botón/flujo de cancelación de proceso | ☐ |

## 6. MCP y extensión

| # | Acción | Hecho |
|---|--------|-------|
| 37 | Exponer endpoint local documentado solo en loopback | ☐ |
| 38 | Empaquetar VSIX de la extensión y cargar en VS Code | ☐ |
| 39 | “Ask about selection” llega al Core sin API keys en la extensión | ☐ |

## 7. Hardening / supply chain

| # | Acción | Hecho |
|---|--------|-------|
| 40 | Revisar CI en GitHub Actions (js + rust + gitleaks) | ☐ |
| 41 | Generar SBOM (`cargo`, `pnpm`) y rellenar `THIRD_PARTY_NOTICES.md` | ☐ |
| 42 | Dependency audit (cargo deny / npm audit) | ☐ |
| 43 | Decidir política de firma de instaladores Windows | ☐ |
| 44 | Fuzz o corpus de manifiestos corruptos | ☐ |

## 8. Beta privada (Fase 11)

| # | Acción | Hecho |
|---|--------|-------|
| 45 | Usar diariamente con `_engine` y un monorepo grande sintético | ☐ |
| 46 | Plantilla de reporte de bugs redacted | ☐ |
| 47 | Checklist de desinstalación limpia | ☐ |
| 48 | Cero fugas de credenciales en logs exportados | ☐ |

## 9. Preparación pública (Fase 12)

| # | Acción | Hecho |
|---|--------|-------|
| 49 | Auditoría de historial Git (secrets, datos laborales) | ☐ |
| 50 | Sustituir cualquier path/dato real residual | ☐ |
| 51 | Confirmar autoría de cada asset | ☐ |
| 52 | `SUPPORT.md` + security advisories en GitHub | ☐ |
| 53 | Capturas y demo 60–90s | ☐ |
| 54 | Tag `v0.1.0` Public Beta (no v1.0.0) | ☐ |
| 55 | Hacer el repo público solo tras checklist | ☐ |

## 10. Pruebas que no automatizamos aún

- UX de burbuja de chat con DPI mixto
- Always-on-top vs juegos / Zoom a pantalla completa
- Credenciales en Windows Hello / múltiples usuarios
- Compatibilidad real de cada feature OpenAI-compatible en xAI
- Rendimiento con repos &gt; 50k archivos
- Accesibilidad (lector de pantalla) — post-MVP

## 11. Comandos de referencia

```powershell
cd "C:\Users\jorge gonzalez\Music\proyects\openfamiliar"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
.\scripts\bootstrap.ps1
.\scripts\test-all.ps1
pnpm --filter @openfamiliar/desktop dev
# cuando exista toolchain Tauri completa:
pnpm --filter @openfamiliar/desktop exec tauri dev
```

## 12. Contactos / cuentas a preparar

- GitHub: `georgiegozz/openfamiliar` (privado → público)
- Google AI Studio / Cloud: Gemini API key
- OpenAI o compatible + opcional xAI API key
- Ollama local
- (Opcional) npm org / crates.io owner cuando publiques paquetes

---

Última generación automática del scaffold: 2026-07-11.
Actualiza las casillas en tu copia local o en issues de GitHub.
