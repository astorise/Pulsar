## ADDED Requirements

### Requirement: Human Action Tool
The orchestrator SHALL expose `request_human_action` for physical checks, secrets, visual judgment, or other qualitative delegation.

#### Scenario: Agent needs a human-only check
- **GIVEN** the model cannot verify a physical or visual condition
- **WHEN** it calls `request_human_action`
- **THEN** the orchestrator suspends and emits the instruction for the developer

### Requirement: Stateless Resumption
The orchestrator SHALL serialize suspended session state to host KV storage and reload it when feedback is received.

#### Scenario: Developer provides feedback
- **GIVEN** a session was suspended for human action
- **WHEN** `resume_session` receives feedback
- **THEN** the trace is restored and appended with the human observation

### Requirement: CLI Suspension Interaction
The CLI SHALL represent suspension messages distinctly and send a resume request after feedback is entered.

#### Scenario: CLI receives suspension
- **GIVEN** the server sends a suspend message
- **WHEN** the CLI displays it
- **THEN** normal chat is blocked until the feedback is submitted
