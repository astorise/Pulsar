export type ClientMessage =
  | {
      type: 'init';
      workspace_url: string;
      workspace_token: string;
    }
  | {
      type: 'user_message';
      content: string;
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
  if (value.type === 'error' && typeof value.message === 'string') {
    return value as ServerMessage;
  }
  throw new Error('Unsupported Pulsar server message');
}
