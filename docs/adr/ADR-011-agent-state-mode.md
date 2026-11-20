# ADR-011: Agent-state mode

- Status: Proposed
- Date: 2026-11-19

What this decides: nothing yet. This ADR records the conflict between a
stateful mode and v0.1's one-shot, no-listener stance; v0.1 does not change.

## Context

The stable Windows v0.1 desktop is a one-shot familiar: one independent
question at a time, no conversation history, and no agent mode in the
stable UI ([ADR-009](./ADR-009-windows-one-shot-mvp.md),
[ADR-010](./ADR-010-additional-one-shot-providers.md)). Idle operation
starts no loopback listener, no MCP server, and no remote UI. The
`familiar-mcp` crate and the experimental permission-broker / agent-mode
scaffolds exist in the monorepo, but they are not initialized or started
by the stable desktop. The roadmap records Local MCP as scaffold only
("no listener or startup") and lists workspace, agent mode, MCP, and ACP
as items that still need their own security and UX decision.

A later product idea — **agent-state mode** — would let the familiar show
or hold ongoing work: a running task, retained context, or a session that
lasts beyond a single question. That idea collides with the current
stable posture. A mode that keeps agent state often wants a long-lived
process, a way for other tools to attach, or a local listener. Those are
exactly the surfaces v0.1 refuses by default.

[ADR-007](./ADR-007-local-ipc-and-mcp-transport.md) earlier accepted a
local MCP transport for external tools (no public network bind by
default). That acceptance has not been carried into the stable desktop.
Reopening it, or adding a different attach point, would be a new
boundary change, not a silent extension of one-shot Quick Ask.

This ADR records the conflict and the alternatives. It does not accept
the mode, and it does not ship any implementation.

## Decision

This ADR is proposed, not accepted. The decision is **pending**. Until
it is accepted (or explicitly rejected) in a later review:

1. The stable desktop stays on the one-shot path in ADR-009 and ADR-010.
   It must not gain a retained agent session, a background agent, a
   loopback listener, or a started MCP server as a side effect of this
   document.
2. Experimental crates (`familiar-mcp`, the permission broker, the agent
   bridge beyond dedicated one-shot CLI adapters) remain uninitialized
   in the stable UI.
3. Any future design for agent-state mode needs its own security and UX
   review: what state is kept, where it is stored, which window can see
   it, whether any local transport is started, and how the operator
   turns it off. That review is not done here.

## Consequences

- Shipping v0.1 does not wait on this decision.
- README and the security playbook keep the current rule: idle operation
  starts no listener, MCP, or remote UI. This ADR does not update
  `docs/security/threat-model.md`, which still describes an earlier MVP
  surface (chat history, workspace, and local MCP as a control).
- Accepting this mode later would have to amend or supersede the stable
  reading of ADR-007, ADR-009, and the roadmap gates for agent mode and
  MCP. It would not be a documentation-only change.
- Rejecting it would make the current one-shot, no-listener posture
  explicit for this idea as well.

## Alternatives considered

- **Keep the one-shot, no-listener stable desktop (status quo).** No
  retained agent state in the product UI. Experimental scaffolds stay
  out of the stable process. This is the default while the decision is
  pending, and it may become the accepted decision.
- **Opt-in UI that only displays local status of a user-launched
  one-shot process** (for example, "this question is still running"),
  without a listener, without MCP, and without keeping conversation
  history after the process exits. Narrower than a full agent-state
  mode, but it still needs a separate UX decision and must not grow
  into a session or an attach point by accident.
- **Revisit ADR-007 and start a local MCP (or other local transport)
  from the desktop.** This is the most direct clash with the later
  no-listener / no-MCP stable rule. It would expose a new local attack
  surface (a malicious local client talking to the familiar) and would
  require capabilities, auth of local callers, and an explicit startup
  choice. Not authorized by this ADR.
- **Background or persistent agents**, already listed as deferred in the
  roadmap. Out of scope for v0.1, and not enabled by proposing this
  record.

This document is an original project decision record. It does not
adapt, wrap, or reimplement any third-party agent product or protocol
source.
