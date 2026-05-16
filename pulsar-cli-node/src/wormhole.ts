export interface WormholeTarget {
  protocol: "tcp" | "udp";
  publicPort: number;
  localPort: number;
}

export interface WormholeOptions {
  relay: string;
  targets: WormholeTarget[];
  ca?: string;
  auth?: {
    cert: string;
    key: string;
  };
  unsecure?: boolean;
}

export interface WormholeTunnel {
  endpoint?: string;
  close: () => void | Promise<void>;
}

export type WormholeFactory = (opts: WormholeOptions) => Promise<WormholeTunnel>;

interface WormholeModule {
  Wormhole: {
    create: WormholeFactory;
  };
}

export async function createWormhole(opts: WormholeOptions): Promise<WormholeTunnel> {
  const mod = await import("@tachyon-mesh/wormhole") as WormholeModule;
  return mod.Wormhole.create(opts);
}

export function parseRelayHost(relay: string): string {
  const parsed = relay.trim();
  if (!parsed) {
    throw new Error("WORMHOLE_RELAY cannot be empty");
  }
  return parsed.split(":")[0] || parsed;
}

export function buildPublicWebdavUrl(relay: string, publicPort: number): string {
  return `http://${parseRelayHost(relay)}:${publicPort}/webdav`;
}
