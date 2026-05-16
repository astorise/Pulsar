# pulsar-cli (Node.js)

Node.js port of `pulsar-cli`. Starts a WebDAV workspace bridge, connects to the
Tachyon orchestrator over WebSocket, sandboxes agent edits in a git worktree,
and exposes a terminal REPL for the human bridge.

## Layout

| File                  | Responsibility                                   |
| --------------------- | ------------------------------------------------ |
| `src/protocol.ts`     | TypeScript discriminated unions for the orchestrator wire protocol (matches the Rust ABI). |
| `src/webdav.ts`       | Token-authenticated WebDAV server (GET/PUT/PROPFIND) over Node's `http` module. |
| `src/ws-client.ts`    | WebSocket client + receive-loop renderer.        |
| `src/git.ts`          | `simple-git` sandbox / worktree / patch apply.   |
| `src/repl.ts`         | Interactive REPL and yes/no confirmation prompt. |
| `src/skillify.ts`     | `.pulsar/skills` → msgpack registry compiler.    |
| `src/main.ts`         | Argument dispatch + lifecycle orchestration.     |

## Scripts

```bash
npm install
npm run typecheck   # tsc --noEmit
npm run build       # esbuild bundle → dist/main.cjs
npm test            # node --test on the suite under src/__tests__
```

The `dist/main.cjs` bundle is the artifact published by CI and can be `require`d
or spawned directly by the `pulsar-vscode` extension.

## Environment

| Variable                 | Default                                  |
| ------------------------ | ---------------------------------------- |
| `PULSAR_ORCHESTRATOR_WS` | `ws://127.0.0.1:8081/orchestrator`       |
| `PULSAR_WEBDAV_ADDR`     | `127.0.0.1:0` (port `0` picks a free one)|
| `WORMHOLE_RELAY`         | unset; when set, exposes WebDAV through Wormhole |
| `WORMHOLE_PUBLIC_PORT`   | local WebDAV port                        |
| `WORMHOLE_CA_CERT`       | unset                                    |
| `WORMHOLE_CLIENT_CERT`   | unset                                    |
| `WORMHOLE_CLIENT_KEY`    | unset                                    |
| `PULSAR_DEV_UNSECURE`    | unset; `1` skips Wormhole CA/auth options |
