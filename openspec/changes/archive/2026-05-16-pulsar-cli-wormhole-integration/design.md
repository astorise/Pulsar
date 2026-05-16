# Design: Wormhole Tunnel Integration

## Architecture & Flow

1. **Local Initialization:** The `pulsar-cli` spawns the WebDAV server on an ephemeral local port (e.g., `127.0.0.1:0`) using the existing `spawnWebdav` logic.

2. **Tunnel Creation:**
   The CLI instantiates `Wormhole.create()` using the `@tachyon-mesh/wormhole` module. It maps the local WebDAV port to a requested public TCP port on the relay.
   
   ```javascript
   const tunnel = await Wormhole.create({
     relay: config.relayEndpoint, // e.g., 'relay.tachyon.io:4433'
     targets: [
       { protocol: 'tcp', publicPort: requestedPublicPort, localPort: webdavLocalPort }
     ],
     ca: config.relayCa,
     auth: { cert: config.clientCert, key: config.clientKey },
   });
   ```

3. **Orchestrator Handshake:**
   The `workspace_url` sent in the WebSocket `ClientMessage::Init` payload will no longer be `http://127.0.0.1:PORT/webdav`. Instead, it will be the public URL exposed by the relay (e.g., `http://<relay_ip>:<publicPort>/webdav`).

4. **Development Mode:**
   To facilitate local testing of the entire stack without mTLS, the CLI will support skipping the `ca` and `auth` parameters if a specific environment variable (e.g., `PULSAR_DEV_UNSECURE=1`) is set, aligning with Wormhole's `WORMHOLE_DEV=1` behavior.

## Configuration Updates
The `CliConfig` interface must be extended to include:
* `WORMHOLE_RELAY`: The target relay address (e.g., `relay.tachyon.io:4433`).
* `WORMHOLE_CA_CERT`: Path to the Relay CA for verification.
* `WORMHOLE_CLIENT_CERT` / `WORMHOLE_CLIENT_KEY`: Paths to the developer's identity certificates.