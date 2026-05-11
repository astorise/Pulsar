# Proposal: Inference Gateway FaaS

## Problem Statement
Various components of the Pulsar ecosystem (the local CLI, the background Skill Extractor) need to generate text using the locally hosted LLMs (Qwen 7B/27B). Directly binding raw `wasi-nn` in every component leads to duplicated code, poor error handling, and hardcoded tensor shapes. Furthermore, we need a unified way to apply LoRA adapters dynamically (like the "Uncertainty LoRA" for the Tier 1 edge model) without restarting the model in VRAM.

## Vision
We introduce `faas/inference-gateway`. This WebAssembly component acts as the central AI router on the Tachyon Mesh. It abstracts the low-level `wasi-nn` tensor manipulation into a clean, developer-friendly interface. 

## Value Proposition
- **Unified Interface:** Components just send a string prompt and get text back.
- **Dynamic LoRA:** The gateway accepts an `adapter-id` parameter. Thanks to Tachyon's zero-cold-start architecture, it maps this ID to the specific LoRA weights on disk and applies them to the base model's computation graph per-request.
- **Protocol Translation:** It exposes a standard WIT interface for internal FaaS communication (like `skill-extractor`), which the Tachyon host automatically exposes as a bidi-streaming gRPC endpoint for external clients (like the `pulsar-cli`).