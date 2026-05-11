# Implementation Tasks for AI Agent

- [x] **Task 1: WIT Integration**
  - Create the file `wit/ai/inference.wit` using the definition from the spec.
  - Update the main Tachyon world to export this interface.
  - Import the standard `wasi:nn/tensor` and `wasi:nn/inference` interfaces.

- [x] **Task 2: FaaS Scaffolding**
  - Initialize a new Rust WASM project in `faas/inference-gateway/`.
  - Add dependencies: `wasi-nn` (Rust bindings), `tachyon-sdk`, `anyhow`, and `serde_json`.

- [x] **Task 3: Implement Context Initialization**
  - Implement the `generate` function.
  - Use the `wasi-nn` bindings to load the graph specified by `req.model-id`.
  - Initialize the execution context.
  - If `req.lora-adapter` is `Some`, apply it. *(Depending on the exact wasi-nn engine implementation, this is typically done via `context.set_input` with a special metadata tensor, or a custom host function).*

- [x] **Task 4: Tensor Execution**
  - Convert the `req.prompt` string into a byte array tensor.
  - Set the tensor as the input for the execution context.
  - Call the compute function.

- [x] **Task 5: Output Processing**
  - Retrieve the output tensor.
  - Decode the bytes back into a Rust `String`.
  - Construct and return the `inference-response` record.
