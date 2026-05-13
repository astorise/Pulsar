## ADDED Requirements

### Requirement: Consecutive Failure Tracking
The orchestrator SHALL track consecutive command failures and the last failed logical target.

#### Scenario: Command fails repeatedly
- **GIVEN** a command produces failure output for the same target
- **WHEN** the agent retries and fails again
- **THEN** the consecutive failure counter increases

### Requirement: Threshold Enforcement
The orchestrator SHALL NOT exceed three consecutive failed attempts at the same logical task before escalating.

#### Scenario: Failure threshold is reached
- **GIVEN** the same target fails three times in a row
- **WHEN** another automated retry would be attempted
- **THEN** the orchestrator generates a situation report instead

### Requirement: Situation Report Generation
The orchestrator SHALL generate a Markdown report during escalation and transmit it to the CLI or supervisor.

#### Scenario: Escalation occurs
- **GIVEN** a repeated failure has been detected
- **WHEN** the report is built
- **THEN** it contains context, attempted actions, and the requested handoff
