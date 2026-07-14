# ADR-009: Windows One-Shot Codex MVP

- Status: Accepted
- Date: 2026-07-14

## Context

The monorepo contains experiments for providers, workspaces, agents, MCP, editor
integration, and creator tooling. Presenting those scaffolds as one desktop product
made the first public experience difficult to secure, test, and explain.

## Decision

Stable v0.1 is Windows 11 x64 only. It runs Perrito Tech in a transparent Tauri
window and submits one independent question through an existing authenticated
Codex CLI installation. Each request is a fresh ephemeral read-only process with
bounded input/output, timeout, process-tree cancellation, and no history.

The stable desktop does not initialize experimental core/storage/provider/agent
crates. Experimental packages remain in the monorepo for future ADRs and are not
visible in the stable UI.

## Consequences

- The public promise is narrow, demonstrable, and locally testable.
- Codex CLI installation/authentication remains an operator prerequisite.
- Tests use a fake CLI and consume no service quota.
- Additional providers or agent behavior require a future explicit decision and
  must not reuse the stable one-shot boundary implicitly.
