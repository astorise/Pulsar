# orchestrator Specification

## Purpose
TBD - created by archiving change faas-orchestrator. Update Purpose after archive.
## Requirements
### Requirement: Orchestrator Session Contract
The system SHALL provide a `tachyon:ai/orchestrator` interface that starts sessions from CLI-provided workspace configuration and accepts user input for an active session.

#### Scenario: CLI starts an orchestrator session
- **GIVEN** the CLI provides a WebDAV workspace URL, workspace token, and maximum escalation tier
- **WHEN** it calls `start-session`
- **THEN** the orchestrator validates the configuration and returns a stable session identifier

### Requirement: WebDAV Workspace Access
The orchestrator SHALL access workspace files through the CLI-provided WebDAV bridge for file reads, writes, and directory discovery.

#### Scenario: Agent edits a workspace file
- **GIVEN** the inference result asks to edit a repo-relative file
- **WHEN** the orchestrator executes the tool call
- **THEN** it writes the new contents through the WebDAV bridge for the active session

### Requirement: Agentic Think Act Observe Loop
The orchestrator SHALL maintain an execution trace and move through thinking, acting, observing, awaiting-user, and finished states while processing submitted input.

#### Scenario: User submits a task
- **GIVEN** an active session
- **WHEN** the CLI submits a user prompt
- **THEN** the orchestrator records the prompt, gathers context, asks inference for the next tool call, executes the tool, and records the observation

### Requirement: Viking Context Integration
The orchestrator SHALL query `tachyon:ai/viking-context` for semantic L0/L1 context before falling back to raw file operations.

#### Scenario: Prompt references a source path
- **GIVEN** a user prompt containing a repo path
- **WHEN** the orchestrator prepares the inference prompt
- **THEN** it resolves the corresponding `viking://` URI and includes semantic context in the model prompt

### Requirement: Async Skill Extraction Trigger
The orchestrator SHALL send the successful task description and serialized execution trace to `tachyon:ai/skill-extractor` when a task reaches the finished state.

#### Scenario: Agent completes a task
- **GIVEN** the selected tool call is `finish`
- **WHEN** the orchestrator records completion
- **THEN** it submits the task and trace to the skill extractor without blocking the completed response on learning failure

