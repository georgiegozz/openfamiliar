# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.x (pre-1.0) | Best-effort on latest `main` / latest pre-release |

## Reporting a vulnerability

Do **not** open a public issue for security vulnerabilities.

Email or private channel (to be configured before public launch):

- Prefer GitHub Security Advisories once the repository is public.
- Until then, contact the repository owner privately.

Please include:

- Description and impact
- Reproduction steps
- Affected component (core, pack reader, MCP, desktop, CLI, extension)
- Whether credentials or workspace data could leak

We aim to acknowledge reports within 7 days.

## Security principles

- Local-first: no mandatory remote OpenFamiliar backend.
- Explicit workspace authorization.
- Context must be previewable before send.
- No scraping of web sessions or cookies for providers.
- API keys only via official developer credentials + OS secret store.
- Chat / Read-only / Agent modes with least privilege.
- All mutable ops require Permission Broker approval + local audit log.
- Packs: no executable JS/binaries in v1; path/MIME/size validation.
- Telemetry off by default.

## Hardening checklist (release)

See `docs/security/threat-model.md` and Phase 10 hardening tasks.
