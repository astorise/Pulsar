## ADDED Requirements

### Requirement: Transparent Git Worktree Sandboxing
The CLI SHALL avoid exposing the live current working directory through WebDAV when started inside a Git repository, and SHALL instead expose an isolated git worktree rooted under `.pulsar/worktrees/`.

#### Scenario: CLI initializes in a Git repository
- **GIVEN** the CLI is started inside a valid Git repository
- **WHEN** the WebDAV server is configured
- **THEN** the CLI creates a session branch and worktree under `.pulsar/worktrees/<session-id>`
- **AND** the WebDAV server serves that worktree path instead of the live repository path

### Requirement: Local Worktree Exclusion
The CLI SHALL keep generated sandbox directories out of normal Git tracking.

#### Scenario: Sandbox is created
- **GIVEN** the repository has a `.git/info/exclude` file
- **WHEN** the CLI creates a sandbox
- **THEN** `.pulsar/` is present in the local exclude file

### Requirement: Finish Event Detection
The CLI SHALL detect orchestrator `action_event` messages with action `finish` and use them to trigger sandbox teardown.

#### Scenario: Orchestrator signals completion
- **GIVEN** the WebSocket receives `{"type":"action_event","action":"finish","target":"session"}`
- **WHEN** the receive loop parses the message
- **THEN** it signals the main CLI lifecycle to review and clean up the sandbox

### Requirement: Review Before Applying Changes
The CLI SHALL display a diff stat from the sandbox and require explicit user confirmation before applying sandbox changes to the active repository.

#### Scenario: User declines changes
- **GIVEN** the sandbox contains file modifications
- **WHEN** the finish teardown prompts the user and the user declines
- **THEN** the CLI removes the sandbox without applying changes to the active worktree

### Requirement: Sandbox Cleanup
The CLI SHALL remove the git worktree and temporary branch during teardown regardless of whether the user accepts the changes.

#### Scenario: Teardown completes
- **GIVEN** the sandbox review flow has completed
- **WHEN** cleanup runs
- **THEN** the CLI removes the worktree path and deletes the session branch
