# Orchestrator Security Hardening Delta

## ADDED Requirements

### Requirement: Workspace Path Guard
The orchestrator SHALL validate all LLM-provided file paths against a configured workspace root before any file access.

#### Scenario: Path traversal is rejected
- **GIVEN** the orchestrator is configured with a workspace root
- **WHEN** a tool call requests a path containing a null byte, an explicit parent-directory component, or a canonical path outside the workspace root
- **THEN** the orchestrator returns an unauthorized access error without touching the file system

#### Scenario: Workspace path is accepted
- **GIVEN** the orchestrator is configured with a workspace root
- **WHEN** a tool call requests a file path that canonicalizes inside that workspace root
- **THEN** the orchestrator allows the operation to proceed with the guarded path

### Requirement: Command Execution Sandbox
The orchestrator SHALL execute commands only through a parsed executable plus argument list and an explicit allowlist.

#### Scenario: Disallowed command is rejected
- **GIVEN** a model requests a command executable outside the configured allowlist
- **WHEN** the command handler validates the request
- **THEN** it returns a command forbidden error without spawning a process

#### Scenario: Shell injection is rejected
- **GIVEN** a model provides a legacy single-string command containing shell operators such as `;`, `|`, or `&&`
- **WHEN** the command handler parses the request
- **THEN** it rejects the command as a command injection attempt

### Requirement: Graceful Tool Error Responses
The orchestrator SHALL return structured tool-call errors to the LLM instead of panicking.

#### Scenario: Tool call fails
- **GIVEN** a guarded path or sandboxed command operation fails
- **WHEN** the tool-call response is formatted
- **THEN** the response contains a structured error status, reason, and message suitable for model self-correction
