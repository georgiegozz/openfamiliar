# Dependency advisories

This page records the 19 November 2026 (America/Mexico_City) follow-up to
[issue #31](https://github.com/p3rformative/openfamiliar/issues/31). It is not
a substitute for `cargo audit`, `pnpm audit`, or a future `pr-review`
workflow. The tracking grace date on #31 remains **2027-02-10** for anything
still open.

## Resolved in this follow-up

### Tauri (not listed in #31, patched here)

| ID                                                                                                 | Before         | After          |
| -------------------------------------------------------------------------------------------------- | -------------- | -------------- |
| [GHSA-w28w-mhc8-qvjv](https://github.com/tauri-apps/tauri/security/advisories/GHSA-w28w-mhc8-qvjv) | `tauri` 2.11.5 | `tauri` 2.11.6 |

2.11.6 is the minimum patched release. 2.12.7 was not taken: 2.12.0 changes
Android `$VIDEO` / `video_dir()`, and `tauri-build` 2.7 declares rust-version
1.90. The JavaScript packages stay on the 2.11 line (`@tauri-apps/api` 2.11.1,
`@tauri-apps/cli` 2.11.5). `tauri-build` remains 2.6.3.

### Rust advisories from #31

| ID                                                                           | Crate           | Before                   | After                                                     |
| ---------------------------------------------------------------------------- | --------------- | ------------------------ | --------------------------------------------------------- |
| RUSTSEC-2026-0369, 0367, 0429, 0368                                          | `zip`           | 2.4.2 (default features) | 9.7.3, `default-features = false`, feature `deflate` only |
| RUSTSEC-2026-0391                                                            | `zstd`          | 0.13.3                   | removed from the graph (no remaining dependents)          |
| RUSTSEC-2026-0398, 0403                                                      | `zstd-safe`     | 7.2.4                    | removed from the graph                                    |
| RUSTSEC-2026-0410                                                            | `zstd-sys`      | 2.0.16+zstd.1.5.7        | removed from the graph                                    |
| RUSTSEC-2026-0340, 0341, 0338, 0339, 0342                                    | `quinn-proto`   | 0.11.16                  | >= 0.11.18                                                |
| RUSTSEC-2026-0285, 0346, 0348, 0349                                          | `rustls`        | 0.23.41                  | 0.23.46                                                   |
| RUSTSEC-2026-0381                                                            | `webpki-roots`  | 1.0.8                    | >= 1.0.10                                                 |
| RUSTSEC-2026-0382                                                            | `rustls-webpki` | 0.103.13                 | >= 0.103.16                                               |
| RUSTSEC-2026-0386 (`unsound`)                                                | `tokio`         | 1.52.3                   | 1.54.1 (>= 1.53.3)                                        |
| RUSTSEC-2026-0538 (not in #31; found by `cargo audit` 0.23.0 on 19 Nov 2026) | `quinn-udp`     | 0.5.15                   | 0.5.16                                                    |
| RUSTSEC-2026-0486 (`unsound`; not in the #31 vulnerability list)             | `rustix`        | 1.1.4                    | 1.1.6                                                     |

The same `zip` 9.7.3 and `quinn-proto` 0.11.19 bumps also clear four
advisories that were not in #31 but showed up in `cargo audit` 0.23.0 on
19 Nov 2026 against current `main`: RUSTSEC-2026-0545 and 0548 (`zip`
2.4.2) and RUSTSEC-2026-0523 and 0524 (`quinn-proto` 0.11.16).

Workspace `rust-version` moves from 1.80 to 1.88 because `zip` 9.0.1+
declares 1.88. 9.7.3 is the newest 9.x that stays on 1.88; 9.8.0 declares
1.89 and was not taken. Default features of `zip` were turned off because
they enable `aes-crypto`, and `aes` 0.9.4 declares rust-version 1.89.
`familiar-cli` only writes and lists Deflated entries, so the `deflate`
feature is enough. CI already installs `stable`, which is newer than 1.88.

`reqwest` stays on the 0.12 line (0.12.28). 0.13 would be a separate
breaking upgrade.

### JavaScript advisories from #31

| ID                                       | Package                      | Before       | After                                                      |
| ---------------------------------------- | ---------------------------- | ------------ | ---------------------------------------------------------- |
| GHSA-28wg-ghj8-5hjv, GHSA-2v37-7h3g-55p8 | `nanoid`                     | 3.3.15       | 3.3.20 (pnpm override; do not jump to 5.x/6.x)             |
| GHSA-r28c-9q8g-f849                      | `postcss`                    | 8.5.17       | 8.5.33                                                     |
| GHSA-c83g-rgw3-j3cx, GHSA-73wf-gq98-2v4g | `browserslist`               | 4.28.6       | 4.29.6                                                     |
| GHSA-68fv-2mgg-jv7q                      | `source-map-js`              | 1.2.1        | 1.2.4                                                      |
| GHSA-587h-7wmj-h5pm                      | `vite` (via `launch-editor`) | `vite` 6.4.3 | `vite` 6.4.6 (patched >= 6.4.4); `launch-editor` >= 2.14.2 |

These JavaScript packages are dev/build-time dependencies, not the
Windows runtime of the shipped desktop app.

## Still open

| ID                                      | Crate                           | Version         | Why it was not upgraded                                                                                                                                                             |
| --------------------------------------- | ------------------------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| RUSTSEC-2026-0459 (GHSA-72w2-rjgg-q729) | `libsqlite3-sys` via `rusqlite` | 0.30.1 / 0.32.1 | The first `rusqlite` that takes `libsqlite3-sys` >= 0.39 is 0.41.0, which declares rust-version 1.89 and is several breaking releases above 0.32.1. Left for a dedicated follow-up. |

`cargo audit` 0.23.0 also still reports warnings that were not part of the #31
vulnerability list and have no safe patch in this follow-up: `chrono` and
several `unic-*` / `proc-macro-error` crates are unmaintained, `chacha20`
0.10.1 is yanked, and `glib` 0.18.5 is unsound (RUSTSEC-2024-0429; fixing it
needs glib 0.20+ and is tied to the Linux desktop stack, which is not the
Windows 11 product target).

No cargo-deny, cargo-audit, or GitHub Actions workflow was changed in this
follow-up. After this work lands on `main`, PR #32's advisory baseline will
need an update so it no longer lists IDs that have been cleared (the baseline
script fails when a listed ID disappears).
