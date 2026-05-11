## ADDED Requirements

### Requirement: VS Code Web Extension Runtime
The system SHALL provide a VS Code web extension that runs in browser-compatible extension hosts without native binaries or local AI runtime dependencies.

#### Scenario: Extension activates in VS Code Web
- **GIVEN** the extension is installed in a web extension host
- **WHEN** the Pulsar chat view is opened
- **THEN** the extension activates through its browser bundle and renders the chat view

### Requirement: Pulsar Sidebar Chat View
The extension SHALL contribute a Pulsar activity-bar container and a chat webview view for interactive user prompts.

#### Scenario: User opens Pulsar sidebar
- **GIVEN** VS Code has loaded the extension contributions
- **WHEN** the user opens the Pulsar activity view
- **THEN** a chat input and transcript area are displayed

### Requirement: Orchestrator WebSocket Client
The extension SHALL connect to the configured Tachyon orchestrator URL using the browser WebSocket API and send the standard `init` payload.

#### Scenario: WebSocket opens
- **GIVEN** `pulsar.orchestratorUrl`, `pulsar.workspaceUrl`, and `pulsar.workspaceToken` are configured
- **WHEN** the WebSocket connection opens
- **THEN** the first outbound message contains `type: init`, `workspace_url`, and `workspace_token`

### Requirement: Streaming Message Rendering
The extension SHALL parse orchestrator JSON messages and render streamed tokens and action events in the webview.

#### Scenario: Orchestrator sends a token
- **GIVEN** the WebSocket receives a `stream_token` message
- **WHEN** the extension parses it
- **THEN** the token content is appended to the chat transcript

### Requirement: Editor Selection Command
The extension SHALL provide `pulsar.sendSelection` to send the selected text or active document content to the orchestrator.

#### Scenario: User sends selected code
- **GIVEN** an editor has selected text
- **WHEN** the user runs `Pulsar: Send Selection`
- **THEN** the extension sends a `user_message` including the selected text and file URI

### Requirement: VSIX Packaging
The build pipeline SHALL package the web extension as a VSIX artifact using `vsce --target web`.

#### Scenario: Build workflow runs
- **GIVEN** the GitHub Actions build workflow starts
- **WHEN** dependencies are installed
- **THEN** the workflow builds the browser bundle, packages a web-target VSIX, and uploads it as an artifact
