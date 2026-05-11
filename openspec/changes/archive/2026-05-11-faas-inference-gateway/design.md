# Design: faas/inference-gateway

## Architecture Overview
This component is a stateless `wasm32-wasip2` function that wraps the standard `wasi:nn` API.

**Execution Flow:**
1. **Input:** Receives an `inference-request` (Model ID, Prompt, Temperature, optional LoRA ID).
2. **Graph Setup:** - Uses `wasi:nn/graph::load` to get the base model context (which Tachyon keeps hot in VRAM).
   - If a `lora-adapter` is provided, it passes this identifier via the `wasi:nn` execution context configuration so the underlying Candle/ggml backend applies the low-rank matrices.
3. **Tensor Conversion:** Tokenizes the prompt and converts it into a `wasi:nn/tensor`.
4. **Execution:** Calls `wasi:nn/inference::compute`.
5. **Output:** Decodes the output tensor into a UTF-8 string and returns it.

*(Note: For the Pulsar CLI, the Tachyon host runtime will intercept this WIT interface and expose it as a gRPC streaming endpoint to allow token-by-token terminal rendering).*

## Component Target
- Target: `wasm32-wasip2`.
- Export: `tachyon:ai/inference`.