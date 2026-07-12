# ADR-003: Plugin isolation

- Status: Accepted
- Date: 2026-07-11

## Context

Third-party mascot packs must not become an RCE vector.

## Decision

In v1, packs are **data only**: JSON manifest, markdown personality, media assets. **No JavaScript**, **no binaries**, no absolute paths, no `..` traversal. Validation enforces size, MIME, and per-file hashes.

## Consequences

- Lower attack surface.
- Personality is prompt text, not code.
- Future executable plugins would need a new major ADR and sandbox design.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
