## ADDED Requirements

### Requirement: Table-Based MCP Observations
The orchestrator SHALL persist swarm observations in an isolated `mcp_observations` resource table.

#### Scenario: Observation is broadcast
- **GIVEN** an agent records useful context
- **WHEN** it broadcasts the observation
- **THEN** the content is stored with session, timestamp, related files, and text fields

### Requirement: Chronological Observation Keys
The orchestrator SHALL encode observation timestamps in big-endian sortable form so table range scans return chronological results.

#### Scenario: Keys are sorted lexicographically
- **GIVEN** two observations have different timestamps
- **WHEN** their encoded keys are sorted as strings
- **THEN** the earlier timestamp sorts before the later timestamp

### Requirement: Paginated Intel Sync
The orchestrator SHALL fetch recent swarm observations with paginated range queries using a bounded `limit`.

#### Scenario: Many observations exist
- **GIVEN** the table contains more observations than one page
- **WHEN** recent intel is fetched
- **THEN** the orchestrator requests chunks and stops at the context budget
