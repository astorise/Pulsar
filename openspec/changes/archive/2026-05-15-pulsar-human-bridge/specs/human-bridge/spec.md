# human-bridge Specification

## ADDED Requirements

### Requirement: Approval Contract Import
The orchestrator Wasm component SHALL import a Human Bridge approval contract with `approved`, `rejected`, and `modified` outcomes.

#### Scenario: Component requests approval
- **GIVEN** the orchestrator evaluates a high-impact operation
- **WHEN** the operation exceeds the approval threshold
- **THEN** the component calls the Human Bridge approval contract with the session id, summary, and impact level

### Requirement: Impact Scoring
The orchestrator SHALL score generated tool calls before executing state-mutating operations.

#### Scenario: Read-only operation is low impact
- **GIVEN** a read-only context or graph tool call
- **WHEN** the impact scorer evaluates it
- **THEN** the score is below the approval threshold

#### Scenario: Mutating operation requires approval
- **GIVEN** an edit or high-impact command such as `git push`
- **WHEN** the impact scorer evaluates it
- **THEN** the score exceeds the approval threshold

### Requirement: Approval Result Handling
The orchestrator SHALL handle Human Bridge approval results before executing gated operations.

#### Scenario: Human approves operation
- **GIVEN** a high-impact operation is awaiting approval
- **WHEN** the Human Bridge returns `approved`
- **THEN** the orchestrator executes the original operation

#### Scenario: Human rejects operation
- **GIVEN** a high-impact operation is awaiting approval
- **WHEN** the Human Bridge returns `rejected`
- **THEN** the orchestrator records the rejection and does not execute the operation

#### Scenario: Human modifies command
- **GIVEN** a high-impact command is awaiting approval
- **WHEN** the Human Bridge returns `modified` with a replacement command
- **THEN** the orchestrator reparses and validates the replacement before executing it
