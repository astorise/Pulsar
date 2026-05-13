# handshake Specification

## Purpose
TBD - created by archiving change context-handshake. Update Purpose after archive.
## Requirements
### Requirement: Mandatory Planning
The orchestrator SHALL reject file-system or command-execution tools until `submit_plan` has been invoked and approved in the current session.

#### Scenario: Model skips planning
- **GIVEN** a session has no approved plan
- **WHEN** the model attempts to run a command or edit a file
- **THEN** the orchestrator records a local error asking for `submit_plan`

### Requirement: Zero-Cost Plan Wait
The handshake SHALL use suspended session serialization so host compute resources can be released while waiting for approval.

#### Scenario: Plan waits for approval
- **GIVEN** the model submits a plan
- **WHEN** approval is required
- **THEN** the session state is serialized under the suspended-session key

### Requirement: Approved Plan Pinning
After approval, the orchestrator SHALL pin the approved plan to the prompt context as immutable truth.

#### Scenario: Acting resumes
- **GIVEN** a plan has been approved
- **WHEN** the next inference prompt is built
- **THEN** the prompt includes the approved plan section before transient context
