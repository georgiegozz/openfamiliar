# CI helper scripts

These scripts are used by `.github/workflows/pr-review.yml`. They have no
third-party Python dependencies.

## Advisory baseline

`check_advisory_baseline.py` compares `cargo audit`, `cargo deny`, and
`pnpm audit` reports with `advisory-baseline.json`. Known IDs are printed on
every run. The job fails when:

- a new vulnerability ID appears
- a baseline entry's `expiresOn` date has passed (the grace period has ended)
- a baseline entry is no longer reported and should be removed

`expiresOn` dates are a maintainer decision. The grace period is tracked by
the issue URL in the `issue` field
(https://github.com/georgiegozz/openfamiliar/issues/31). Do not add `ignore` entries in `deny.toml` for the same IDs; that
would hide them from `cargo deny` itself.

## Pinned release installer

`install_pinned_release.sh` downloads a release archive, verifies a SHA-256
digest, and installs one binary. The workflow pins versions and digests in
environment variables next to each install step.

Checksum sources used by the workflow:

- gitleaks 8.30.1: GitHub release `gitleaks/gitleaks` tag `v8.30.1`, asset
  `gitleaks_8.30.1_linux_x64.tar.gz`, digest from
  `gitleaks_8.30.1_checksums.txt` on that release.
- cargo-deny 0.20.2: GitHub release `EmbarkStudios/cargo-deny` tag
  `0.20.2`, asset
  `cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz`, digest from
  `cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz.sha256` on that
  release.
- cargo-audit 0.23.0: GitHub release `rustsec/rustsec` tag
  `cargo-audit/v0.23.0`, asset
  `cargo-audit-x86_64-unknown-linux-musl-v0.23.0.tgz`. That release does not
  publish a checksums file; the SHA-256 pinned in the workflow was recorded
  from this official asset on 2026-11-12.
