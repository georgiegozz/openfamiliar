# ADR-005: Secure credential storage

- Status: Accepted
- Date: 2026-07-11

## Context

Users need API keys without storing them in plaintext config files.

## Decision

Store secrets via an OS credential abstraction (Windows Credential Manager first). Config files hold non-secret settings only. `.env` is for local dev examples and is gitignored.

## Consequences

- Desktop app owns secret I/O; extension never holds API keys.
- Headless CI uses env vars or mocks, never real keys in fixtures.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.

## Amendment — stable desktop keychain (2026-11-12)

Status: Accepted (authorized for implementation in the of-01–of-07 series).

The stable Windows desktop honors this ADR without initializing the experimental `familiar-storage` crate.

- Secrets for remote providers (starting with the Anthropic API key) are stored only in the OS keychain through `keyring`, service name `dev.openfamiliar.desktop`. Windows uses Credential Manager.
- `set_provider_secret` and `delete_provider_secret` are accepted only from the Settings window (Rust checks the webview label `settings`). Invoke arguments are never written to logs. After a successful save, the frontend discards the input; subsequent IPC replies only return `{ configured: true|false }`.
- `preferences.json` may store `activeProviderId` (`codex` | `anthropic` | `opencode` | `agy`) and never the secret.
- Tests and CI use `MemorySecretStore` and obviously fake fixture strings, never live keys.
- Release builds do not open WebView devtools. This series does not add a `devtools` flag to `tauri.conf.json` (devtools stay off unless a debug session explicitly opens them).
