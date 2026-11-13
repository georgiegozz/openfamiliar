# Local and CI verification

This page records how to verify OpenFamiliar on a development machine and what
the GitHub Actions workflows already in the repository actually run. Commands
are copied from `package.json`, `scripts/test-all.ps1`, `AGENTS.md`, and
`.github/workflows/ci.yml`. If a tool is missing, report that limitation
instead of claiming a pass.

The official product target is Windows 11 x64. CI jobs `js` and `rust` also
run on `windows-latest`.

## Toolchain

| Tool                                               | Declared by                                           |
| -------------------------------------------------- | ----------------------------------------------------- |
| Node.js 20+                                        | `package.json` `engines`                              |
| pnpm 9.15.0                                        | `packageManager` and CI `pnpm/action-setup`           |
| Rust stable, `rust-version` 1.80, rustfmt, clippy  | root `Cargo.toml`, CI `dtolnay/rust-toolchain@stable` |
| Python 3.12                                        | CI `js` job and asset scripts                         |
| Visual Studio Build Tools 2022 (MSVC) and WebView2 | [dev-windows.md](../guides/dev-windows.md)            |
| Codex CLI, installed and authenticated             | only for a human Quick Ask smoke test                 |

Windows bootstrap:

```powershell
# From the repository root.
corepack enable
corepack prepare pnpm@9.15.0 --activate
pnpm install --frozen-lockfile
py -3.12 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r .\scripts\assets\requirements.txt
```

`scripts/bootstrap.ps1` is the scripted form of this setup. See
[dev-windows.md](../guides/dev-windows.md).

## Local checks on Windows

Full automated sweep used in `scripts/test-all.ps1`:

```powershell
pnpm format:check
pnpm typecheck
pnpm test
pnpm assets:check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -q -p familiar-cli -- pack validate mascots/perrito-tech
cargo run -q -p familiar-cli -- pack validate mascots/blank-template
```

That script does **not** run `pnpm build`, `pnpm licenses:check`, or
`pnpm --filter @openfamiliar/desktop tauri build`. Before a release or a
dependency change, also run:

```powershell
pnpm build
pnpm licenses:check
pnpm --filter @openfamiliar/desktop tauri build
```

`pnpm ci:js` is the JavaScript CI entry point:

```text
pnpm format:check && pnpm typecheck && pnpm test && pnpm assets:check && pnpm validate:packs && pnpm licenses:check
```

`pnpm validate:packs` validates `mascots/perrito-tech` and
`mascots/blank-template`. Automated tests must not call a real Codex service.
Use the repository fake CLI. A clean compile is not a clean-install test; see
[WINDOWS_MVP_SMOKE_TEST.md](../testing/WINDOWS_MVP_SMOKE_TEST.md).

## Local checks on Linux

Linux is not the v0.1 product target. These are the practical differences
observed in the scripts and confirmed on a Linux agent while writing this
page:

- `pnpm format:check`, `pnpm typecheck`, and most package `test` scripts are
  ordinary Node commands and can be run if Node and pnpm 9.15.0 are installed.
- Asset scripts in `package.json` call `.venv\Scripts\python.exe`. That
  Windows venv path does not exist on a typical Linux checkout. On Linux,
  invoke the same files with a local Python 3.12 environment, for example
  `python -m unittest discover -s scripts/assets/tests -p test_*.py`, instead
  of `pnpm assets:check`, unless you create an equivalent venv layout.
