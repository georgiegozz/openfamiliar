# SignPath Foundation preparation

This page is a maintainer checklist for a possible SignPath Foundation
application. It does not mean OpenFamiliar is accepted, that a certificate
exists, or that any current binary is signed. Public v0.1.0 installers remain
unsigned. Do not promise signed releases.

The companion workflow `.github/workflows/signpath-windows.yml` is
**manual** (`workflow_dispatch` only) and **inert** until a maintainer
enables it. Adding that workflow is a GitHub Actions workflow change and
needs Jorge's explicit authorization before it is merged.

Read this together with `docs/guides/windows-code-signing.md`. SignPath is
only one of the public-signing options listed there. This work is low
priority: a rejection or a later decision to use another path is acceptable.

Conditions below were verified against the SignPath Foundation terms at
[signpath.org/terms](https://signpath.org/terms.html) on 19 November 2026
(America/Mexico_City). Those terms were still marked as a **draft** Code of
Conduct. Recheck them before applying.

## Current project facts

- The repository `p3rformative/openfamiliar` is public, Apache-2.0 for code,
  and had **1** GitHub star when this page was written (19 November 2026).
- Official Perrito Tech artwork is licensed under **CC BY 4.0**. Blank mascot
  templates use **CC0-1.0**. See `NOTICE` and `mascots/perrito-tech/LICENSE`.
- Releases so far are an unsigned pre-release. The repo must not claim
  otherwise.

## Risks that may block acceptance

These are taken from the Foundation terms, not from a private conversation
with SignPath. SignPath Foundation is under no obligation to accept a
project, and there is no independent appeal.

**Verifiable reputation.** For executable programs that people may download
and run based on the signature, SignPath requires "a certain verifiable
reputation" because the certificate is issued to **SignPath Foundation**,
whose name and reputation are on the line. "We cannot sign binaries based
on source code that nobody knows." With about one star and limited public
use, **rejection is likely**. The terms say some rules are "soft and fuzzy"
and that the decision is theirs alone.

**OSI license for every component.** A free OSS subscription requires an
OSI-approved Open Source license, without commercial dual-licensing, for
**all components**, and no proprietary or non-open-source component (system
libraries in signed packages are allowed). Apache-2.0 is OSI-approved. CC
BY 4.0 and CC0-1.0 are Creative Commons content licenses, not OSI-approved
software licenses. The official mascot art **may clash** with that "all
components" rule. Do not assume SignPath will treat packaged artwork as
out of scope. Confirm this before, or during, an application.

**Do not promise signing.** Until the Foundation accepts the project, a
certificate is in use, and a specific build has actually been signed, no
README, release note, or website text should say that OpenFamiliar is
SignPath-signed or that signing is forthcoming as a guarantee.

**Publisher is SignPath Foundation.** If a certificate is granted, it is
issued to SignPath Foundation, which is then the publisher of the OSS
project. They can pause or terminate the subscription and revoke the
certificate, including retroactively, if their rules are broken. They
accept no liability for damages from software signed with those
certificates.

## If the project is accepted: homepage "Code signing policy"

The terms require a section titled **Code signing policy** on the project's
home page and on download or release pages (as a heading or a link). **This
repository does not add that section to the README now.** Jorge should add
it only after acceptance, with the required content, including:

- The sentence "Free code signing provided by SignPath.io, certificate by
  SignPath Foundation".
- Team roles and the people or GitHub teams who hold them:
  - **Authors / committers:** trusted to change source without extra review.
  - **Reviewers:** review changes from people who are not committers.
  - **Approvers:** approve each signing request for a release.
- A privacy policy link, or the exact allowed wording: "This program will
  not transfer any information to other networked systems unless
  specifically requested by the user or the person installing or operating
  it". Also account for third-party components. See `PRIVACY.md`.

Other conditions that would apply after acceptance include: MFA on GitHub
and SignPath for every team member; sign only this team's own projects and
binaries; every release needs manual signing approval; artifact metadata
(product name and version) must be set and enforced; no malware or
"hacking tools" as defined in the terms; documented, maintained, and
already released software; all jobs leading up to a signing request must
run on GitHub-hosted runners.

## Maintainer checklist (Jorge)

Nobody should register the project, create a SignPath account, or paste
tokens in chat on Jorge's behalf. If he decides to apply, he should:

1. Reread [signpath.org/terms](https://signpath.org/terms.html) and
   [SignPath's GitHub integration docs](https://docs.signpath.io/trusted-build-systems/github).
2. Weigh the reputation and license risks above. An application can wait.
3. If applying, do it himself through SignPath Foundation. Install the
   SignPath GitHub App only if they require it.
4. After a written acceptance, create a SignPath project linked to
   `p3rformative/openfamiliar`, a signing policy, and an artifact
   configuration that matches the unsigned MSI/NSIS outputs.
5. Create an API token for a user with submitter permission. Store it only
   as the GitHub Actions repository secret `SIGNPATH_API_TOKEN`. Never
   commit it, put it in a workflow file, or paste it into a pull request
   or chat.
6. Set repository **variables** (not secrets, except the token):
   - `SIGNPATH_ORGANIZATION_ID`
   - `SIGNPATH_PROJECT_SLUG`
   - `SIGNPATH_SIGNING_POLICY_SLUG`
7. Only then set `SIGNPATH_ENABLED` to `true`. Until the variables are
   present, `signpath-windows` prints a notice and exits without building
   or calling SignPath. If the variables are set but `SIGNPATH_API_TOKEN`
   is missing, a short preflight job fails before any Windows build.
8. Add the **Code signing policy** section to the home page as required
   by the terms. Do not add it early, and do not claim a signature that
   does not exist.
9. Require MFA for everyone who can push or approve signing.
10. Run `.github/workflows/signpath-windows.yml` by hand and confirm the
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
