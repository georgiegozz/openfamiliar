# Privacy Policy — Windows MVP

OpenFamiliar is a local-first desktop application with no OpenFamiliar account,
cloud backend, analytics, or telemetry.

## Stored locally

- Non-sensitive preferences such as mascot scale, palette, motion, language,
  timeout, startup, and window position. `preferences.json` also stores the
  Codex executable path in the `codexPath` field, the selected provider in
  `activeProviderId`, and the Anthropic model id in `anthropicModel`.
- Category-only operational logs used for diagnostics.

The stable MVP does **not** store chat history, answers, workspace paths,
workspace files, Codex tokens, or raw Codex stdout/stderr. Questions and
answers exist in application memory only for the current ephemeral request.
OpenFamiliar does not put optional provider keys in `preferences.json` or
logs; if you add one in Settings, it is stored in the OS keychain. If you
select Anthropic in Settings and a key is stored, Quick Ask uses that key
to answer. If you select Anthropic and no key is stored, the request fails
as not authenticated and does not go through Codex CLI. Otherwise Quick
Ask still goes through the installed Codex CLI.

## One-shot answers and network

When the operator submits a question, OpenFamiliar sends it on one of two
paths. It does not add workspace context, screen contents, or conversation
history to the request.

If you select Anthropic in Settings and a key is stored in the OS keychain,
the desktop Rust process (not the webview) sends an HTTPS POST to
`https://api.anthropic.com/v1/messages`. TLS is required and redirects are
refused. The JSON body includes the model id from Settings (`anthropicModel`,
default `claude-sonnet-4-5`), a `max_tokens` limit of 1024, and a single
user message with the trimmed question text. The request headers set by
OpenFamiliar are `x-api-key` (the key is read from the keychain for that
request only), `anthropic-version` (`2023-06-01`), and `content-type`
(`application/json`). The HTTP client may also attach its default headers,
such as a User-Agent. The key is not written to `preferences.json` or logs.
Anthropic processes the question under Anthropic's own terms and privacy
policy. OpenFamiliar is not affiliated with, sponsored by, or endorsed by
Anthropic, and does not control what Anthropic retains. If you select
Anthropic but no key is stored, Quick Ask does not fall back to Codex CLI;
the request fails as not authenticated.

If the selected provider is Codex CLI, OpenCode, or Antigravity,
OpenFamiliar starts the installed Codex CLI in a fresh ephemeral process
with `--sandbox read-only`. Codex CLI owns its authentication and network
interaction under its own configuration and terms. OpenFamiliar does not
read or copy Codex authentication storage.

No request is started by idle animation, startup, palette selection, or
mascot movement. Browser sessions and cookies are never scraped.

## Sharing diagnostics

There is no automatic log upload. Any diagnostic report is shared voluntarily;
review it using `docs/guides/share-diagnostics.md` before publication.

To report a security issue, follow [SECURITY.md](./SECURITY.md). For other
questions or support, see [SUPPORT.md](./SUPPORT.md).

This document describes the implemented pre-1.0 product and is not legal advice
for every deployment scenario.
