# Security Policy

## Supported versions

| Version       | Supported                                         |
| ------------- | ------------------------------------------------- |
| 0.x (pre-1.0) | Best-effort on latest `main` / latest pre-release |

## Reporting a vulnerability

Do **not** open a public issue for security vulnerabilities. Prefer GitHub
Security Advisories when private reporting is enabled; otherwise contact the
repository owner privately.

Please include the impact, reproduction steps, affected component, and whether
credentials, prompt content, or local data could be exposed. Reports should be
acknowledged within seven days.

## Stable MVP security boundary

- The Windows desktop has no OpenFamiliar cloud backend or telemetry.
- A Quick Ask starts one fresh Codex CLI process only after an explicit question.
- The process is read-only and ephemeral, with input/output limits, timeout,
  cancellation, and process-tree cleanup.
- OpenFamiliar does not read Codex authentication files, browser sessions,
  cookies, API keys, `.env` files, or workspace content.
- Questions, answers, stdout, and stderr are not written to application logs.
- Logs contain event categories only; preferences contain non-secret UI settings.
- Stable Tauri IPC does not expose a generic shell, arbitrary command,
  user-controlled CLI arguments, provider selection, workspace access, or agents.
- Familiar packs are declarative assets; pack validation rejects missing or
  invalid paths and no pack may contain executable code.

Experimental crates and future designs are not part of this stable boundary.
See `docs/security/threat-model.md` and
`docs/agent-playbooks/security-boundaries.md`.
