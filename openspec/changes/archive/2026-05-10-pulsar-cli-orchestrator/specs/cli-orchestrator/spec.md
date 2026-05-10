# Specification: Pulsar CLI Core

## Configuration
The CLI expects a `pulsar.toml` in the user's home directory or workspace, defining the Tachyon cluster endpoints:
```toml
[backend]
tachyon_endpoint = "http://localhost:443" # Forwarded from WSL2/k3s
tier1_inference = "local" # Uses unified memory / eGPU
tier2_inference = "grpc://talos-node:443"
```

## Internal Modules
- `main.rs`: Entry point, loads config, starts Tokio runtime.
- `repl.rs`: Terminal input handling.
- `orchestrator.rs`: The state machine for the AI loop.
- `mesh_client.rs`: The gRPC wrapper to call `tachyon:ai/viking-context` and inference nodes.