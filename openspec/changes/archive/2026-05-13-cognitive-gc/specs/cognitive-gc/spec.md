## ADDED Requirements

### Requirement: Embedding Interface
The `inference` interface SHALL expose `embed(model, text)` for semantic vector generation.

#### Scenario: Observation embedding is requested
- **GIVEN** the GC worker needs to compare observations
- **WHEN** it calls `embed`
- **THEN** the host returns a vector embedding for the text

### Requirement: Atomic Memory Compaction
The table interface SHALL support atomic swaps of deleted observation keys and inserted replacement facts.

#### Scenario: Golden fact replaces contradictions
- **GIVEN** contradictory observations were judged
- **WHEN** compaction runs
- **THEN** old keys are removed and the golden fact is inserted atomically

### Requirement: Contradiction Clustering
The cognitive GC worker SHALL cluster recent observations by cosine similarity before asking the judge model for resolution.

#### Scenario: Similar observations exist
- **GIVEN** recent observations have embedding similarity above the threshold
- **WHEN** clustering runs
- **THEN** the worker groups them for contradiction analysis
