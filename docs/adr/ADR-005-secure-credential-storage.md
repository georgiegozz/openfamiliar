# ADR-005: Secure credential storage

- Status: Accepted
- Date: 2026-07-11

## Context

Users need API keys without storing them in plaintext config files.

## Decision

Store secrets via an OS credential abstraction (Windows Credential Manager first). Config files hold non-secret settings only. `.env` is for local dev examples and is gitignored.

## Consequences

- Desktop app owns secret I/O; extension never holds API keys.
- Headless CI uses env vars or mocks, never real keys in fixtures.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
