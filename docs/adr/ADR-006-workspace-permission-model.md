# ADR-006: Workspace permission model

- Status: Accepted
- Date: 2026-07-11

## Context

The mascot must understand code without exfiltrating entire disks or repos.

## Decision

Workspaces require **explicit user selection**. Modes: Chat (no FS), Read-only, Agent (via Permission Broker). Paths are canonicalized; `.gitignore` and `.openfamiliarignore` apply; sensitive globs are blocked; context is previewed with token estimates.

## Consequences

- Never send whole workspace by default.
- History of files sent is retained locally.
- Symlinks resolved; escape outside workspace rejected.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
