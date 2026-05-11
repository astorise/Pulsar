# inference Specification

## Purpose
TBD - created by archiving change faas-inference-gateway. Update Purpose after archive.
## Requirements
### Requirement: Inference Gateway WIT Contract
The system SHALL provide a `tachyon:ai/inference` interface for internal FaaS components to request local LLM generation through a single gateway.

#### Scenario: Component requests blocking generation
- **GIVEN** a component has an `inference-request` with `model-id`, `prompt`, `max-tokens`, `temperature`, and optional `lora-adapter`
- **WHEN** it calls `generate`
- **THEN** the gateway returns an `inference-response` containing generated text, prompt token count, and completion token count

### Requirement: Runtime Tensor Execution
The gateway SHALL convert prompts to UTF-8 byte tensors, execute them through the host-provided Tachyon bridge to the active wasi-nn backend, and decode UTF-8 output tensors into response text.

#### Scenario: Prompt is executed through the model context
- **GIVEN** a valid inference request
- **WHEN** the gateway receives it
- **THEN** it loads the requested model graph, sets the prompt tensor as input, computes the context, and reads output slot zero

### Requirement: Dynamic LoRA Adapter Metadata
The gateway SHALL pass generation options and optional LoRA adapter selection to the host runtime as structured metadata.

#### Scenario: Request includes an adapter
- **GIVEN** an inference request with `lora-adapter` set
- **WHEN** the gateway initializes the model context
- **THEN** the adapter identifier is included in runtime metadata so the host can apply the adapter without restarting the model

