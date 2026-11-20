# Privacy Policy — Windows MVP

OpenFamiliar is a local-first desktop application with no OpenFamiliar account,
cloud backend, analytics, or telemetry.

## Stored locally

- Non-sensitive preferences such as mascot scale, palette, motion, language,
  timeout, startup, and window position. `preferences.json` also stores the
  Codex executable path in the `codexPath` field.
- Category-only operational logs used for diagnostics.

The stable MVP does **not** store chat history, answers, workspace paths,
workspace files, Codex tokens, or raw Codex stdout/stderr. Questions and
answers exist in application memory only for the current ephemeral request.
OpenFamiliar does not put optional provider keys in `preferences.json` or
logs; if you add one in Settings, it is stored in the OS keychain. If you
select Anthropic in Settings and a key is stored, Quick Ask uses that key
to answer. If you select Anthropic and no key is stored, Quick Ask shows an
authentication error and does not fall back to Codex CLI.

## Codex CLI and network

When the operator submits a question through Codex CLI, OpenFamiliar starts
the installed Codex CLI in a fresh read-only process. Codex CLI owns its
authentication and network interaction under its own configuration and terms.
OpenFamiliar does not read or copy Codex authentication storage and does not
add workspace context.

If you select Anthropic and a key is stored, OpenFamiliar sends your question
directly to api.anthropic.com over HTTPS using your key; Anthropic handles it
under its own terms and OpenFamiliar does not control what it retains. No
request is made to Anthropic unless you select it and a key is stored.

No request is started by idle animation, startup, palette selection, or mascot
movement. Browser sessions and cookies are never scraped.

## Sharing diagnostics

There is no automatic log upload. Any diagnostic report is shared voluntarily;
review it using `docs/guides/share-diagnostics.md` before publication.

To report a security issue, follow [SECURITY.md](./SECURITY.md). For other
questions or support, see [SUPPORT.md](./SUPPORT.md).

This document describes the implemented pre-1.0 product and is not legal advice
for every deployment scenario.
