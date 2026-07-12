# Threat model (MVP)

## Assets

- API keys
- Workspace source code
- Chat history
- Audit logs
- Imported packs

## Actors

- Local user
- Malicious pack author
- Compromised model provider
- Malicious MCP client on same machine
- Accidental path traversal / symlink escape

## Controls

- Pack data-only validation (ADR-003)
- Explicit workspace roots + canonicalization
- Sensitive path blocklist
- Context preview + token budget
- Permission Broker for mutable ops
- Secrets in OS keystore
- Local MCP only
- Audit JSONL
- Emergency cancel

## Non-goals (MVP)

- Multi-user OS isolation beyond standard user account
- Remote attestation of packs marketplace (no marketplace yet)