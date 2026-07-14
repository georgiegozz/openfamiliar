# Roadmap

Status vocabulary is evidence-based:

- **implemented**: production path exists;
- **tested locally**: relevant automated/local check passed;
- **scaffold only**: structure or mock exists, not a product capability;
- **experimental**: retained outside stable v0.1;
- **deferred**: intentionally outside the current release.

## Stable Windows v0.1

| Capability                                     | Status                          | Release evidence                                    |
| ---------------------------------------------- | ------------------------------- | --------------------------------------------------- |
| Two-window Tauri desktop shell                 | implemented                     | Production build + human smoke pending/final result |
| Perrito Tech spritesheet and animation runtime | implemented, tested locally     | Runtime unit tests + visual smoke                   |
| One-shot Quick Ask state machine               | implemented, tested locally     | Reducer and desktop tests                           |
| Restricted Codex CLI adapter                   | implemented, tested locally     | Fake CLI integration tests only                     |
| Timeout/cancel/process-tree termination        | implemented, tested locally     | Fake slow/child scenarios                           |
| Tray, click-through recovery, preferences      | implemented                     | Human Windows interaction smoke required            |
| Monitor/DPI-safe restore                       | implemented, tested locally     | Rust unit tests + multi-monitor human smoke         |
| MSI and NSIS packaging                         | implemented                     | Local installer build result recorded per release   |
| Clean Windows install/reinstall                | deferred to operator validation | Human checklist                                     |
| Code signing                                   | deferred                        | Requires publisher identity and certificate         |

## Retained Experiments

| Area                                     | Status                | Stable v0.1 rule                                 |
| ---------------------------------------- | --------------------- | ------------------------------------------------ |
| Ollama/OpenAI-compatible/Gemini adapters | experimental scaffold | Not initialized or visible                       |
| Workspace context                        | experimental crate    | No stable IPC or UI                              |
| Permission broker and agent mode         | experimental scaffold | No execution/write path                          |
| Agent CLI bridge                         | experimental scaffold | Stable desktop uses dedicated Codex adapter only |
| Local MCP                                | scaffold only         | No listener or startup                           |
| VS Code extension                        | scaffold only         | Not part of desktop release                      |
| Creator Studio                           | scaffold only         | Not part of desktop release                      |
| Advanced `.familiar` distribution        | experimental          | Perrito Tech bundled only                        |

## Future Decision Gates

1. Complete clean-user Windows smoke, performance soak, accessibility review,
   uninstall/reinstall, and unsigned installer distribution evidence.
2. Decide code-signing and update-channel ownership before public binaries.
3. Any additional provider, workspace, agent, MCP, or editor capability requires
   its own security/UX decision and cannot silently enter stable v0.1.
4. macOS, Linux, voice, screen capture, marketplace, accounts, remote backend,
   cross-device sync, repository indexing, and background agents remain deferred.
