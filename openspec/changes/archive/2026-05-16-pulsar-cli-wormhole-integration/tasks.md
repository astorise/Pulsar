# Implementation Tasks

- [x] **Phase 1: Dependency Integration**
  - [x] Add `@tachyon-mesh/wormhole` to the `pulsar-cli-node` `package.json` dependencies.

- [x] **Phase 2: Configuration & Environment**
  - [x] Extend `loadConfig()` in `src/main.ts` to parse Wormhole-specific environment variables (`WORMHOLE_RELAY`, `WORMHOLE_CA_CERT`, etc.).
  - [x] Define the logic for assigning or requesting the `publicPort` from the relay.

- [x] **Phase 3: Connection Pipeline (`src/main.ts`)**
  - [x] Await the WebDAV server spawn to retrieve its dynamically assigned local port.
  - [x] Call `Wormhole.create()` targeting the local WebDAV port.
  - [x] Format the public WebDAV URL based on the Wormhole endpoint and pass it to the `ws-client` init payload.

- [x] **Phase 4: Graceful Teardown**
  - [x] Ensure that `tunnel.close()` (or equivalent teardown mechanism) is invoked when the REPL closes or the `ClientMessage::Finish` action is triggered.
  - [x] Add unit/integration tests simulating a relay connection using the unsecure dev mode.
