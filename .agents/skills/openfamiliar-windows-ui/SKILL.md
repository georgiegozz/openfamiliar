---
name: openfamiliar-windows-ui
description: Implement or review OpenFamiliar's Tauri 2 Windows UI, including transparent mascot behavior, Quick Ask, settings, tray actions, drag handling, monitor-safe placement, accessibility, and recovery from click-through. Use for changes under apps/desktop that affect visible Windows interactions or Tauri window configuration.
---

# OpenFamiliar Windows UI

## Contract

Keep the stable desktop MVP focused on two windows: `mascot` and `settings`.
The mascot is transparent, decoration-free, fixed-size, always-on-top by
preference, and absent from the taskbar. Settings is a conventional hidden
window opened on demand.

Do not add provider/model selectors, workspace authorization, generic shell
access, chat history, or an agent mode to the stable UI.

## Inputs

- Requested Windows interaction and its stable v0.1 acceptance criteria.
- Current Tauri config/capabilities, React window, typed backend, and Rust command.

## Workflow

1. Read `apps/desktop/src-tauri/tauri.conf.json`, the affected React window,
   `apps/desktop/src/lib/backend.ts`, and the matching Rust command.
2. Preserve the narrow typed IPC surface. Add a purpose-specific command only
   when frontend behavior cannot stay local.
3. Make drag/click gestures unambiguous: use a movement threshold and suppress
   the click following a drag.
4. Persist physical coordinates with monitor name and scale factor. Restore by
   clamping to current monitor work areas, including negative coordinates.
5. Keep a recovery route for click-through through the tray's Ask action.
6. Add or update reducer/unit tests, then run desktop typecheck, test, and the
   Rust desktop tests.

## UI Rules

- Keyboard: Enter submits, Shift+Enter inserts a line, Escape closes/cancels.
- Never leave a loading state without a visible Cancel action.
- Use semantic labels, focus-visible styles, and `prefers-reduced-motion`.
- Do not fetch remote fonts or images. The CSP must remain restrictive.
- Browser preview may mock IPC, but the Tauri build is the release authority.

## Acceptance Criteria

- Mascot and Settings remain separate windows with no abrupt resize.
- Drag cannot trigger click, and click-through is recoverable from the tray.
- Keyboard, monitor/DPI restore, reduced motion, loading, and errors are covered.

## Limits and Anti-patterns

Do not add remote assets, polling, generic IPC, history, experimental controls,
fractional sprite scaling, or a click-through state without a recovery route.

Relevant files: `apps/desktop/src/windows`, `apps/desktop/src/features`,
`apps/desktop/src-tauri/src/commands`, `monitor_position.rs`, `tauri.conf.json`.

## Validation

```powershell
pnpm --filter @openfamiliar/desktop typecheck
pnpm --filter @openfamiliar/desktop test
cargo test -p openfamiliar-desktop
```
