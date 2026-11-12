---
name: openfamiliar-add-provider
description: Add or review an OpenFamiliar model or CLI provider. Use when introducing Anthropic, an OpenAI-compatible host, a CLI such as OpenCode or Antigravity, or when changing provider IPC, credentials, or adapter stubs.
---

# Add a provider

There are two different layers. Do not mix them.

1. **Stable desktop one-shot** (what v0.1 actually runs): Rust in
   `apps/desktop/src-tauri/`, typed IPC in `apps/desktop/src/lib/backend.ts`,
   and `OneShotProvider` types in `packages/provider-sdk`. Today only Codex is
   wired (`detect_codex`, `ask_codex`, `cancel_codex`). Adding another
   one-shot provider requires [ADR-010](../../../docs/adr/ADR-010-additional-one-shot-providers.md)
   to be accepted, or an equivalent explicit product decision. Follow the
   Codex skill at `../openfamiliar-codex-adapter/SKILL.md` for process
   isolation, fakes, and logging.
2. **Experimental scaffolds** (not started by the stable desktop):
   `ModelProvider` in `crates/familiar-core/src/providers.rs` (Ollama,
   OpenAI-compatible, Gemini), `CredentialStore` in `familiar-storage`,
   CLI adapters in `crates/familiar-agent-bridge`, and one-line stubs under
   `adapters/`.

## Workflow

1. Read [ADR-009](../../../docs/adr/ADR-009-windows-one-shot-mvp.md),
   [ADR-004](../../../docs/adr/ADR-004-provider-abstraction.md),
   [ADR-005](../../../docs/adr/ADR-005-secure-credential-storage.md), and
   ADR-010. If the change would surface a new provider in the stable UI or
   IPC, stop unless that ADR is accepted.
2. Reuse an existing contract. For a one-shot CLI, follow `CodexService`.
   For a hosted HTTP API, call it from Rust only; do not `fetch` from the
   webview and do not widen `connect-src`.
3. Store API keys only in the OS credential store. The UI may show
   configured / not configured. Never write the secret to preferences, logs,
   fixtures, or command output. CLI products that already have their own
   login (Codex, OpenCode, Antigravity) must keep that login; do not read
   their auth files.
4. Use exact dependency versions and check
   [allowed-licenses.md](../../../docs/legal/allowed-licenses.md). After a
   lockfile change, regenerate the inventory on Windows
   (`../openfamiliar-fix-license-inventory/SKILL.md`).
5. Do not change capabilities, the CSP, or `.github/workflows/` unless the
   pull request justifies it.
6. Add tests that use mocks, `httpmock` (dev-dependency only, license
   checked), or a fake executable. Never call a real network API in CI.

## PowerShell

```powershell
# Existing experimental router tests, if you touch familiar-core:
cargo test -p familiar-core providers
# Agent-bridge argv tests, if you touch a CLI adapter:
cargo test -p familiar-agent-bridge
# Stable desktop Codex tests (pattern to copy for a new one-shot CLI):
cargo test -p openfamiliar-desktop services::codex_process::tests
pnpm --filter @openfamiliar/provider-sdk test
```

## Python

Use this only as a review helper. It is not a product test.

```python
from pathlib import Path
import re

root = Path(".")
needles = ("api_key", "sk-", "x-api-key", "OPENFAMILIAR_")
forbidden = []
for path in list(root.glob("**/*test*")) + list(root.glob("**/fixtures/**")):
    if any(part in path.parts for part in (".git", "node_modules", "target")):
        continue
    if path.suffix.lower() not in {".rs", ".ts", ".tsx", ".json", ".md", ".ps1"}:
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        continue
    if any(n.lower() in text.lower() for n in needles):
        forbidden.append(path)
print("inspect fixtures that mention key-like strings:")
for path in forbidden:
    print(path)
```

## Acceptance checks

```powershell
cargo test -p <crate-you-changed>
pnpm --filter @openfamiliar/provider-sdk test
pnpm licenses:check   # only after running the inventory on Windows
git diff -- apps/desktop/src-tauri/capabilities/default.json apps/desktop/src-tauri/tauri.conf.json .github/workflows
```

The last command must be empty, or the pull request must justify the change.

## Limits

- Do not log invoke arguments for secret commands.
- Do not make the Anthropic base URL configurable in release builds.
- Do not launch OpenCode or Antigravity through `cmd.exe` / a shell string.
- Do not claim a CLI provider is merge-ready without a real Windows run.
