# ADR-004: Provider abstraction

- Status: Accepted
- Date: 2026-07-11

## Context

UI must not depend on a single AI vendor.

## Decision

Define a `ModelProvider` contract with `validateConfiguration`, `listModels`, `stream`, `cancel`, and optional `estimateCost`. Initial adapters: `ollama-local`, `openai-compatible`, `gemini-native`.

## Consequences

- xAI can be configured via OpenAI-compatible base URL after capability testing.
- Secrets never appear in provider logs.
- Fallback between providers is optional and never automatic by default.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.

## Amendment — one-shot providers in the stable desktop (2026-11-12)

Status: Accepted (authorized for the of-01–of-07 series).

Stable v0.1 remains a one-shot Quick Ask surface. In addition to the existing Codex CLI path, the desktop may expose these one-shot providers, each selected in Settings as `activeProviderId`:

- `codex`: existing local Codex CLI (`codex exec`), no API key in our keychain.
- `anthropic`: Claude via the official Messages API. The webview never `fetch`es the provider; Rust performs HTTPS to the fixed base `https://api.anthropic.com` (implemented in a follow-up PR). The API key lives only in the OS keychain (ADR-005).
- `opencode`: local OpenCode CLI one-shot (`opencode run`), using the operator's own CLI session. OpenFamiliar does not read that CLI's credentials.
- `agy`: local Antigravity CLI one-shot (`agy -p`), same boundary as Codex/OpenCode.

Conversation history, MCP listeners, ACP, agent mode, and additional hosted providers stay out of this stable path (see ADR-009 and the roadmap). Fallback between providers is never automatic.
