# security-fuzzing Specification

## Purpose
Define the fuzzing coverage required to continuously exercise Pulsar's path and command security boundaries.
## Requirements
### Requirement: PathGuard Fuzz Target
The orchestrator SHALL include a cargo-fuzz target that exercises `PathGuard::validate` with arbitrary byte input.

#### Scenario: Accepted path remains normalized
- **GIVEN** fuzzed bytes are converted into a lossy string
- **WHEN** `PathGuard::validate` accepts the input
- **THEN** the resulting path contains no null bytes, absolute prefix, backslash separators, or `..` components

#### Scenario: Workspace escape is rejected
- **GIVEN** fuzzed input attempts parent traversal above the workspace
- **WHEN** `PathGuard::validate` evaluates the input
- **THEN** it returns an unauthorized access error

### Requirement: Command Lexer Fuzz Target
The orchestrator SHALL include a cargo-fuzz target that exercises legacy command parsing with arbitrary byte input.

#### Scenario: Shell operators are rejected
- **GIVEN** fuzzed command input contains shell operators or null bytes
- **WHEN** the legacy command parser evaluates it
- **THEN** the parser returns a command injection error instead of panicking

#### Scenario: Parsed command remains structured
- **GIVEN** fuzzed command input parses successfully
- **WHEN** the parser returns a command request
- **THEN** the executable is non-empty and parsed arguments contain no shell operators or null bytes

### Requirement: CI Fuzz Smoke
The repository CI SHALL run a short nightly cargo-fuzz smoke job for the orchestrator fuzz targets.

#### Scenario: Fuzz job runs
- **GIVEN** the test workflow is triggered
- **WHEN** the fuzz job runs on nightly Rust
- **THEN** both orchestrator fuzz targets execute for a bounded time budget
