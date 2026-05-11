import * as vscode from 'vscode';
import { PulsarClient } from './PulsarClient';

type WebviewMessage =
  | { type: 'ready' }
  | { type: 'send'; content: string };

export class ChatViewProvider implements vscode.WebviewViewProvider {
  static readonly viewType = 'pulsar.chatView';
  private view: vscode.WebviewView | undefined;

  constructor(
    private readonly extensionUri: vscode.Uri,
    private readonly client: PulsarClient
  ) {}

  resolveWebviewView(webviewView: vscode.WebviewView): void {
    this.view = webviewView;
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this.extensionUri]
    };
    webviewView.webview.html = this.render(webviewView.webview);

    webviewView.webview.onDidReceiveMessage((message: WebviewMessage) => {
      if (message.type === 'ready') {
        this.client.connect();
      }
      if (message.type === 'send' && message.content.trim()) {
        this.post({ type: 'user', text: message.content.trim() });
        this.client.sendUserMessage(message.content.trim());
      }
    });

    this.client.onEvent((event) => {
      if (event.kind === 'status') {
        this.post({ type: 'status', text: event.text });
        return;
      }
      const message = event.message;
      if (message.type === 'stream_token') {
        this.post({ type: 'token', text: message.content });
      } else if (message.type === 'action_event') {
        this.post({ type: 'status', text: `${message.action}: ${message.target}` });
      } else {
        this.post({ type: 'status', text: message.message });
      }
    });
  }

  sendSelection(text: string): void {
    if (!text.trim()) {
      return;
    }
    this.post({ type: 'user', text });
    this.client.sendUserMessage(text);
  }

  private post(message: unknown): void {
    void this.view?.webview.postMessage(message);
  }

  private render(webview: vscode.Webview): string {
    const nonce = getNonce();
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    body { margin: 0; padding: 0; color: var(--vscode-foreground); background: var(--vscode-sideBar-background); font-family: var(--vscode-font-family); }
    #log { height: calc(100vh - 60px); overflow-y: auto; padding: 10px; box-sizing: border-box; white-space: pre-wrap; }
    .user { color: var(--vscode-textLink-foreground); margin: 8px 0; }
    .status { color: var(--vscode-descriptionForeground); margin: 6px 0; }
    .token { margin: 0; }
    form { display: flex; gap: 6px; padding: 8px; border-top: 1px solid var(--vscode-sideBar-border); }
    input { flex: 1; min-width: 0; color: var(--vscode-input-foreground); background: var(--vscode-input-background); border: 1px solid var(--vscode-input-border); padding: 6px; }
    button { color: var(--vscode-button-foreground); background: var(--vscode-button-background); border: 0; padding: 6px 10px; }
  </style>
</head>
<body>
  <div id="log"></div>
  <form id="form">
    <input id="input" placeholder="Ask Pulsar" />
    <button type="submit">Send</button>
  </form>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const log = document.getElementById('log');
    const form = document.getElementById('form');
    const input = document.getElementById('input');
    function append(cls, text) {
      const node = document.createElement('div');
      node.className = cls;
      node.textContent = text;
      log.appendChild(node);
      log.scrollTop = log.scrollHeight;
    }
    window.addEventListener('message', event => append(event.data.type, event.data.text));
    form.addEventListener('submit', event => {
      event.preventDefault();
      const content = input.value.trim();
      if (!content) return;
      vscode.postMessage({ type: 'send', content });
      input.value = '';
    });
    vscode.postMessage({ type: 'ready' });
  </script>
</body>
</html>`;
  }
}

function getNonce(): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let value = '';
  for (let i = 0; i < 32; i += 1) {
    value += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return value;
}
