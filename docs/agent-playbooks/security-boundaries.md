# Stable MVP Security Boundaries

## Use When

Reviewing any stable desktop change that touches IPC, processes, files, network,
logs, preferences, capabilities, CSP, dependencies, or experimental scaffolds.

## Inputs

- Proposed data flow and exact frontend/Rust boundary.
- `AGENTS.md`, capability JSON, CSP, AppPaths, logger, preferences, and adapter.

## Steps

1. Classify data as public metadata, non-sensitive preference, sensitive content,
   credential, process output, or experimental capability.
2. Keep prompts/answers in memory only and credentials outside the application.
3. Reject generic commands and validate request IDs, prompt length, timeout, path,
   output size, UTF-8, and enum values at the Rust boundary.
4. Confirm idle operation starts no listener, remote request, MCP, workspace scan,
   provider, agent bridge, or experimental crate.
5. Keep CSP restrictive and Tauri capabilities minimal.
6. Test failures for safe messages and category-only logs.

## Acceptance Criteria

- No secrets, personal paths, `.env` runtime loading, auth scraping, loopback HTTP,
  remote content, prompt history, or raw process output in durable storage.
- Codex is restricted to ephemeral read-only one-shot requests.

## Validation

```powershell
rg -n -i "C:\\Users\\|tauri-diag|csp.*null|auth\.json|api[_-]?key|console\.log" .
cargo test -p openfamiliar-desktop
pnpm --filter @openfamiliar/desktop build
```

## Limits and Anti-patterns

Never weaken controls to improve demo convenience. Do not add a provider, model,
workspace, shell, agent, MCP, or API-key UI to stable v0.1.

Relevant files: `AGENTS.md`, `apps/desktop/src-tauri`, `apps/desktop/src/lib`,
`docs/security`, and `docs/adr/ADR-009-windows-one-shot-mvp.md`.
