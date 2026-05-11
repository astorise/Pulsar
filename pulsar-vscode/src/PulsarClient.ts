import * as vscode from 'vscode';
import { ClientMessage, ServerMessage, parseServerMessage } from './protocol';

export type PulsarEvent =
  | { kind: 'status'; text: string }
  | { kind: 'message'; message: ServerMessage };

export class PulsarClient {
  private socket: WebSocket | undefined;
  private readonly listeners = new Set<(event: PulsarEvent) => void>();

  onEvent(listener: (event: PulsarEvent) => void): vscode.Disposable {
    this.listeners.add(listener);
    return new vscode.Disposable(() => this.listeners.delete(listener));
  }

  connect(): void {
    this.close();

    const config = vscode.workspace.getConfiguration('pulsar');
    const orchestratorUrl = config.get<string>('orchestratorUrl', '');
    const workspaceUrl = config.get<string>('workspaceUrl', '');
    const workspaceToken = config.get<string>('workspaceToken', '');

    if (!orchestratorUrl || !workspaceUrl) {
      this.emit({ kind: 'status', text: 'Missing Pulsar connection settings.' });
      return;
    }

    this.socket = new WebSocket(orchestratorUrl);
    this.socket.addEventListener('open', () => {
      this.sendRaw({
        type: 'init',
        workspace_url: workspaceUrl,
        workspace_token: workspaceToken
      });
      this.emit({ kind: 'status', text: 'Connected to Pulsar.' });
    });
    this.socket.addEventListener('close', () => {
      this.emit({ kind: 'status', text: 'Disconnected from Pulsar.' });
    });
    this.socket.addEventListener('error', () => {
      this.emit({ kind: 'status', text: 'Pulsar connection error.' });
    });
    this.socket.addEventListener('message', (event) => {
      if (typeof event.data !== 'string') {
        return;
      }
      try {
        this.emit({ kind: 'message', message: parseServerMessage(event.data) });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        this.emit({ kind: 'status', text: message });
      }
    });
  }

  sendUserMessage(content: string): void {
    this.sendRaw({ type: 'user_message', content });
  }

  close(): void {
    this.socket?.close();
    this.socket = undefined;
  }

  dispose(): void {
    this.close();
    this.listeners.clear();
  }

  private sendRaw(message: ClientMessage): void {
    if (this.socket?.readyState !== WebSocket.OPEN) {
      this.emit({ kind: 'status', text: 'Pulsar is not connected.' });
      return;
    }
    this.socket.send(JSON.stringify(message));
  }

  private emit(event: PulsarEvent): void {
    for (const listener of this.listeners) {
      listener(event);
    }
  }
}
