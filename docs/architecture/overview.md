# Architecture overview

```
OpenFamiliar Desktop
        |
  Familiar Core
  - Event Bus
  - Permission Broker
  - Session Manager
  - Secure Storage
  - Audit Log
  - Context Budget
     /      |       \
Mascot SDK  AI Router  Workspace Context
  packs      adapters   files/git/IDE
             |
        Agent CLIs / MCP
```

See ADRs for decisions. Crates live under `crates/`; TS SDKs under `packages/`.
