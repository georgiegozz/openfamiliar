# Windows MVP Human Smoke Test

Run this checklist on Windows 11 x64 using a clean local user or disposable VM.
Record build hash, Windows build, scale, monitor layout, Codex version, installer
type, and result. Automated tests must not call real Codex; the real CLI is used
only in the explicitly marked human steps.

## Setup

- [ ] Install the MSI or NSIS package for the current build.
- [ ] Confirm no development checkout or Node/Rust process is running.
- [ ] Test once with Codex CLI authenticated and once with Codex unavailable.

## Startup and Visuals

- [ ] Launch from Start. Perrito Tech appears without console or technical panel.
- [ ] With default scale, the compact input window is 152×152 px and the sprite
      viewport is 128×128 px (one third smaller than the former 192 px default).
- [ ] Window background and sprite edges are truly transparent.
- [ ] Sprite pixels remain crisp at 100%, 125%, and 150% Windows scale.
- [ ] Leave the pointer and keyboard untouched for 60 seconds: position, frame,
      head, and both eyes remain unchanged; there is no random blink/look/sleep.
- [ ] Both eyes remain symmetric during idle and request-state transitions.
- [ ] Listening, thinking, answering, success, error, wake, and dragging frames
      are visually sane and appear only in response to an explicit event.
- [ ] Every stable event frame looks friendly and collar-free.
- [ ] Teal, midnight, and burgundy palettes change only small state props without
      changing coat, white markings, eye geometry, neck, transparency, or frame alignment.
- [ ] Idle operation produces no network traffic and remains near 0–1% CPU after soak.

## Interaction

- [ ] Single click toggles Quick Ask.
- [ ] Drag starts only from the visible handle after about 5 px and does not open Quick Ask.
- [ ] The drag handle is no larger than 48×24 px; dragging elsewhere is disabled.
- [ ] Empty transparent pixels outside compact 152×152 do not intercept other apps.
- [ ] Right-click opens the small menu; Settings opens the normal settings window.
- [ ] Enter submits; Shift+Enter creates a line; Escape closes/cancels.
- [ ] Response scrolls, basic fenced code is readable, Copy works, New clears all content.
- [ ] Closing and reopening does not restore the prior question or answer.

## Codex CLI — Human/Quota-Bearing

- [ ] With Codex missing, the mascot remains usable and Settings shows onboarding.
- [ ] With authenticated compatible Codex, one harmless question returns a Unicode answer.
- [ ] Start a slow request and Cancel; process and children disappear from Task Manager.
- [ ] Configure a short timeout and confirm a safe timeout error with no orphan process.
- [ ] Confirm local logs contain event categories only, not question, answer, stdout, stderr, or token.

## Tray, Preferences, and Lifecycle

- [ ] Tray contains Ask Codex, Settings, Always on top, Click-through, Reset position,
      About, and Quit.
- [ ] Enable Click-through, then recover interaction using tray Ask.
- [ ] Restart resets unsafe click-through while preserving allowed preferences.
- [ ] Always-on-top, scale, animation, reduced motion, language, timeout, and startup persist.
- [ ] Click Save twice with startup disabled and its registry entry absent; both
      saves succeed without `os error 2`.
- [ ] Change scale or timeout, save, reopen Settings, and confirm the value persisted.
- [ ] Quit during a request removes all Codex child processes and exits all windows.

## Monitor and DPI

- [ ] Persist and restore a position at 100%, 125%, and 150% scale.
- [ ] Drag to a monitor left of primary with negative coordinates and restart.
- [ ] Disconnect that monitor; mascot returns inside primary work area above taskbar.
- [ ] Change the primary monitor and use Reset position.
- [ ] Suspend/resume Windows; animation recovers without a timer storm or high CPU.

## Install Lifecycle

- [ ] Uninstall removes the application and startup entry without deleting unrelated files.
- [ ] Reinstall launches successfully and recreates only app-local config/log directories.
- [ ] Neither installer, runtime config, nor logs contain a developer checkout path.

## Result

- Build/commit:
- Installer and SHA-256:
- Environment:
- Passed:
- Failed/blockers:
- Evidence location:
