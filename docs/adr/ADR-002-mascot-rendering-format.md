# ADR-002: Mascot rendering format

- Status: Accepted
- Date: 2026-07-11

## Context

Mascots must animate states (idle, thinking, working, …) without shipping a heavy game engine.

## Decision

v1 rendering uses **WebP animations** (or static WebP frames) referenced from `familiar.json` state map. Spritesheets are supported at authoring time and converted to WebP in Creator Studio.

## Consequences

- Simple validation and MIME checks.
- No Live2D runtime in MVP (avoids third-party model license complexity).
- Packs stay content-only; no executable animation scripts in v1.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
