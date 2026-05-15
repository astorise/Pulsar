## ADDED Requirements

### Requirement: Node.js Runtime
The Pulsar CLI SHALL be implemented in TypeScript and execute under Node.js (LTS) so it can share interfaces with `pulsar-vscode` and be bundled by the VS Code extension.

#### Scenario: Developer runs the CLI
- **GIVEN** a developer has installed the Node.js Pulsar CLI package
- **WHEN** they execute `pulsar-cli`
- **THEN** the CLI starts under a Node.js runtime and exposes the same workspace + WebSocket bridges as the previous Rust binary

### Requirement: esbuild Bundling Pipeline
The Node.js CLI SHALL provide an `esbuild` build configuration that produces a single bundled entry point suitable for direct embedding inside the `pulsar-vscode` extension.

#### Scenario: Extension build embeds the CLI
- **GIVEN** the CLI sources live in `pulsar-cli-node/src`
- **WHEN** the build script runs `esbuild` against the entry point
- **THEN** it emits a single JavaScript bundle that the extension can spawn or require without a separate native binary

### Requirement: Shared WebSocket Protocol Types
The Node.js CLI SHALL export TypeScript interfaces for every orchestrator WebSocket message (`init`, `user_message`, `stream_token`, `action_event`) so that `pulsar-vscode` consumes the same wire shapes.

#### Scenario: Extension imports protocol types
- **GIVEN** `pulsar-vscode` depends on the CLI package
- **WHEN** it imports the protocol module
- **THEN** the `ClientMessage` and `ServerMessage` discriminated unions are available and match the Rust orchestrator ABI

## MODIFIED Requirements

### Requirement: Authenticated Workspace WebDAV Bridge
The CLI SHALL expose the current working directory through a Node.js WebDAV server (built on `webdav-server` v2) that supports file reads, file writes, and directory listing with bearer-token authorization.

#### Scenario: Orchestrator reads a file
- **GIVEN** the orchestrator sends a WebDAV request carrying the dynamically generated bearer token
- **WHEN** it requests a workspace file
- **THEN** the Node.js bridge returns the file bytes from the active worktree

### Requirement: Terminal REPL Forwarding
The CLI SHALL capture interactive terminal input via Node.js `readline` (or `inquirer` for prompts) and forward each non-empty line as a `user_message` JSON payload.

#### Scenario: User enters a prompt
- **GIVEN** the user types a non-empty prompt at the REPL
- **WHEN** `readline` emits the line
- **THEN** the CLI sends a JSON message with `type: user_message` and the prompt content over the `ws` connection
