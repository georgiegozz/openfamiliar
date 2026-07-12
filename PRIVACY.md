# Privacy Policy (product intent)

OpenFamiliar is designed as a **local-first** desktop application.

## What stays on your machine

- Chat history (local SQLite)
- Audit logs (local JSONL)
- Workspace paths you authorize
- Pack files you import
- Configuration (non-secret)

## Secrets

API keys are stored using the operating system credential store abstraction
(Windows Credential Manager / Keychain-equivalent). Keys are never written to
logs, screenshots, or pack files.

## Network

OpenFamiliar only contacts endpoints you configure:

- Local Ollama
- Official provider APIs (OpenAI-compatible, Gemini, etc.)
- Agent CLIs you launch

There is **no** OpenFamiliar cloud account in the MVP and **no** automatic
upload of workspace files.

## Telemetry

Telemetry is **disabled by default**. If optional diagnostics are added later,
they will require explicit opt-in and will not include file contents or secrets.

## Beta / private testing

- No automatic log shipping
- Voluntary reports only
- Use the secret-redaction checklist before sharing diagnostics
  (`docs/guides/share-diagnostics.md`)

## Context sent to models

Only content you select or that falls under an explicit context budget is sent.
The UI shows a context preview before sending when workspace mode is enabled.

This document describes product intent for the open-source project and is not
legal advice for every deployment scenario.
