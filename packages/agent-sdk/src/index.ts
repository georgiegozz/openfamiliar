export type AgentEventType =
  | "session.started"
  | "agent.thinking"
  | "agent.reading"
  | "agent.editing"
  | "agent.running"
  | "agent.waiting_approval"
  | "agent.completed"
  | "agent.failed"
  | "session.cancelled";

export interface AgentEvent {
  type: AgentEventType;
  sessionId: string;
  payload?: Record<string, unknown>;
}

export interface AgentAdapterInfo {
  id: string;
  available: boolean;
}