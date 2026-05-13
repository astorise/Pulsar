# supervisor Specification

## Purpose
TBD - created by archiving change swarm-supervisor. Update Purpose after archive.
## Requirements
### Requirement: Task Decomposition
The supervisor FaaS SHALL analyze high-level user directives and partition them into non-overlapping sub-tasks.

#### Scenario: User asks for a broad feature
- **GIVEN** a directive spans multiple files or components
- **WHEN** the supervisor maps the request
- **THEN** it emits independent sub-tasks with explicit ownership hints

### Requirement: Distributed Orchestration
The supervisor SHALL track child orchestrator jobs through running, failed, and succeeded lifecycle states.

#### Scenario: Child jobs are spawned
- **GIVEN** multiple sub-tasks were produced
- **WHEN** the supervisor starts execution
- **THEN** each child job is tracked until it reaches a terminal state

### Requirement: Worktree Merge Bridge
The `workspace-bridge` interface SHALL support `merge-worktree` and return conflicting file paths.

#### Scenario: A child branch is reduced
- **GIVEN** a child worktree branch has completed
- **WHEN** the supervisor asks the bridge to merge it
- **THEN** the bridge returns an empty list for success or paths with unresolved conflicts

### Requirement: Token Budget Circuit Breaker
The swarm SHALL stop gracefully when the shared token counter exceeds the configured session budget.

#### Scenario: Budget is exhausted
- **GIVEN** child orchestrators increment a shared token counter
- **WHEN** the counter passes the allowed limit
- **THEN** the supervisor halts remaining automated work
