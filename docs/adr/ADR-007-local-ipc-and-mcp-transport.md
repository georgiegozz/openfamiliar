# ADR-007: Local IPC and MCP transport

- Status: Accepted
- Date: 2026-07-11

## Context

Editors and agents need to talk to Familiar Core without reimplementing providers.

## Decision

- Desktop ↔ Core: Tauri commands/events.
- External tools: **local MCP** transport only (no public network bind by default).
- VS Code extension is thin: selection/file/diff → Core.

## Consequences

- Keys stay in Core/desktop.
- MCP resources use `familiar://` URIs.
- Remote MCP exposure is explicitly out of scope for MVP.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
