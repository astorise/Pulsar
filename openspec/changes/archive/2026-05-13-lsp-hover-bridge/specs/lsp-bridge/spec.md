## ADDED Requirements

### Requirement: Workspace LSP Hover Bridge
The `workspace-bridge` interface SHALL expose `lsp-hover(session-id, path, line, character)` and return Markdown hover documentation.

#### Scenario: Orchestrator asks for a signature
- **GIVEN** the agent is unsure about a symbol signature
- **WHEN** it calls the LSP hover bridge with a file coordinate
- **THEN** the bridge returns compiler or language-server hover text

### Requirement: CLI Hover Message Correlation
The CLI protocol SHALL support `lsp_hover_request` and `lsp_hover_response` messages with correlation identifiers.

#### Scenario: Blocking hover request completes
- **GIVEN** the orchestrator sends a hover request to the connected client
- **WHEN** the client responds with the same correlation id
- **THEN** the waiting bridge call receives the hover Markdown

### Requirement: VS Code Hover Execution
The VS Code extension SHALL execute `vscode.executeHoverProvider` for hover requests and return formatted Markdown content.

#### Scenario: Extension receives hover request
- **GIVEN** a hover request includes a URI, line, and character
- **WHEN** VS Code returns hover entries
- **THEN** the extension sends a single Markdown response back to Pulsar
