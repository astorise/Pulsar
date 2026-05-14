# Repository Hygiene and Secrets Delta

## ADDED Requirements

### Requirement: Runtime Secret Access
The orchestrator SHALL avoid persisting workspace tokens in static session state and retrieve them only when needed for authorization.

#### Scenario: Session is stored
- **GIVEN** a session is created
- **WHEN** the orchestrator persists session configuration
- **THEN** the workspace token is not stored in static session state

#### Scenario: Authorized request is sent
- **GIVEN** a workspace API call needs authorization
- **WHEN** the request is built
- **THEN** the token is retrieved from the runtime environment or host secret interface and applied directly to the request header

### Requirement: Workspace CI Coverage and Secret Scanning
The GitHub Actions test workflow SHALL validate the full Rust workspace and scan the repository for committed secrets.

#### Scenario: CI runs Rust checks
- **GIVEN** the test workflow is triggered
- **WHEN** Rust validation runs
- **THEN** it executes workspace-level tests and Clippy checks

#### Scenario: CI scans secrets
- **GIVEN** the test workflow is triggered
- **WHEN** the hygiene checks run
- **THEN** gitleaks scans the repository for committed secrets

### Requirement: Repository Documentation and Active Spec Scope
The repository SHALL document its Tachyon-Mesh dependency, include licensing, describe each FaaS bounded context, and move inactive phantom specs out of active scope.

#### Scenario: Documentation is reviewed
- **GIVEN** a contributor opens the repository
- **WHEN** they read README and FaaS documentation
- **THEN** they can distinguish target architecture from the current MVP and understand that Pulsar requires Tachyon-Mesh

#### Scenario: Active specs are listed
- **GIVEN** OpenSpec validates active specifications
- **WHEN** inactive knowledge-graph and token-killer specs are no longer in scope
- **THEN** they are archived outside the active specs directory rather than deleted
