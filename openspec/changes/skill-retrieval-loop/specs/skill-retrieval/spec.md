# Specification: Procedural Memory Loop

## Requirement: Skill Discovery
The Orchestrator SHALL attempt to discover learned skills at the beginning of a reasoning cycle before querying the inference gateway.

### Scenario: User asks to refactor a component
- **GIVEN** a `.pulsar/skills/refactor-component.md` exists in the workspace
- **WHEN** the user inputs "Refactor the authentication component"
- **THEN** the orchestrator discovers the skill via WebDAV PROPFIND and loads its content.

## Requirement: Prompt Injection
The Orchestrator SHALL inject the loaded skill content into the LLM system prompt without altering the primary task directive.

### Scenario: Inference context generation
- **GIVEN** a loaded skill markdown payload
- **WHEN** `build_inference_prompt` is executed
- **THEN** the prompt template includes a "Relevant Past Experience" block containing the skill.