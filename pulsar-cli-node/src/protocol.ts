export interface InitMessage {
  type: "init";
  workspace_url: string;
  workspace_token: string;
}

export interface UserMessage {
  type: "user_message";
  content: string;
}

export interface LspHoverResponse {
  type: "lsp_hover_response";
  id: string;
  markdown: string;
}

export interface ResumeRequest {
  type: "resume_request";
  session_id: string;
  feedback: string;
}

export type ClientMessage =
  | InitMessage
  | UserMessage
  | LspHoverResponse
  | ResumeRequest;

export interface StreamToken {
  type: "stream_token";
  content: string;
}

export interface ActionEvent {
  type: "action_event";
  action: string;
  target: string;
}

export interface LspHoverRequest {
  type: "lsp_hover_request";
  id: string;
  path: string;
  line: number;
  character: number;
}

export interface Suspend {
  type: "suspend";
  instruction: string;
  requires_feedback: boolean;
}

export interface Handshake {
  type: "handshake";
  plan: unknown;
}

export interface Escalated {
  type: "escalated";
  report: string;
}

export interface Kiln {
  type: "kiln";
  message: string;
  dataset_size: number;
  training_submitted: boolean;
}

export interface ErrorMessage {
  type: "error";
  message: string;
}

export type ServerMessage =
  | StreamToken
  | ActionEvent
  | LspHoverRequest
  | Suspend
  | Handshake
  | Escalated
  | Kiln
  | ErrorMessage;

export function encodeClientMessage(message: ClientMessage): string {
  return JSON.stringify(message);
}

export function decodeServerMessage(payload: string): ServerMessage {
  const parsed = JSON.parse(payload) as ServerMessage;
  if (!parsed || typeof parsed !== "object" || typeof parsed.type !== "string") {
    throw new Error(`invalid server message payload: ${payload}`);
  }
  return parsed;
}

export function isFinishAction(payload: string): boolean {
  try {
    const message = decodeServerMessage(payload);
    return message.type === "action_event" && message.action === "finish";
  } catch {
    return false;
  }
}
