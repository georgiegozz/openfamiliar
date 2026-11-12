# Architecture overview

Stable v0.1 is the Windows desktop app in `apps/desktop/`. It talks to a narrow
Rust IPC surface in `apps/desktop/src-tauri/` and runs one ephemeral Codex CLI
process at a time. Shared crates under `crates/` implement future core services
and are **not** started by that desktop process. See
[ADR-009](../adr/ADR-009-windows-one-shot-mvp.md).

The module and crate map is in [modules.md](./modules.md). Decisions live in
[docs/adr/](../adr/README.md).

```mermaid
flowchart TB
  subgraph stable["Stable Windows v0.1"]
    UI["apps/desktop React UI"]
    RT["packages/mascot-runtime"]
    PSDK["packages/provider-sdk types"]
    TAURI["openfamiliar-desktop Rust"]
    CODEX["Codex CLI process"]
    PACK["mascots/perrito-tech"]
    UI --> RT
    UI --> PSDK
    UI -->|"typed IPC only"| TAURI
    TAURI --> CODEX
    UI --> PACK
  end

  subgraph experimental["Experimental: present in the monorepo, not initialized by the stable desktop"]
    CORE["familiar-core"]
    STORAGE["familiar-storage"]
    PERMS["familiar-permissions"]
    CTX["familiar-context"]
    BRIDGE["familiar-agent-bridge"]
    MCP["familiar-mcp"]
    ADAPTERS["adapters/* stubs"]
    CORE --> STORAGE
    CORE --> PERMS
    CORE --> CTX
    PERMS --> STORAGE
    BRIDGE --> PERMS
    MCP --> CORE
  end

  subgraph tooling["Tooling and schemas"]
    FCLI["tools/familiar-cli"]
    SCH["packages/schemas"]
    ASSETS["scripts/assets"]
    FCLI --- SCH
    ASSETS --- PACK
  end

  subgraph scaffolds["Scaffolds, not part of the desktop release"]
    CS["apps/creator-studio"]
    VSC["apps/vscode-extension"]
  end
```
