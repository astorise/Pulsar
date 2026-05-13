## ADDED Requirements

### Requirement: AST Graph Edge Extraction
The `viking-context` component SHALL extract graph edges for Rust imports, type declarations, trait declarations, and trait implementations during L1 parsing.

#### Scenario: Rust file contains declarations
- **GIVEN** a Rust source file contains `use`, `struct`, `trait`, and `impl Trait for Type` items
- **WHEN** Viking resolves the file at L1
- **THEN** graph edges are generated with deterministic URNs and JSON properties

### Requirement: Graph Resource Batching
The component SHALL import a graph resource and commit extracted edges in a batch after parsing completes.

#### Scenario: L1 parsing succeeds
- **GIVEN** edge extraction produced one or more edges
- **WHEN** the graph resource is available
- **THEN** Viking calls `add-edges` once for the batch

### Requirement: Graph Query Tooling
The orchestrator SHALL expose a graph query tool that returns dependent URIs or entities for a requested graph subject and depth.

#### Scenario: Agent asks for dependents
- **GIVEN** the model needs to refactor an entity safely
- **WHEN** it calls the graph query tool
- **THEN** the orchestrator records graph traversal results in the session trace

### Requirement: Edge Synchronization Helpers
The Viking graph implementation SHALL provide helpers to compute added and removed edges when a file changes.

#### Scenario: File graph changes
- **GIVEN** an old edge set and a newly parsed edge set
- **WHEN** the diff helper runs
- **THEN** it returns stale edges to delete and fresh edges to insert
