import { resolve } from "node:path";
import { basename } from "node:path";
import {
  applyPatchToRepo,
  cleanupSandbox,
  createSandbox,
  diffStat,
  mergeWorktree,
  repoRoot,
  Sandbox,
} from "./git.js";
import { promptYesNo, startRepl } from "./repl.js";
import * as skillify from "./skillify.js";
import { generateToken, spawn as spawnWebdav, WebdavHandle } from "./webdav.js";
import { buildPublicWebdavUrl, createWormhole, WormholeFactory, WormholeTunnel } from "./wormhole.js";
import { connect } from "./ws-client.js";

export interface CliConfig {
  orchestratorUrl: string;
  webdavHost: string;
  webdavPort: number;
  workspaceRoot: string;
  wormhole: WormholeConfig | null;
}

export interface WormholeConfig {
  relay: string;
  publicPort: number;
  ca?: string;
  clientCert?: string;
  clientKey?: string;
  unsecure: boolean;
}

function parseOptionalPort(value: string | undefined, name: string): number | undefined {
  if (value === undefined || value === "") return undefined;
  const port = Number.parseInt(value, 10);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error(`${name} must be a TCP port between 1 and 65535`);
  }
  return port;
}

export function parseWormholeConfig(env: NodeJS.ProcessEnv): WormholeConfig | null {
  const relay = env.WORMHOLE_RELAY;
  if (!relay) return null;

  const clientCert = env.WORMHOLE_CLIENT_CERT;
  const clientKey = env.WORMHOLE_CLIENT_KEY;
  if ((clientCert && !clientKey) || (!clientCert && clientKey)) {
    throw new Error("WORMHOLE_CLIENT_CERT and WORMHOLE_CLIENT_KEY must be set together");
  }

  return {
    relay,
    publicPort: parseOptionalPort(env.WORMHOLE_PUBLIC_PORT, "WORMHOLE_PUBLIC_PORT") ?? 0,
    ca: env.WORMHOLE_CA_CERT,
    clientCert,
    clientKey,
    unsecure: env.PULSAR_DEV_UNSECURE === "1" || env.WORMHOLE_DEV === "1",
  };
}

export function loadConfig(): CliConfig {
  const orchestratorUrl = process.env.PULSAR_ORCHESTRATOR_WS ?? "ws://127.0.0.1:8081/orchestrator";
  const rawAddr = process.env.PULSAR_WEBDAV_ADDR ?? "127.0.0.1:0";
  const [host, portStr] = rawAddr.split(":");
  const port = Number.parseInt(portStr ?? "0", 10);
  if (Number.isNaN(port)) {
    throw new Error(`PULSAR_WEBDAV_ADDR must be host:port (got ${rawAddr})`);
  }
  return {
    orchestratorUrl,
    webdavHost: host || "127.0.0.1",
    webdavPort: port,
    workspaceRoot: resolve(process.cwd()),
    wormhole: parseWormholeConfig(process.env),
  };
}

async function runSkillify(config: CliConfig): Promise<number> {
  const summary = await skillify.run(config.workspaceRoot);
  process.stdout.write(`${summary}\n`);
  return 0;
}

async function runMergeWorktree(config: CliConfig, branch: string | undefined): Promise<number> {
  if (!branch) {
    process.stderr.write("merge-worktree requires a branch name\n");
    return 2;
  }
  const root = await repoRoot(config.workspaceRoot);
  if (!root) {
    process.stderr.write("merge-worktree must be run inside a git repository\n");
    return 2;
  }
  const conflicts = await mergeWorktree(root, branch);
  if (conflicts.length === 0) {
    process.stdout.write("Merge completed without conflicts.\n");
  } else {
    process.stdout.write(`${conflicts.join("\n")}\n`);
  }
  return 0;
}

async function handleSandboxFinish(sandbox: Sandbox): Promise<void> {
  process.stdout.write("\nPulsar session finished. Sandbox diff:\n");
  const diff = await diffStat(sandbox);
  if (diff.length === 0) {
    process.stdout.write("No changes to apply.\n");
    await cleanupSandbox(sandbox);
    return;
  }
  process.stdout.write(`${diff}\n`);
  if (await promptYesNo("Apply these changes? [y/N] ")) {
    await applyPatchToRepo(sandbox);
    process.stdout.write("Changes applied to the active worktree index.\n");
  } else {
    process.stdout.write("Changes discarded.\n");
  }
  await cleanupSandbox(sandbox);
}

export async function openWormhole(
  config: WormholeConfig,
  localPort: number,
  factory: WormholeFactory = createWormhole,
): Promise<{ tunnel: WormholeTunnel; workspaceUrl: string }> {
  const publicPort = config.publicPort === 0 ? localPort : config.publicPort;
  const tunnel = await factory({
    relay: config.relay,
    targets: [{ protocol: "tcp", publicPort, localPort }],
    ca: config.unsecure ? undefined : config.ca,
    auth: config.unsecure || !config.clientCert || !config.clientKey
      ? undefined
      : { cert: config.clientCert, key: config.clientKey },
    unsecure: config.unsecure,
  });
  return {
    tunnel,
    workspaceUrl: buildPublicWebdavUrl(config.relay, publicPort),
  };
}

export async function runInteractive(
  config: CliConfig,
  wormholeFactory: WormholeFactory = createWormhole,
): Promise<number> {
  const token = generateToken();
  const sandbox = await createSandbox(config.workspaceRoot, token);
  const webdavRoot = sandbox?.worktreePath ?? config.workspaceRoot;

  let webdav: WebdavHandle | null = null;
  let tunnel: WormholeTunnel | null = null;
  try {
    webdav = await spawnWebdav({
      root: webdavRoot,
      host: config.webdavHost,
      port: config.webdavPort,
      token,
    });
    let workspaceUrl = webdav.url;
    if (config.wormhole) {
      const opened = await openWormhole(config.wormhole, webdav.localPort, wormholeFactory);
      tunnel = opened.tunnel;
      workspaceUrl = opened.workspaceUrl;
    }

    let finishTriggered = false;
    const ws = connect({
      endpoint: config.orchestratorUrl,
      init: { type: "init", workspace_url: workspaceUrl, workspace_token: token },
      onFinish: () => {
        finishTriggered = true;
      },
    });
    const repl = startRepl(ws.send);

    await Promise.race([ws.done, repl.done]);
    repl.close();
    await ws.close();

    if (sandbox && finishTriggered) {
      await handleSandboxFinish(sandbox);
    } else if (sandbox) {
      await cleanupSandbox(sandbox);
    }
    return 0;
  } finally {
    if (tunnel) await tunnel.close();
    if (webdav) await webdav.close();
  }
}

export async function main(argv: string[]): Promise<number> {
  const config = loadConfig();
  const subcommand = argv[2];
  if (subcommand === "skillify") return runSkillify(config);
  if (subcommand === "merge-worktree") return runMergeWorktree(config, argv[3]);
  return runInteractive(config);
}

const entrypoint = process.argv[1] ? basename(process.argv[1]) : "";
if (entrypoint === "pulsar-cli" || /^main\.(c?js|ts)$/.test(entrypoint)) {
  main(process.argv)
    .then((code) => process.exit(code))
    .catch((err) => {
      process.stderr.write(`pulsar-cli: ${(err as Error).message}\n`);
      process.exit(1);
    });
}
