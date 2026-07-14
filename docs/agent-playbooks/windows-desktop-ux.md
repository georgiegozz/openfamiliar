# Windows Desktop UX Playbook

## Use When

Changing Tauri windows, tray actions, Quick Ask, drag, click-through, settings,
monitor placement, DPI behavior, keyboard interaction, or accessibility.

## Inputs

- The requested interaction and stable v0.1 acceptance criteria.
- Current Tauri config/capabilities, React window, typed backend, and Rust command.

## Steps

1. Confirm the change belongs to `mascot` or `settings`; do not merge the two.
2. Keep the mascot fixed-size, transparent, taskbar-hidden, and decoration-free.
3. Use a 4–6 px drag threshold and suppress the following click.
4. Persist physical position, monitor name, and scale factor; clamp to work area.
5. Preserve tray recovery before enabling click-through.
6. Add keyboard, focus-visible, reduced-motion, loading, and error behavior.
7. Test browser state logic, then the Tauri path.

## Acceptance Criteria

- No abrupt mascot-window resizing or technical panel.
- Click, drag, right-click, Enter, Shift+Enter, Escape, Cancel, and Copy work.
- Disconnected monitors and negative coordinates cannot strand the mascot.

## Validation

```powershell
pnpm --filter @openfamiliar/desktop typecheck
pnpm --filter @openfamiliar/desktop test
cargo test -p openfamiliar-desktop
```

## Limits and Anti-patterns

Do not add remote content, polling, provider/model selectors, workspace controls,
history, generic IPC, fractional sprite scaling, or a click-through dead end.

Relevant files: `apps/desktop/src/windows`, `apps/desktop/src/features`,
`apps/desktop/src-tauri/src/commands`, `monitor_position.rs`, `tauri.conf.json`.
