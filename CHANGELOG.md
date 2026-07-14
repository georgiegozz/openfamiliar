# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
