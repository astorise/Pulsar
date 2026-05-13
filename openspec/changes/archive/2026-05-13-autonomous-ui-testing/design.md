# Design: Dual Engine CDP Architecture

## 1. The `tachyon:browser/cdp` WIT Interface
We introduce a new standard interface for WebAssembly FaaS to control a browser, regardless of where it runs.
```wit
package tachyon:browser@1.0.0;

interface cdp {
    enum engine-profile { local, smolvm }

    resource session {
        constructor(profile: engine-profile);
        navigate(url: string) -> result<_, string>;
        evaluate-js(script: string) -> result<string, string>;
        take-screenshot() -> result<list<u8>, string>; // Returns PNG/WebP bytes
    }
}
```

## 2. Host Implementation: Local Mode
When `constructor(local)` is called, `core-host` looks for an existing Chrome process with `--remote-debugging-port=9222`. It establishes a standard WebSocket bridge. This is highly efficient for local iterating on `localhost:3000`.

## 3. Host Implementation: SmolVM Mode
When `constructor(smolvm)` is called, `core-host` communicates via IPC with `system-faas-microvm-runner`.
1. A microVM is booted using `smolvm` with a pre-baked `ext4` image containing Chromium and `xvfb` (virtual framebuffer).
2. The VM is attached to a virtual TAP network interface with routing rules restricted *only* to the IP of the dev-server.
3. The `core-host` establishes a WebSocket connection to the CDP port forwarded from the microVM.
4. When the `session` resource is dropped by the Wasm Garbage Collector, the microVM receives a `SIGKILL`, ensuring zero state leakage between UI tests.

## 4. The Vision Agent Loop
The `faas/browser-agent` receives a `Screenshot` blob. It calls the `tachyon:ai/inference` interface (routing to a multimodal model) with the prompt: 
*"Analyze this UI screenshot. The user requested a centered blue button. Does it match the requirement? If not, output the CSS selectors that need fixing."*