# ADR-001: Desktop framework

- Status: Accepted
- Date: 2026-07-11

## Context

OpenFamiliar needs a lightweight always-on-top transparent desktop companion on Windows 11 x64, with a modern UI and access to OS features (tray, credentials, subprocess).

## Decision

Use **Tauri 2** with a **React + TypeScript** frontend and **Rust** for Familiar Core.

## Consequences

- Small binary and low idle RAM vs Electron.
- Requires Rust + WebView2 toolchain on Windows.
- Core logic can be shared as crates outside the UI shell.
- macOS/Linux deferred to post-MVP but stack is multi-platform ready.

## Alternatives considered

Documented in plan.txt product analysis (OpenPets, CoPet, DeskPet, Clawd, etc.). Implementation remains original.
