# Specification: Browser Automation Protocol

## Requirement: MicroVM Ephemerality
When utilizing the `smolvm` engine profile, the Tachyon host MUST guarantee that the MicroVM is completely destroyed immediately upon the Wasm component dropping the CDP session resource or upon FaaS execution timeout.

## Requirement: Egress Restriction
The `smolvm` instance MUST NOT be granted generic internet egress. Its network namespace MUST be strictly limited to communicating with the specified local preview server to prevent supply-chain attacks via `npm install` during the rendering phase.

## Requirement: Vision Integration
The `system-faas-model-broker` MUST support routing multimodal queries (text + image bytes) to a designated Vision-LLM endpoint (either a local Candle instance or a clustered vLLM node).