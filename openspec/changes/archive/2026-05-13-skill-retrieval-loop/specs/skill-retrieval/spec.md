## ADDED Requirements

### Requirement: Skill Discovery
The orchestrator SHALL attempt to discover learned skills at the beginning of a reasoning cycle before querying inference.

#### Scenario: User asks to refactor a component
- **GIVEN** `.pulsar/skills/refactor-component.md` exists in the workspace
- **WHEN** the user inputs "Refactor the authentication component"
- **THEN** the orchestrator discovers candidate skills via WebDAV PROPFIND

### Requirement: Skill Matching
The orchestrator SHALL select a candidate skill using keyword overlap between the prompt and skill filenames.

#### Scenario: Matching skill is available
- **GIVEN** multiple skill files exist
- **WHEN** the prompt shares keywords with one skill filename
- **THEN** the orchestrator selects the highest scoring skill

### Requirement: Skill Prompt Injection
The orchestrator SHALL inject loaded skill content into the LLM prompt without replacing the primary task directive.

#### Scenario: Inference prompt is generated
- **GIVEN** a matching skill was fetched
- **WHEN** `build_inference_prompt` runs
- **THEN** the prompt includes a clearly marked past experience section
