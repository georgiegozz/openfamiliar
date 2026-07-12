export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface ChatRequest {
  model: string;
  messages: ChatMessage[];
  maxTokens?: number;
  sessionId: string;
}

export interface ModelInfo {
  id: string;
  name: string;
}

export interface ValidationResult {
  ok: boolean;
  message: string;
}

export type ChatEvent =
  | { type: "delta"; text: string }
  | { type: "done" }
  | { type: "error"; message: string };

export interface ModelProvider {
  id: string;
  validateConfiguration(): Promise<ValidationResult>;
  listModels(): Promise<ModelInfo[]>;
  stream(request: ChatRequest): AsyncIterable<ChatEvent>;
  cancel(sessionId: string): Promise<void>;
  estimateCost?(request: ChatRequest): Promise<{ currency: string; amount: number }>;
}

export async function collectStream(provider: ModelProvider, request: ChatRequest): Promise<string> {
  let out = "";
  for await (const ev of provider.stream(request)) {
    if (ev.type === "delta") out += ev.text;
    if (ev.type === "error") throw new Error(ev.message);
  }
  return out;
}