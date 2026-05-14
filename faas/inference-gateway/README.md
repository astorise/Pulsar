# inference-gateway

`inference-gateway` normalizes model generation requests for Pulsar agents. It estimates usage, forwards prompts to the Tachyon-Mesh inference host, and returns structured generation responses.

The crate builds as both an `rlib` for native tests and a `cdylib` for WebAssembly component packaging.
