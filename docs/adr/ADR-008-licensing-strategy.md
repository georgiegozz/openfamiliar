# ADR-008: Licensing strategy

- Status: Accepted
- Date: 2026-07-11

## Context

Project must stay contribution-friendly without brand entanglement or AGPL contamination.

## Decision

- Code: **Apache-2.0**
- Official Perrito Tech art: **CC-BY-4.0**
- Blank templates: **CC0-1.0**
- Third-party packs: declare SPDX license; gallery requires license; local import of unlicensed packs is allowed with warning and `NOASSERTION`.
- GPL/AGPL blocked for core by default; no unlicensed code copy.

## Consequences

- Clean NOTICE / THIRD_PARTY_NOTICES discipline.
- Trademark-safe product naming (no vendor mascot product names).

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
