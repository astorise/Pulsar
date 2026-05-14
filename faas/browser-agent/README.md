# browser-agent

`browser-agent` is the Tachyon-Mesh browser automation component. It owns browser-facing actions and keeps those interactions isolated from the orchestration loop.

The crate builds as both an `rlib` for native tests and a `cdylib` for WebAssembly component packaging.