- `pnpm licenses:audit` launches
  `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/legal/Export-DependencyLicenses.ps1`.
  PowerShell 7 on Linux can regenerate the file, but the `rust` array order
  will not match CI. See [License inventory](#license-inventory).
- The `rust` and `js` CI jobs do not run on Linux. Building
  `openfamiliar-desktop` / the full Cargo workspace may require extra native
  libraries (for example GTK or WebKit, pulled in by Tauri dependencies).
  That Linux build was **not re-run for this documentation**. Do not assume
  `cargo test --workspace` is green on Linux without running it.
- `pnpm --filter @openfamiliar/desktop tauri build` produces Windows MSI/NSIS
  artifacts and is not a Linux release path.

## What CI runs

### `.github/workflows/ci.yml`

Triggered on push and pull request to `main`. Three jobs:

**`js`** (`windows-latest`):

1. Checkout, pnpm 9.15.0, Node 20 (pnpm cache), Python 3.12, Rust stable.
2. `pnpm install --frozen-lockfile`.
3. `python -m venv .venv` and
   `.venv\Scripts\python.exe -m pip install -r scripts/assets/requirements.txt`.
4. `pnpm ci:js` (format, typecheck, tests, asset checks, pack validation,
   license inventory check).

This is the job that enforces `docs/legal/dependency-licenses.json`. Because
it runs Windows PowerShell (the `powershell` command used by
`pnpm licenses:audit`), its sort order is the source of truth.

**`rust`** (`windows-latest`):

1. Checkout and Rust stable with `rustfmt` and `clippy`.
2. `cargo fmt --all -- --check`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo test --workspace`
5. `cargo run -p familiar-cli -- pack validate mascots/perrito-tech`

This job does not validate `mascots/blank-template` (that pack is covered in
`pnpm ci:js` via `pnpm validate:packs`).

**`secrets`** (`ubuntu-latest`):

1. Checkout with `fetch-depth: 0`.
2. `gitleaks/gitleaks-action@v2` using `GITHUB_TOKEN`.

Permissions on this job are `contents: read` and `pull-requests: read`.

### `.github/workflows/pr-review.yml`

A second pull-request workflow. It runs only on `pull_request` to `main`
(never `pull_request_target`), with workflow `permissions: contents: read`,
`persist-credentials: false` on every checkout, and concurrency per pull
request. It does not replace the `js`, `rust`, or `secrets` jobs in `ci.yml`.
All third-party actions are pinned to a full commit SHA, with the release tag
in a comment.

**`secret-scan`** (`ubuntu-24.04`):

1. Checkout with `fetch-depth: 0`.
2. Install gitleaks 8.30.1 from the official GitHub release after verifying
   the published SHA-256 digest (the MIT-licensed binary, not
   `gitleaks/gitleaks-action`). The digest comes from
   `gitleaks_8.30.1_checksums.txt` on the `v8.30.1` release, for the
   `gitleaks_8.30.1_linux_x64.tar.gz` asset.
3. `gitleaks detect` on the pull request commit range and again on the full
   git history, with `--redact`. `.gitleaks.toml` extends the default rules
   with checks for subscription GUIDs (only with subscription context) and
   Windows user-profile paths. It allowlists the fictional provider-shaped
   strings used to unit-test `redact_secrets` only in
   `crates/familiar-storage/src/lib.rs` and only for the `generic-api-key`
   rule. `.gitleaksignore` lists fingerprints for historical
   `windows-user-path` findings that are no longer in the current tree.

**`dependency-review`** (`ubuntu-24.04`):

`actions/dependency-review-action` with `fail-on-severity: high` and
`allow-licenses` aligned with `deny.toml`. This reviews dependency changes in
the pull request; it does not replace the lockfile-wide `cargo` and `pnpm`
advisory jobs.

**`rust-advisories`** (`ubuntu-24.04`):

1. Rust stable via `dtolnay/rust-toolchain`, pinned by action SHA.
2. cargo-deny 0.20.2 and cargo-audit 0.23.0, installed from GitHub releases
   with SHA-256 verification.
   - cargo-deny: release `EmbarkStudios/cargo-deny` tag `0.20.2`, asset
     `cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz`, digest from the
     matching `.sha256` asset on that release.
   - cargo-audit: release `rustsec/rustsec` tag `cargo-audit/v0.23.0`, asset
     `cargo-audit-x86_64-unknown-linux-musl-v0.23.0.tgz`. That release does
     not publish a checksums file; the digest pinned in the workflow was
     recorded from this official asset on 2026-11-12.
3. `cargo deny --locked check licenses bans sources` (hard fail).
4. Unit tests under `scripts/ci/tests`.
5. `cargo audit --json` and `cargo deny --locked -f json check advisories`,
   compared to `scripts/ci/advisory-baseline.json`. Known IDs are printed.
   The job fails on a new ID, an expired baseline entry (the grace period
   has ended), or a stale entry. The baseline is linked to the tracking
   issue in the `issue` field (https://github.com/georgiegozz/openfamiliar/issues/31).

**`javascript-audit`** (`ubuntu-24.04`):

pnpm 9.15.0 and Node 20, then `pnpm audit --json` against the lockfile (no
`pnpm install`, so no dependency lifecycle scripts run), compared to the
same baseline (high and critical only).

Local equivalents:

```bash
python3 -m unittest discover -s scripts/ci/tests -p 'test_*.py'
gitleaks detect --source . --redact --no-banner --exit-code 1
cargo deny --locked check licenses bans sources
cargo audit --json > /tmp/cargo-audit.json || true
cargo deny --locked -f json check advisories > /tmp/cargo-deny-advisories.json 2>&1 || true
python3 scripts/ci/check_advisory_baseline.py --ecosystem rust \
  --cargo-audit-json /tmp/cargo-audit.json \
  --cargo-deny-json /tmp/cargo-deny-advisories.json
pnpm audit --json > /tmp/pnpm-audit.json || true
python3 scripts/ci/check_advisory_baseline.py --ecosystem javascript \
  --pnpm-audit-json /tmp/pnpm-audit.json
```

### Other workflow, not part of the PR CI gate

### Other workflow, not part of the PR CI gate

`.github/workflows/windows-package.yml` is a separate workflow. It runs only
on `workflow_dispatch`, builds unsigned MSI/NSIS installers on
`windows-latest`, writes `SHA256SUMS.txt`, attests artifacts when the
repository is public, and uploads them. It is not started by pull requests.
This documentation does not change that file.

## License inventory

`package.json` defines:

```text
pnpm licenses:audit  ->  powershell ... scripts/legal/Export-DependencyLicenses.ps1
pnpm licenses:check  ->  pnpm licenses:audit && git diff --exit-code -- docs/legal/dependency-licenses.json
```

`licenses:check` therefore **regenerates** `docs/legal/dependency-licenses.json`
and then requires a clean `git diff` on that file.

### What the script does

Confirmed in `scripts/legal/Export-DependencyLicenses.ps1`:

1. `cargo metadata --format-version 1 --locked` (fails if `Cargo.lock` is out
   of date).
2. Each Cargo package becomes `{ name, version, license }`. An empty license
   is written as `NOASSERTION`. The array is ordered with
   `Sort-Object name, version`.
3. `pnpm licenses list --prod --json`. Production JavaScript packages are
   grouped by license, then sorted by name and version.
4. The document is written as UTF-8 without BOM, with a trailing newline, and
   then passed through `pnpm exec prettier --write`.

### Why Linux and Windows can disagree

`Sort-Object` uses the current culture's string comparison.

- On GitHub Actions `windows-latest`, `pnpm licenses:audit` calls
  `powershell` (Windows PowerShell 5.1). That runtime uses Windows **word
  sort**, which ignores hyphens when comparing names. The committed inventory
  follows that order. Examples already in the file: `autocfg` then
  `auto-launch`; `bitflags` then `bit-set`.
- On Linux, PowerShell 7 (`pwsh`) uses ICU / ordinal-style comparison, which
  treats `-` as a real character. The same package set is therefore written
  in a different order: `auto-launch` then `autocfg`; `bit-set` then
  `bitflags`.

This was confirmed by sorting the committed `rust` names with PowerShell
7.4.6 on Linux (invariant culture): **86** positional mismatches, no added
or removed packages. Prettier does not reorder arrays, so the mismatch
survives formatting and fails `git diff --exit-code`.

The JavaScript side of the inventory was not shown to have this hyphen
issue in that check; the observed drift was in the `rust` array. Whether
`ConvertTo-Json` in Windows PowerShell 5.1 versus PowerShell 7 would also
diverge _before_ prettier was not separately verified; prettier is always
run afterwards.

### How to regenerate it correctly

On Windows, from a checkout whose lockfiles match the change:

```powershell
pnpm licenses:audit
git diff -- docs/legal/dependency-licenses.json
pnpm licenses:check
```

Commit the file only when the diff is about real package or license changes,
not a wholesale reorder of existing crates.

If the script is run on Linux and the only diff is a shuffle of the `rust`
array, do not commit it. Restore the file and regenerate on Windows, or take
the version produced by the `js` CI job. Windows is the source of truth
because that is where CI evaluates `licenses:check`.

A reusable procedure is in
[the license-inventory skill](../../.agents/skills/openfamiliar-fix-license-inventory/SKILL.md).
