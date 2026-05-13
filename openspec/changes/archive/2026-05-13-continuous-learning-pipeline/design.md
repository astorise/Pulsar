# Design: Native LoRA Fine-Tuning Architecture

## 1. Trace Interception & ShareGPT Formatting
When the Swarm Supervisor successfully merges a complex task, the `faas/skill-extractor` formats the execution trace into a strict ShareGPT/Alpaca JSONL format (filtering out failed attempts to only keep the "Happy Path").

## 2. Dataset Storage
The resulting JSON object is appended to an immutable dataset file via the `embedded-core-store` or WebDAV.

## 3. Native Training Trigger (Fire-and-Forget)
[cite_start]When the dataset reaches a critical mass (e.g., 500 examples), the FaaS calls the `tachyon:ai/training` interface's `submit-job` function. [cite_start]The FaaS immediately terminates, preserving sub-millisecond latency.

## 4. Background Execution & Model Brokering
[cite_start]The Tachyon host executes the LoRA training via a low-priority message broker (`system-faas-buffer`). [cite_start]The internal Candle engine manages the autograd tree, offloading optimizer states and gradients to system DDR RAM if VRAM is pressured. [cite_start]Upon completion, the new `.safetensors` adapter is hashed and registered in the `system-faas-model-broker`, ready for the next FaaS inference call.