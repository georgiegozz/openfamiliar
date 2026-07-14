# AGENTS.md — OpenFamiliar

These instructions bind all automated contributors in this repository.

## Product Boundary

- Target Windows 11 x64 first.
- Stable v0.1 is Perrito Tech, a transparent pixel-art desktop companion with
  one independent Codex CLI question at a time.
- Keep other providers, workspaces, agents, MCP, Creator Studio, and the VS Code
  extension experimental. Do not initialize or expose them in the stable build.
- Prefer small reuse-first changes. Do not rewrite the monorepo or duplicate an
  existing abstraction.

## Security and Privacy

- Never hardcode a user, machine, checkout, executable, log, or secret path.
- Never read Codex auth files, browser sessions, cookies, tokens, or credentials.
- Never log prompts, answers, full stdout/stderr, credentials, or tokens.
- Stable IPC must expose purpose-specific commands only. Never add generic shell,
  command, path, argument, provider, workspace, or agent execution.
- Codex questions must be ephemeral, read-only, timeout-bounded, output-capped,
  cancellable, and process-tree terminated on Windows.
- Automated tests must use the fake Codex fixture and must never call real Codex.

## Visual and Asset Rules

- Preserve pixel-perfect integer scaling and nearest-neighbor rendering.
- Do not add circular avatar crops, permanent glow rings, technical state badges,
  fractional scaling, or remote visual assets to the stable mascot.
- Do not copy third-party characters, sprites, logos, or branded proportions.
- Preserve Apache-2.0 for code, CC BY 4.0 for Perrito Tech, and CC0 where stated.

## Working Agreement

- Check `git status --short --branch` before edits and preserve unrelated work.
- Keep all project automation, skills, docs, fixtures, and generated deliverables
  inside this repository. Do not place project files in sibling workspaces.
- Do not commit, push, publish, sign, or open a PR unless the operator explicitly
  requests that action. Never push automatically.
- Do not delete future-phase scaffolds unless removal is necessary and approved.

## Validation

Run the narrowest relevant checks, then the full release set before packaging:

```powershell
pnpm install --frozen-lockfile
pnpm format:check
pnpm typecheck
pnpm test
pnpm build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p familiar-cli -- pack validate mascots/perrito-tech
pnpm --filter @openfamiliar/desktop tauri build
```

If a tool component is unavailable, report that exact limitation; do not claim
the check passed.
