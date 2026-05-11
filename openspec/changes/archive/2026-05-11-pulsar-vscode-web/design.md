# Design: pulsar-vscode

## Architecture Overview
The extension acts as a graphical alternative to `pulsar-cli`.

1. **Webview UI (The View):**
   - Registers a `WebviewViewProvider` in the VS Code sidebar (`workbench.view.extension.pulsar-chat`).
   - Renders a chat interface using vanilla HTML/CSS/JS (or a lightweight framework like Preact) compiled via esbuild.

2. **Extension Host (The Controller):**
   - Reads the `pulsar.orchestratorUrl` from VS Code settings.
   - Maintains a `WebSocket` connection to the Orchestrator.
   - Translates UI actions into JSON protocol messages (`Init`, `UserMessage`).
   - Streams incoming `StreamToken` messages to the Webview.

3. **Workspace Bridge (The Model):**
   - For Phase 1, the extension expects the user to have the local `pulsar-cli` running in the background to serve the WebDAV bridge (`workspace_url`).
   - The extension will read the `workspace_url` and `workspace_token` from the local environment or settings and pass them in the `Init` payload.