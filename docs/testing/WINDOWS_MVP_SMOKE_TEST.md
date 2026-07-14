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
- [ ] Window background and sprite edges are truly transparent.
- [ ] Sprite pixels remain crisp at 100%, 125%, and 150% Windows scale.
- [ ] Idle, blink/look, thinking, success, error, sleep, and wake are visually sane.
- [ ] Idle operation produces no network traffic and remains near 0–1% CPU after soak.

## Interaction

- [ ] Single click toggles Quick Ask.
- [ ] A movement of about 5 px starts drag and does not also open Quick Ask.
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
