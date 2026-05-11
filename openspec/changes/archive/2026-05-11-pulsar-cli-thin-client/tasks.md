# Implementation Tasks

- [x] **Task 1: Project Scaffolding**
  - Initialize a new Rust binary in `pulsar-cli/`.
  - Add dependencies: `tokio` (full), `hyper`, `dav-server` (or similar WebDAV crate), `tokio-tungstenite` (for WebSockets), `serde`, `serde_json`, and `rustyline`.

- [x] **Task 2: Implement the WebDAV Server**
  - Create `src/webdav.rs`.
  - Configure a `dav-server` instance serving the current working directory (`std::env::current_dir()`).
  - Add a simple Bearer token authentication layer to protect the local files.
  - Spawn the server asynchronously on a local port.

- [x] **Task 3: Implement the WebSocket Client**
  - Create `src/ws_client.rs`.
  - Implement the connection logic to the Tachyon Orchestrator endpoint.
  - Send the `init` JSON payload with the WebDAV coordinates immediately after connection.

- [x] **Task 4: The Terminal REPL**
  - Create `src/repl.rs`.
  - Use `rustyline` to capture user input.
  - Route the input as `user_message` JSON payloads to the WebSocket sender.

- [x] **Task 5: The Receive Loop**
  - Listen to the incoming WebSocket stream.
  - Parse `stream_token` messages and print them to `stdout` in real-time.
  - Handle `action_event` messages by printing them as system logs (e.g., `[Agent is modifying src/broker.rs]`).
