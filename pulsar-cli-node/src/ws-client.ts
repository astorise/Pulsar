import WebSocket, { RawData } from "ws";
import {
  ClientMessage,
  ServerMessage,
  decodeServerMessage,
  encodeClientMessage,
  isFinishAction,
} from "./protocol.js";

export interface WsClientHandle {
  send: (message: ClientMessage) => void;
  close: () => Promise<void>;
  done: Promise<void>;
}

export interface WsClientOptions {
  endpoint: string;
  init: ClientMessage;
  onFinish: () => void;
  onText?: (rendered: string) => void;
}

export function renderServerMessage(message: ServerMessage): string {
  switch (message.type) {
    case "stream_token":
      return message.content;
    case "action_event":
      return `[Agent ${message.action}: ${message.target}]`;
    case "lsp_hover_request":
      return `[LSP hover requested: ${message.id} ${message.path}:${message.line}:${message.character}]`;
    case "suspend":
      return `[Agent suspended${message.requires_feedback ? ": feedback required" : ""}]\n${message.instruction}\n`;
    case "handshake":
      return `[Plan approval requested]\n${JSON.stringify(message.plan)}\n`;
    case "escalated":
      return `WARNING: RABBIT HOLE DETECTED. Handing over Situation Report...\n${message.report}\n`;
    case "kiln":
      return `${message.message}\n`;
    case "error":
      return `[Agent error: ${message.message}]`;
  }
}

export function handleWsText(payload: string): string {
  return renderServerMessage(decodeServerMessage(payload));
}

export function connect(opts: WsClientOptions): WsClientHandle {
  const ws = new WebSocket(opts.endpoint);
  const onText = opts.onText ?? ((rendered: string) => process.stdout.write(rendered));

  const done = new Promise<void>((resolveDone, rejectDone) => {
    ws.once("open", () => {
      ws.send(encodeClientMessage(opts.init));
    });
    ws.on("message", (data: RawData) => {
      const text = typeof data === "string" ? data : data.toString();
      if (isFinishAction(text)) {
        opts.onFinish();
      }
      try {
        onText(handleWsText(text));
      } catch {
        // Drop messages we cannot render rather than crashing the receive loop.
      }
    });
    ws.once("error", rejectDone);
    ws.once("close", () => resolveDone());
  });

  return {
    send: (message: ClientMessage) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(encodeClientMessage(message));
      }
    },
    close: () =>
      new Promise<void>((resolveClose) => {
        if (ws.readyState === WebSocket.CLOSED) {
          resolveClose();
          return;
        }
        ws.once("close", () => resolveClose());
        ws.close();
      }),
    done,
  };
}
