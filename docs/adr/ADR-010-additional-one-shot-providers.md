# ADR-010: Additional one-shot providers

- Status: Proposed
- Date: 2026-11-12

## Context

[ADR-009](./ADR-009-windows-one-shot-mvp.md) limits stable v0.1 to one
independent Codex CLI question at a time. [ADR-004](./ADR-004-provider-abstraction.md)
already defines an experimental `ModelProvider` contract (Ollama, OpenAI-compatible,
Gemini). [ADR-005](./ADR-005-secure-credential-storage.md) already requires OS
credential storage and forbids plaintext keys in config files.

On the current `main` tree those experimental crates are not initialized by the
stable desktop. The desktop has no Anthropic client, no Settings key UI, and no
IPC for OpenCode or Antigravity. `familiar-agent-bridge` has OpenCode argv
`opencode run <task>` and an Antigravity scaffold that launches a program named
`antigravity` with the task as its only argument.

The maintainer authorized a product expansion: additional **one-shot**
providers may be added to the desktop, still behind an explicit decision, not
by silently reusing the Codex boundary.

## Decision

This ADR is proposed, not accepted. When accepted, the stable desktop may offer
more than Codex as a one-shot ask path, with the same product limits as ADR-009:
one independent question, no conversation history, no generic shell, and no
workspace or agent mode in the stable UI.

1. **Anthropic / Claude** is implemented in the desktop Rust host, not in the
   webview. The request is a single Messages API call. In release builds the
   base URL is fixed to `https://api.anthropic.com`, TLS is required, and
   redirects are disabled so the key cannot follow a redirect to another host.
   The key is read only from the OS credential store. The UI may show whether
   a key is configured, never the secret itself. API error bodies are not
   logged or shown verbatim.
2. **Secrets** stay in the OS credential store, consistent with ADR-005. The
   stable desktop should not initialize the entire experimental
   `familiar-storage` crate only to store a key; it may use a dedicated
   keyring helper with a desktop-specific service name. Tests use a mock
   store. No real keys appear in fixtures, preferences JSON, or logs. Secret
   IPC commands, if added, must reject calls that do not come from the
   `settings` window (checked in Rust by window label) and must not log
   invoke arguments.
3. **OpenCode and Antigravity** follow the Codex CLI pattern: detect a binary
   on `PATH`, run a one-shot argv as separate process arguments, use that
   CLI's own login, and never read its credential files. The intended one-shot
   argv is `opencode run <prompt>` and `agy -p <prompt>`. The current
   `antigravity` scaffold does not match that argv and must be corrected
   before it is wired into the desktop. `.cmd` / `.bat` shims are rejected, or
   the real `.exe` is resolved, so arguments are not reinterpreted by `cmd.exe`.
   Tool use is denied or the process is launched in an empty temporary
   directory. This provider path is not merged until it is tested on a real
   Windows machine.
4. This expansion does not enable MCP, workspace context, Creator Studio, or
   the VS Code extension in the stable UI. It does not by itself change
   capabilities, the CSP, or CI workflows.

## Consequences

- ADR-009 remains the description of shipping v0.1 until this ADR is accepted
  and the corresponding code ships.
- Provider HTTP from the webview stays forbidden; `connect-src` remains IPC
  only.
- Adding Anthropic does not add a configurable release base URL. A
  configurable base URL would send the key to an arbitrary host.
- OpenCode and Antigravity are agent-capable CLIs. One-shot wrapping is a
  product choice that still needs a Windows proof, not only unit tests with
  fakes.
- ADR-004's experimental streaming `ModelProvider` path stays experimental.
  This ADR extends the **one-shot** desktop boundary, not the streaming
  router, and does not start reading `OPENFAMILIAR_*` environment variables in
  the stable app.

## Alternatives considered

- Routing Claude, OpenCode, and Antigravity through `familiar-core`'s
  `ModelProvider` and its environment-variable router: rejected for the
  stable desktop, because ADR-009 keeps those crates uninitialized there.
- Calling provider HTTP from the webview or adding a TypeScript AI SDK:
  rejected, because it would require CSP / `connect-src` changes and would
  place secrets closer to the UI.
- Storing keys in preferences or `.env`: rejected by ADR-005.
- Shipping the Antigravity scaffold as-is (`antigravity` + `[task]`):
  rejected; it does not match the intended one-shot CLI.
