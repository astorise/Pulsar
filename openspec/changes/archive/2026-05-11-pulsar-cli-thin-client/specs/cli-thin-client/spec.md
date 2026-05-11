## ADDED Requirements

### Requirement: Thin Client Runtime
The Pulsar CLI SHALL run as a lightweight bridge that starts a local workspace server, connects to the Tachyon orchestrator over WebSocket, and forwards terminal input without embedding AI inference logic.

#### Scenario: CLI starts
- **GIVEN** a user launches `pulsar-cli`
- **WHEN** configuration is loaded
- **THEN** the CLI starts the workspace bridge and connects to the configured orchestrator endpoint

### Requirement: Authenticated Workspace WebDAV Bridge
The CLI SHALL expose the current working directory through a local HTTP workspace bridge that supports file reads, file writes, and directory listing with bearer-token authorization.

#### Scenario: Orchestrator reads a file
- **GIVEN** the orchestrator sends a request with the correct bearer token
- **WHEN** it requests a workspace file
- **THEN** the CLI returns the file bytes from the current working directory

### Requirement: WebSocket Handshake
The CLI SHALL send an `init` JSON message immediately after connecting to the orchestrator with the workspace URL and workspace token.

#### Scenario: WebSocket connects
- **GIVEN** the CLI has started its workspace bridge
- **WHEN** the WebSocket connection is established
- **THEN** the first message sent contains `type: init`, `workspace_url`, and `workspace_token`

### Requirement: Terminal REPL Forwarding
The CLI SHALL capture terminal input and forward each non-empty line as a `user_message` JSON payload.

#### Scenario: User enters a prompt
- **GIVEN** the user types a non-empty prompt
- **WHEN** the REPL accepts the line
- **THEN** the CLI sends a JSON message with `type: user_message` and the prompt content

### Requirement: Streaming Receive Loop
The CLI SHALL parse orchestrator messages and render `stream_token` content directly while rendering `action_event` messages as system logs.

#### Scenario: Orchestrator streams output
- **GIVEN** the CLI receives a `stream_token`
- **WHEN** the receive loop handles the message
- **THEN** the token content is printed to stdout
