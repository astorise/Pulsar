# skill-extractor Specification

## Purpose
TBD - created by archiving change faas-skill-extractor. Update Purpose after archive.
## Requirements
### Requirement: Skill Extractor WIT Contract
The system SHALL provide a `tachyon:ai/skill-extractor` interface that accepts a completed task description and serialized execution trace, then returns the saved skill URI and learned concept.

#### Scenario: Agent submits successful trace
- **GIVEN** a completed task and its execution trace
- **WHEN** the agent calls `extract`
- **THEN** the extractor returns an `extraction-response` with the path of the generated skill and a short learned concept

### Requirement: Deterministic Meta Prompt
The extractor SHALL build a meta prompt that instructs the local Tier 2 model to produce only a Markdown `SKILL.md` document with context, deterministic steps, and commands/tools.

#### Scenario: Trace contains dead ends
- **GIVEN** an execution trace that includes failed attempts before success
- **WHEN** the extractor builds the prompt
- **THEN** the prompt instructs the model to avoid the dead ends and preserve the successful workflow

### Requirement: Inference And Storage
The extractor SHALL call `tachyon:ai/inference` with the generated meta prompt and store the returned Markdown through `tachyon:mesh/storage-broker`.

#### Scenario: Model returns fenced Markdown
- **GIVEN** the inference response contains fenced Markdown
- **WHEN** the extractor stores the generated skill
- **THEN** it strips the fence, writes `.pulsar/skills/<sanitized-task-name>.md`, and returns that path

