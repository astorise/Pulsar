export type ClientMessage =
  | {
      type: 'init';
      workspace_url: string;
      workspace_token: string;
    }
  | {
      type: 'user_message';
      content: string;
    }
  | {
      type: 'lsp_hover_response';
      id: string;
      markdown: string;
    };

export type ServerMessage =
  | {
      type: 'stream_token';
      content: string;
    }
  | {
      type: 'action_event';
      action: string;
      target: string;
    }
  | {
      type: 'lsp_hover_request';
      id: string;
      path: string;
      line: number;
      character: number;
    }
  | {
      type: 'suspend';
      instruction: string;
      requires_feedback: boolean;
    }
  | {
      type: 'handshake';
      plan: unknown;
    }
  | {
      type: 'escalated';
      report: string;
    }
  | {
      type: 'kiln';
      message: string;
      dataset_size: number;
      training_submitted: boolean;
    }
  | {
      type: 'error';
      message: string;
    };

export function parseServerMessage(payload: string): ServerMessage {
  const value = JSON.parse(payload) as Partial<ServerMessage>;
  if (value.type === 'stream_token' && typeof value.content === 'string') {
    return value as ServerMessage;
  }
  if (
    value.type === 'action_event' &&
    typeof value.action === 'string' &&
    typeof value.target === 'string'
  ) {
    return value as ServerMessage;
  }
  if (
    value.type === 'lsp_hover_request' &&
    typeof value.id === 'string' &&
    typeof value.path === 'string' &&
    typeof value.line === 'number' &&
    typeof value.character === 'number'
  ) {
    return value as ServerMessage;
  }
  if (
    value.type === 'suspend' &&
    typeof value.instruction === 'string' &&
    typeof value.requires_feedback === 'boolean'
  ) {
    return value as ServerMessage;
  }
  if (value.type === 'handshake' && 'plan' in value) {
    return value as ServerMessage;
  }
  if (value.type === 'escalated' && typeof value.report === 'string') {
    return value as ServerMessage;
  }
  if (
    value.type === 'kiln' &&
    typeof value.message === 'string' &&
    typeof value.dataset_size === 'number' &&
    typeof value.training_submitted === 'boolean'
  ) {
    return value as ServerMessage;
  }
  if (value.type === 'error' && typeof value.message === 'string') {
    return value as ServerMessage;
  }
  throw new Error('Unsupported Pulsar server message');
}
