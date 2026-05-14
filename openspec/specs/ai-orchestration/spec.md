# ai-orchestration Specification

## Purpose
Define how Pulsar selects learned skills, escalates inference across model tiers, and records escalation events for later learning and debugging.
## Requirements
### Requirement: Skill-Aware Planning
The orchestrator SHALL select a relevant learned skill before model inference and pass the selected adapter into inference requests when one exists.

#### Scenario: Learned skill is selected
- **GIVEN** a user prompt enters the orchestration loop
- **WHEN** the supervisor identifies a matching learned skill
- **THEN** the orchestrator includes the returned LoRA adapter identifier in the inference request

#### Scenario: No learned skill matches
- **GIVEN** a user prompt enters the orchestration loop
- **WHEN** the supervisor does not identify a matching learned skill
- **THEN** the orchestrator sends the inference request without a LoRA adapter

### Requirement: Tiered Inference Escalation
The orchestrator SHALL retry inference across configured model tiers when recoverable escalation triggers occur.

#### Scenario: Resource exhaustion escalates
- **GIVEN** a tier-one inference request fails with resource exhaustion
- **WHEN** tier two is configured
- **THEN** the orchestrator retries the same prompt and adapter against tier two

#### Scenario: Rabbit-hole marker escalates
- **GIVEN** a model response contains `<rabbit_hole_detected>`
- **WHEN** another tier is available
- **THEN** the orchestrator records the escalation event and retries on the next tier

### Requirement: Escalation Observability
The orchestrator SHALL record each inference escalation event for later learning and debugging.

#### Scenario: Escalation occurs
- **GIVEN** an escalation trigger is observed
- **WHEN** the orchestrator retries on another tier
- **THEN** it emits an escalation event containing the source tier, target tier, and trigger reason
