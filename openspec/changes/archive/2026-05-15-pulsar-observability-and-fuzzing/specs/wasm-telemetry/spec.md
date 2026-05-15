# wasm-telemetry Specification

## ADDED Requirements

### Requirement: Wasm Tracing Subscriber
The orchestrator SHALL initialize a Wasm-compatible tracing subscriber at session startup.

#### Scenario: Session starts
- **GIVEN** the orchestrator component receives `start-session`
- **WHEN** the session startup path begins
- **THEN** the tracing subscriber is initialized exactly once before session state is stored

#### Scenario: Telemetry is host-readable
- **GIVEN** tracing events are emitted inside the Wasm component
- **WHEN** the subscriber formats them
- **THEN** events are emitted as structured JSON suitable for collection by the Tachyon host

### Requirement: Static Regex Safety Notes
Static regex initialization in the orchestrator sanitizer SHALL document why fail-fast initialization is acceptable.

#### Scenario: Audit reviews sanitizer regexes
- **GIVEN** static regexes are initialized through `LazyLock`
- **WHEN** the sanitizer source is reviewed
- **THEN** comments explain that the patterns are static crate assets and invalid patterns should fail fast
