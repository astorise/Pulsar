import * as vscode from 'vscode';
import { ChatViewProvider } from './ChatViewProvider';
import { PulsarClient } from './PulsarClient';

export function activate(context: vscode.ExtensionContext): void {
  const client = new PulsarClient();
  const provider = new ChatViewProvider(context.extensionUri, client);

  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(ChatViewProvider.viewType, provider),
    vscode.commands.registerCommand('pulsar.sendSelection', () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return;
      }
      const selection = editor.selection.isEmpty
        ? editor.document.getText()
        : editor.document.getText(editor.selection);
      const prefix = `File: ${editor.document.uri.toString()}\n\n`;
      provider.sendSelection(`${prefix}${selection}`);
    }),
    client
  );
}

export function deactivate(): void {}
