import { test } from "node:test";
import { strict as assert } from "node:assert";
import { openWormhole, parseWormholeConfig } from "../main.js";
import { buildPublicWebdavUrl, WormholeOptions } from "../wormhole.js";

test("wormhole config is disabled without a relay", () => {
  assert.equal(parseWormholeConfig({}), null);
});

test("wormhole config parses relay, credentials, public port and dev mode", () => {
  const config = parseWormholeConfig({
    WORMHOLE_RELAY: "relay.tachyon.io:4433",
    WORMHOLE_PUBLIC_PORT: "18443",
    WORMHOLE_CA_CERT: "/certs/ca.pem",
    WORMHOLE_CLIENT_CERT: "/certs/client.pem",
    WORMHOLE_CLIENT_KEY: "/certs/client.key",
    PULSAR_DEV_UNSECURE: "1",
  });

  assert.deepEqual(config, {
    relay: "relay.tachyon.io:4433",
    publicPort: 18443,
    ca: "/certs/ca.pem",
    clientCert: "/certs/client.pem",
    clientKey: "/certs/client.key",
    unsecure: true,
  });
});

test("wormhole config requires cert and key together", () => {
  assert.throws(
    () => parseWormholeConfig({ WORMHOLE_RELAY: "relay:4433", WORMHOLE_CLIENT_CERT: "/cert.pem" }),
    /WORMHOLE_CLIENT_CERT and WORMHOLE_CLIENT_KEY/,
  );
});

test("public WebDAV URL uses the relay host and requested port", () => {
  assert.equal(buildPublicWebdavUrl("relay.tachyon.io:4433", 18443), "http://relay.tachyon.io:18443/webdav");
});

test("openWormhole binds local WebDAV port and returns public relay URL", async () => {
  let captured: WormholeOptions | null = null;
  const opened = await openWormhole(
    {
      relay: "relay.tachyon.io:4433",
      publicPort: 18443,
      ca: "/certs/ca.pem",
      clientCert: "/certs/client.pem",
      clientKey: "/certs/client.key",
      unsecure: false,
    },
    49152,
    async (opts) => {
      captured = opts;
      return { close: () => undefined };
    },
  );

  assert.equal(opened.workspaceUrl, "http://relay.tachyon.io:18443/webdav");
  assert.deepEqual(captured, {
    relay: "relay.tachyon.io:4433",
    targets: [{ protocol: "tcp", publicPort: 18443, localPort: 49152 }],
    ca: "/certs/ca.pem",
    auth: { cert: "/certs/client.pem", key: "/certs/client.key" },
    unsecure: false,
  });
});

test("openWormhole defaults requested public port to the local WebDAV port", async () => {
  const opened = await openWormhole(
    { relay: "relay.local:4433", publicPort: 0, unsecure: true },
    49153,
    async (opts) => {
      assert.deepEqual(opts.targets, [{ protocol: "tcp", publicPort: 49153, localPort: 49153 }]);
      assert.equal(opts.ca, undefined);
      assert.equal(opts.auth, undefined);
      return { close: () => undefined };
    },
  );

  assert.equal(opened.workspaceUrl, "http://relay.local:49153/webdav");
});
