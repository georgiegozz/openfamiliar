# Signed in-app updates

Current OpenFamiliar installers are **not signed**, and this repository does
**not** publish a signed update channel yet. The updater in Settings is inert
until a maintainer generates a key pair, commits only the **public** key,
configures the release workflow secrets, and later creates a version tag.
This document does not create keys, tags, or GitHub Releases.

The companion workflow `.github/workflows/windows-release.yml` may run on a
`v*.*.*` tag or on `workflow_dispatch`, but it stays **inert** until the
repository variable `UPDATER_RELEASE_ENABLED` is set to `true` and the signing
secrets below are present. Adding that workflow is a GitHub Actions workflow
change and needs explicit maintainer authorization before it is merged.

Read this together with `docs/guides/windows-code-signing.md`. In-app update
signatures (minisign) are not a substitute for Windows Authenticode / SignPath
on the installers themselves.

## MSRV and plugin version

The workspace `rust-version` is 1.88 and the desktop crate is pinned to Tauri
2.11.6.

- Current `tauri-plugin-updater` 2.13.0 and later declare **Rust 1.90** and
  depend on `tauri ^2.12`. That would also pull `tauri-build` versions that
  declare rust-version 1.90. Tauri 2.12.0 changes Android `$VIDEO` /
  `video_dir()`. Those bumps were deliberately not taken in the 2.11.6
  advisory patch.
- This repository therefore pins **`tauri-plugin-updater` 2.12.0** and
  `@tauri-apps/plugin-updater` 2.12.0: the newest 2.x plugin that still
  declares rust-version 1.77.2 and depends on `tauri ^2.10`.
- The plugin is added with `rustls-tls` and `system-proxy` only. The default
  `zip` feature is off so Windows MSI/NSIS updates do not pull `zip` 4 (the
  workspace already uses `zip` 9.7.3 for `familiar-cli`).

Raising the workspace MSRV to 1.90 in order to take updater 2.13+ is a later
maintainer decision, not part of this change.

## Generate the signing keys (do not do this in CI or in the repo)

Tauri's updater will not install an artifact that lacks a valid minisign
signature. A maintainer must generate the key pair on a trusted machine, not
in this repository, not in a pull request, and not in a workflow log.

From the repository root, with the desktop package dependencies installed:

```powershell
pnpm --filter @openfamiliar/desktop tauri signer generate -w $HOME/.tauri/openfamiliar.key
```

On Unix:

```bash
pnpm --filter @openfamiliar/desktop tauri signer generate -w "$HOME/.tauri/openfamiliar.key"
```

The command prints a **public** key and writes the private key to the path
given by `-w`. Optional password protection is offered by the CLI.

Then:

1. Store the **private** key only as the GitHub Actions repository secret
   `TAURI_SIGNING_PRIVATE_KEY` (the key contents or a path available to the
   runner; prefer the contents). If the key has a password, store it only as
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Never commit the private key, the
   password, or a `.key` file, and never paste them into a pull request or
   chat.
2. Put the **public** key in `apps/desktop/src-tauri/tauri.conf.json` at
   `plugins.updater.pubkey`. The public key is not a secret.
3. Set `plugins.updater.endpoints` to:

   `https://github.com/p3rformative/openfamiliar/releases/latest/download/latest.json`

4. Leave `bundle.createUpdaterArtifacts` **unset or false** in the committed
   config so local and unsigned CI builds still work without the private key.
   The release workflow turns that flag on only for a signing build.
5. Only then set the repository variable `UPDATER_RELEASE_ENABLED` to `true`.
6. A maintainer creates a version tag (`vX.Y.Z`) themselves when they are
   ready. This tree must not create tags or GitHub Releases on its own.

Losing the private key means already-installed copies that embedded the
matching public key cannot be updated with a newly generated key. Users would
need to install a new build another way.

## What the Settings window can do

`updater:*` permissions are granted only to the `settings` window
(`apps/desktop/src-tauri/capabilities/settings.json`). The mascot window
cannot check, download, or install updates. Settings does not look for
updates at startup; the operator has to click **Buscar actualizaciones**.
Until the public key and endpoint are configured, that action reports that
the signed channel is not enabled and does not publish anything.

## What the release workflow does

- Triggers: `workflow_dispatch`, or a push of a `v*.*.*` tag.
- Default path: a notice job runs and does not build, sign, tag, or create a
  GitHub Release.
- Enabled path: a preflight job confirms that `TAURI_SIGNING_PRIVATE_KEY` is
  present (it records only whether the secret exists, never the value). A
  Windows job then builds MSI/NSIS installers with updater artifacts and
  signatures. A GitHub Release is created only for a `v*.*.*` tag. A manual
  `workflow_dispatch` run may upload workflow artifacts but does not create a
  release or a tag.
- Permissions: `contents: read` at the workflow level. The publishing job
  also uses `contents: write` so it can create the release. No `id-token` or
  `packages` permissions.
- Third-party actions are pinned to full commit SHAs.

The existing `windows-package.yml` workflow still produces unsigned
installers and is unchanged.
