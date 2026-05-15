import { test } from "node:test";
import { strict as assert } from "node:assert";
import { tmpdir } from "node:os";
import { mkdtempSync, writeFileSync, mkdirSync } from "node:fs";
import { join, sep } from "node:path";
import { buildPropfindXml, resolveWorkspacePath, generateToken, spawn } from "../webdav.js";

test("rejects parent directory traversal", () => {
  const root = mkdtempSync(join(tmpdir(), "webdav-"));
  assert.equal(resolveWorkspacePath(root, "../secret"), null);
});

test("joins safe workspace path", () => {
  const root = mkdtempSync(join(tmpdir(), "webdav-"));
  const resolved = resolveWorkspacePath(root, "src/main.rs");
  assert.ok(resolved !== null);
  assert.ok(resolved!.endsWith(`src${sep}main.rs`));
});

test("propfind xml escapes entries", () => {
  const xml = buildPropfindXml(["a&b.rs"]);
  assert.ok(xml.includes("a&amp;b.rs"));
  assert.ok(xml.includes("multistatus"));
});

test("token is generated with the secret_ prefix", () => {
  assert.ok(generateToken().startsWith("secret_"));
});

test("webdav GET requires bearer token and returns file contents", async () => {
  const root = mkdtempSync(join(tmpdir(), "webdav-"));
  mkdirSync(join(root, "src"), { recursive: true });
  writeFileSync(join(root, "src", "main.rs"), "fn main() {}");
  const token = "tok-123";

  const handle = await spawn({ root, host: "127.0.0.1", port: 0, token });
  try {
    const unauthorized = await fetch(`${handle.url}/src/main.rs`);
    assert.equal(unauthorized.status, 401);

    const authorized = await fetch(`${handle.url}/src/main.rs`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    assert.equal(authorized.status, 200);
    assert.equal(await authorized.text(), "fn main() {}");
  } finally {
    await handle.close();
  }
});
