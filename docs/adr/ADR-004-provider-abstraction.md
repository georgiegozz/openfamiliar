# ADR-004: Provider abstraction

- Status: Accepted
- Date: 2026-07-11

## Context

UI must not depend on a single AI vendor.

## Decision

Define a `ModelProvider` contract with `validateConfiguration`, `listModels`, `stream`, `cancel`, and optional `estimateCost`. Initial adapters: `ollama-local`, `openai-compatible`, `gemini-native`.

## Consequences

- xAI can be configured via OpenAI-compatible base URL after capability testing.
- Secrets never appear in provider logs.
- Fallback between providers is optional and never automatic by default.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
