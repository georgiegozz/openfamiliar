# Roadmap

High-level phases from the project plan. Dates are estimates, not commitments.

| Phase | Focus | Status |
|-------|--------|--------|
| 0 | Public-ready foundation in private repo | **Done (scaffold)** |
| 1 | Architectural spikes + ADRs | **Done (ADRs + core spikes)** |
| 2 | Desktop shell (Tauri) | **Scaffolded** — needs MSVC + human UI soak |
| 3 | FamiliarKit + `.familiar` packs | **Done (schema + CLI + packs)** |
| 4 | Model router + chat | **Done (core adapters + mock)** |
| 5 | Workspace read-only context | **Done (crate + IPC)** |
| 6 | Agent CLI bridges | **Scaffolded** — needs real CLI install tests |
| 7 | Permission Broker + Agent mode | **Done (crate)** — UI approval pending |
| 8 | Local MCP + VS Code extension | **Scaffolded** — loopback HTTP pending |
| 9 | Familiar Creator Studio | **Scaffolded wizard** |
| 10 | Hardening + supply chain | **Partial** (CI + unit tests; SBOM human) |
| 11 | Private beta | **Operator** — see HUMAN_FOLLOWUP_MANUAL |
| 12 | Public preparation + v0.1.0 | **Operator** — see HUMAN_FOLLOWUP_MANUAL |

## Explicitly out of MVP

- Voice
- Autonomous screen capture
- Marketplace / gallery backend
- User accounts / remote OpenFamiliar backend
- Cross-device sync
- Terminal execution / file write (until Permission Broker gates + UI)
- macOS / Linux (post-MVP)
- Full automatic repo indexing
- Always-on background agents
- Web session / cookie scraping for ChatGPT, Gemini, SuperGrok
