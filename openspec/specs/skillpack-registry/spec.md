# skillpack-registry Specification

## Purpose
TBD - created by archiving change dynamic-skillpack-registry. Update Purpose after archive.
## Requirements
### Requirement: Markdown Skill Definitions
The system SHALL support skill definitions as Markdown files with YAML frontmatter describing name, summary, prompts, and allowed tools.

#### Scenario: Skill file is compiled
- **GIVEN** a `SKILL.md` file has frontmatter and body content
- **WHEN** the skill compiler reads it
- **THEN** the resulting definition preserves metadata and the system prompt

### Requirement: Precompiled Skill Cache
The CLI SHALL compile skills into MessagePack records before runtime so FaaS invocation does not need to read Markdown files from disk.

#### Scenario: Skillify command runs
- **GIVEN** `.pulsar/skills` contains skill Markdown files
- **WHEN** the developer runs `pulsar-cli skillify`
- **THEN** compiled skill records are written to the registry table payload

### Requirement: Vector-Based Routing
The supervisor SHALL choose the best skill for a delegated task using cosine similarity against cached skill embeddings.

#### Scenario: Supervisor delegates a sub-task
- **GIVEN** skill embeddings exist in the registry
- **WHEN** a sub-task is mapped
- **THEN** the supervisor picks the highest scoring skill above the threshold
