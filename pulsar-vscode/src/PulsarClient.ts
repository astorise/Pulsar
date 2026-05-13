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
        const message = parseServerMessage(event.data);
        if (message.type === 'lsp_hover_request') {
          void this.respondToHoverRequest(message);
          return;
        }
        this.emit({ kind: 'message', message });
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

  private async respondToHoverRequest(message: Extract<ServerMessage, { type: 'lsp_hover_request' }>): Promise<void> {
    try {
      const uri = parseWorkspaceUri(message.path);
      const hovers = await vscode.commands.executeCommand<vscode.Hover[]>(
        'vscode.executeHoverProvider',
        uri,
        new vscode.Position(message.line, message.character)
      );
      this.sendRaw({
        type: 'lsp_hover_response',
        id: message.id,
        markdown: formatHovers(hovers ?? [])
      });
    } catch (error) {
      const text = error instanceof Error ? error.message : String(error);
      this.sendRaw({
        type: 'lsp_hover_response',
        id: message.id,
        markdown: `Hover failed: ${text}`
      });
    }
  }

  private emit(event: PulsarEvent): void {
    for (const listener of this.listeners) {
      listener(event);
    }
  }
}

function parseWorkspaceUri(path: string): vscode.Uri {
  if (/^[a-z][a-z0-9+.-]*:/i.test(path)) {
    return vscode.Uri.parse(path);
  }
  const folder = vscode.workspace.workspaceFolders?.[0]?.uri;
  return folder ? vscode.Uri.joinPath(folder, path) : vscode.Uri.file(path);
}

function formatHovers(hovers: vscode.Hover[]): string {
  return hovers
    .flatMap((hover) => hover.contents)
    .map(formatMarkedString)
    .filter((part) => part.trim().length > 0)
    .join('\n\n---\n\n');
}

function formatMarkedString(value: vscode.MarkdownString | vscode.MarkedString): string {
  if (typeof value === 'string') {
    return value;
  }
  if ('language' in value && 'value' in value) {
    return `\`\`\`${value.language}\n${value.value}\n\`\`\``;
  }
  return value.value;
}
