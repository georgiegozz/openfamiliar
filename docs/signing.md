# SignPath Foundation preparation

Current OpenFamiliar installers are **not signed**. Signing with SignPath
Foundation is under evaluation. There is no guarantee of acceptance, a
certificate, or a signed release. Do not describe binaries as signed until
a specific build has actually been signed.

The companion workflow `.github/workflows/signpath-windows.yml` is
**manual** (`workflow_dispatch` only) and **inert** until a maintainer
enables it. Adding that workflow is a GitHub Actions workflow change and
needs explicit maintainer authorization before it is merged.

Read this together with `docs/guides/windows-code-signing.md`. SignPath is
one of several public-signing options listed there. Conditions below were
checked against the SignPath Foundation terms at
[signpath.org/terms](https://signpath.org/terms.html) on 19 November 2026
(America/Mexico_City). Those terms were still marked as a **draft** Code of
Conduct. Recheck them before any evaluation proceeds.

## What to keep in mind

- SignPath Foundation decides whether to accept a project. They are under
  no obligation to do so, and there is no independent appeal.
- For downloadable executables they require a verifiable project
  reputation, because the certificate would be issued in their name.
- A free OSS subscription requires an OSI-approved license for all
  components. The code is Apache-2.0 (OSI-approved). Official Perrito Tech
  art is CC BY 4.0, and blank templates are CC0-1.0; those are not
  OSI-approved software licenses and should be reviewed against that rule.
- If a project is accepted, the terms require a **Code signing policy**
  section on the home page and download pages, with required wording, team
  roles (authors, reviewers, approvers), and a privacy statement. This
  repository does not add that section to the README now.

## Maintainer checklist

1. Reread [signpath.org/terms](https://signpath.org/terms.html) and
   [SignPath's GitHub integration docs](https://docs.signpath.io/trusted-build-systems/github).
2. If maintainers choose to evaluate SignPath, complete any application
   and GitHub App setup themselves.
3. After written acceptance, create a SignPath project linked to
   `p3rformative/openfamiliar`, a signing policy, and an artifact
   configuration that matches the unsigned MSI/NSIS outputs.
4. Create an API token for a user with submitter permission. Store it only
   as the GitHub Actions repository secret `SIGNPATH_API_TOKEN`. Never
   commit it or put it in a workflow file.
5. Set repository **variables** (not secrets, except the token):
   - `SIGNPATH_ORGANIZATION_ID`
   - `SIGNPATH_PROJECT_SLUG`
   - `SIGNPATH_SIGNING_POLICY_SLUG`
6. Only then set `SIGNPATH_ENABLED` to `true`. Until the variables are
   present, `signpath-windows` prints a notice and exits without building
   or calling SignPath. If the variables are set but `SIGNPATH_API_TOKEN`
   is missing, a short preflight job fails before any Windows build.
7. Add the **Code signing policy** section to the home page only after
   acceptance, as required by the terms. Do not claim a signature that
   does not exist.
8. Require MFA for everyone who can push or approve signing.
9. Run `.github/workflows/signpath-windows.yml` by hand and confirm the
   signed artifacts before any public "signed" label.

## What the workflow does

- Trigger: `workflow_dispatch` only. It is not part of `ci.yml`.
- Default path (today): the `skipped-notice` job runs and does nothing
  else. This is the intended state.
- Enabled path: a short `preflight` job checks that `SIGNPATH_API_TOKEN`
  exists (it records only whether the secret is present, never the value).
  Then, on a GitHub-hosted `windows-latest` runner, the workflow builds
  unsigned MSI/NSIS installers, uploads them as a workflow artifact,
  submits a SignPath signing request, and uploads the signed output as a
  separate artifact. It does not create a GitHub Release or change repo
  settings.
- Permissions: `contents: read` at the workflow level; the signing job
  also uses `actions: read` (required by the SignPath action to read the
  uploaded artifact). No `id-token`, `contents: write`, or `packages`
  permissions.
- Third-party actions are pinned to full commit SHAs.

The existing `windows-package.yml` workflow still produces unsigned
installers and is unchanged.
