# e2e-testing Specification

## Purpose
Define the Wasmtime-based integration harness used to exercise Pulsar components with deterministic Tachyon-Mesh host mocks.
## Requirements
### Requirement: Wasmtime E2E Harness
The repository SHALL provide an integration test harness that exercises compiled Pulsar Wasm artifacts through Wasmtime-compatible host mocks.

#### Scenario: Harness initializes
- **GIVEN** the E2E test target is executed
- **WHEN** the harness starts
- **THEN** it initializes a Wasmtime engine, linker, and isolated temporary workspace for test execution

### Requirement: Tachyon Host Mocks
The E2E harness SHALL provide deterministic mocks for the Tachyon AI and mesh interfaces needed by Pulsar components.

#### Scenario: Mock inference escalates
- **GIVEN** the mock inference host is configured to fail tier one with resource exhaustion
- **WHEN** the orchestrator requests inference
- **THEN** the harness can assert that a later tier is attempted

#### Scenario: Mock mesh state is used
- **GIVEN** a test needs key-value or graph data
- **WHEN** the component accesses the mocked mesh interfaces
- **THEN** the harness serves deterministic in-memory responses

### Requirement: Security and Recovery E2E Scenarios
The E2E suite SHALL cover path traversal blocking, successful skill escalation, and rabbit-hole recovery.

#### Scenario: Path traversal test
- **GIVEN** a component is executed with a virtual workspace
- **WHEN** it attempts to access `../../etc/passwd`
- **THEN** the result reports unauthorized access without trapping

#### Scenario: Rabbit-hole recovery test
- **GIVEN** a tool command fails during orchestration
- **WHEN** the orchestrator builds the next prompt
- **THEN** the prompt includes the tool error context for self-correction
