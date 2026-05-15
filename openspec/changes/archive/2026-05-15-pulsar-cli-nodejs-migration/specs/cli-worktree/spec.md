## MODIFIED Requirements

### Requirement: Transparent Git Worktree Sandboxing
The Node.js CLI SHALL avoid exposing the live current working directory through WebDAV when started inside a Git repository, and SHALL instead expose an isolated git worktree rooted under `.pulsar/worktrees/` using `simple-git` (or equivalent Node.js git bindings).

#### Scenario: CLI initializes in a Git repository
- **GIVEN** the Node.js CLI is started inside a valid Git repository
- **WHEN** the WebDAV server is configured
- **THEN** the CLI creates a session branch and worktree under `.pulsar/worktrees/<session-id>` via `simple-git`
- **AND** the WebDAV server serves that worktree path instead of the live repository path

### Requirement: Review Before Applying Changes
The Node.js CLI SHALL display a diff stat from the sandbox using an interactive terminal prompt (e.g. `inquirer`) and require explicit `[y/N]` user confirmation before applying sandbox changes to the active repository.

#### Scenario: User declines changes
- **GIVEN** the sandbox contains file modifications
- **WHEN** the finish teardown prompts the user with `inquirer` and the user declines
- **THEN** the CLI removes the sandbox without applying changes to the active worktree
