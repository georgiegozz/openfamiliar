# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security

- Bump the desktop Tauri crate from 2.11.5 to 2.11.6 (GHSA-w28w-mhc8-qvjv). This is the minimum patched 2.11 release; 2.12 was not taken because 2.12.0 changes Android `$VIDEO` / `video_dir()` and `tauri-build` 2.7 declares rust-version 1.90. `@tauri-apps/cli` moves to 2.11.5 to stay on the 2.11 line with `@tauri-apps/api` 2.11.1 and `tauri-build` 2.6.3.
- Raise workspace `rust-version` from 1.80 to 1.88 so `familiar-cli` can take `zip` 9.7.3 with only the `deflate` feature (the floor that patches RUSTSEC-2026-0369 / 0367 / 0368 / 0429 is 9.0.1). 9.7.3 is the newest 9.x that still declares rust-version 1.88; 9.8.0 would require 1.89. Default `zip` features were disabled because they pull `aes` 0.9.4 (rust-version 1.89). `zstd` / `zstd-safe` / `zstd-sys` leave the graph with that change.
- Refresh transitive Rust TLS and QUIC crates (`rustls` 0.23.46, `rustls-webpki`, `webpki-roots`, `quinn` / `quinn-proto` / `quinn-udp` 0.5.16), `tokio` 1.54.1, and `rustix` 1.1.6 to clear the other patched advisories in issue #31 plus RUSTSEC-2026-0538 and unsound RUSTSEC-2026-0486 found by a later `cargo audit`.
- Bump Vite to 6.4.6 and override vulnerable transitive JavaScript packages listed in issue #31 (`nanoid`, `postcss`, `browserslist`, `source-map-js`, `launch-editor`).
- `rusqlite` 0.32.1 / `libsqlite3-sys` 0.30.1 (RUSTSEC-2026-0459) stays open: the first `rusqlite` that takes `libsqlite3-sys` >= 0.39 is 0.41.0, which declares rust-version 1.89. See `docs/security/dependency-advisories.md`.

### Added

- Agent workflow documentation: module and crate map, local and CI
  verification (including the Windows license-inventory sort order), a pull
  request checklist, repo skills for adding a provider, adding a mascot pack,
  and fixing the license inventory, and proposed
  [ADR-010](docs/adr/ADR-010-additional-one-shot-providers.md) for additional
  one-shot providers

- Original, happy, collar-free dark-brindle Perrito Tech spritesheet based on
  operator-owned dog references, with deterministic teal, midnight, and burgundy
  variants limited to small state-prop accents
- Compact/expanded mascot window states and a dedicated 48×24 drag handle
- Deterministic dependency-license inventory and Dependabot configuration
- Functional/technical public-beta retrospective and explicit release gates

- Repository foundation: Apache-2.0 LICENSE, NOTICE, governance docs
- Cargo + pnpm monorepo skeleton
- Familiar Core crates scaffolding
- FamiliarKit schemas and pack validation
- Provider SDK contracts (Ollama, OpenAI-compatible, Gemini)
- Desktop shell scaffolding (Tauri 2 + React)
- CLI pack tooling scaffolding
- VS Code extension scaffolding
- Creator Studio scaffolding
- ADRs 001–008
- Synthetic examples and demo workspace data
- Personal operator follow-up manual and private tracking (gitignored)

### Changed

- Reduced default mascot display from 192 px to 128 px with crisp integer scaling
- Made idle visually static; animations now run only for explicit UI/request events
- Reworked animation mappings so both eyes remain symmetric in stable states
- Restricted the transparent input surface from 560×360 to 152×152 while idle
- Updated privacy, security, contribution, and third-party documentation to match
  the implemented Windows MVP instead of future scaffolds

### Fixed

- Settings no longer fails with Windows `os error 2` when saving while the
  disabled startup registry entry is already absent
- Preference storage creates its parent directory and keeps in-memory state
  unchanged when a disk write fails
- Detection of the official npm-installed Codex Windows native binary
- Pack manifest camelCase deserialization and variant-asset validation
- Mascot drag state rendering and position persistence from trusted window bounds
