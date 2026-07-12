# OpenFamiliar

**Build a companion. Connect your workspace.**  
**Crea una mascota. Conecta tu workspace.**

OpenFamiliar is a local-first desktop platform for creating and running AI-powered
desktop companions (“familiars”) that react to coding agents, chat with local or
official APIs, and request approval before any mutable action.

> OpenFamiliar is an independent open-source project and is not affiliated with,
> sponsored by, or endorsed by OpenAI, Google, xAI, Anthropic, Microsoft, Amazon,
> or their respective products.

| Component | Name |
|-----------|------|
| Desktop app | OpenFamiliar Desktop |
| Core | Familiar Core |
| Pet SDK | FamiliarKit |
| CLI | `familiar` |
| Pack format | `.familiar` |
| Editor extension | OpenFamiliar Companion |
| Starter mascot | Perrito Tech |

## Status

Private MVP foundation. Target platform for MVP: **Windows 11 x64**.

## Features (MVP scope)

- Transparent always-on-top mascot window
- States: idle, thinking, working, approval, success, error, …
- Text chat with streaming
- Providers: Ollama (local), OpenAI-compatible, Gemini native
- Explicit workspace selection + read-only context preview
- Secure API key storage (OS credential store abstraction)
- Import `.familiar` packs (no executable JS in v1)
- Local history, cancel, basic token budget
- Permission Broker for any future write/execute path

## Security modes

| Mode | Can | Cannot |
|------|-----|--------|
| **Chat** | Converse with selected model | Read workspace, run commands, write files |
| **Read-only** | Read authorized files, Git, active file | Write files, run mutable commands |
| **Agent** | Propose changes, show diffs, request execution | Act without explicit approval + local audit log |

## Repository layout

```
apps/           desktop, creator-studio, vscode-extension
crates/         familiar-core, context, permissions, storage, agent-bridge, mcp
packages/       mascot-sdk, provider-sdk, agent-sdk, schemas, ui
adapters/       ollama, openai-compatible, gemini, agent CLIs
mascots/        perrito-tech, blank-template
tools/          familiar-cli
examples/       synthetic demos only
docs/           architecture, ADRs, security, legal, guides
```

## Quick start (development)

### Prerequisites

- Windows 11 x64
- Node.js 20+
- pnpm 9+
- Rust stable + Cargo
- WebView2 (Windows)
- Optional: Ollama for local models

### Install

```powershell
pnpm install
cargo build --workspace
pnpm --filter @openfamiliar/desktop dev
```

### CLI

```powershell
pnpm --filter @openfamiliar/cli build
# or after install:
familiar pack validate mascots/perrito-tech
familiar pack inspect mascots/perrito-tech
```

### Tests

```powershell
pnpm test
cargo test --workspace
```

## License

- **Code**: [Apache License 2.0](./LICENSE)
- **Perrito Tech art**: CC-BY-4.0
- **Blank templates**: CC0-1.0

See [NOTICE](./NOTICE), [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md),
and [docs/legal/](./docs/legal/).

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).  
Security reports: [SECURITY.md](./SECURITY.md).  
Privacy: [PRIVACY.md](./PRIVACY.md).

## Roadmap

See [ROADMAP.md](./ROADMAP.md) and Architecture Decision Records under `docs/adr/`.
