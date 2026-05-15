import { test } from "node:test";
import { strict as assert } from "node:assert";
import { tmpdir } from "node:os";
import { existsSync, mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { stat } from "node:fs/promises";
import { join, sep } from "node:path";
import {
  buildPropfindXml,
  generateToken,
  parseDestinationHeader,
  resolveWorkspacePath,
  spawn,
} from "../webdav.js";

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

test("propfind xml emits resourcetype, size and modified date", () => {
  const xml = buildPropfindXml([
    { href: "/webdav/src/", isDirectory: true, size: 0, modifiedAt: new Date("2026-05-12T10:00:00Z") },
    { href: "/webdav/README.md", isDirectory: false, size: 42, modifiedAt: new Date("2026-05-12T10:00:00Z") },
  ]);
  assert.ok(xml.includes("<d:resourcetype><d:collection/></d:resourcetype>"));
  assert.ok(xml.includes("<d:resourcetype/>"));
  assert.ok(xml.includes("<d:getcontentlength>42</d:getcontentlength>"));
  assert.ok(xml.includes("<d:getlastmodified>"));
});

test("token is generated with the secret_ prefix", () => {
  assert.ok(generateToken().startsWith("secret_"));
});

test("destination header is parsed into a workspace-relative path", () => {
  assert.equal(parseDestinationHeader("http://host:8080/webdav/src/new.ts"), "src/new.ts");
  assert.equal(parseDestinationHeader("/webdav/src/new.ts"), "src/new.ts");
  assert.equal(parseDestinationHeader("http://host/other"), null);
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

test("PROPFIND distinguishes files from directories and honors Depth 0/1", async () => {
  const root = mkdtempSync(join(tmpdir(), "webdav-"));
  mkdirSync(join(root, "src"), { recursive: true });
  writeFileSync(join(root, "README.md"), "# Pulsar");
  const token = "tok-listing";

  const handle = await spawn({ root, host: "127.0.0.1", port: 0, token });
  try {
    const depthDefault = await fetch(`${handle.url}/`, {
      method: "PROPFIND",
      headers: { Authorization: `Bearer ${token}` },
    });
    assert.equal(depthDefault.status, 207);
    const body = await depthDefault.text();
    assert.ok(body.includes("<d:href>/webdav/src/</d:href>"));
    assert.ok(body.match(/<d:href>\/webdav\/src\/<\/d:href>[\s\S]*?<d:resourcetype><d:collection\/><\/d:resourcetype>/));
    assert.ok(body.match(/<d:href>\/webdav\/README\.md<\/d:href>[\s\S]*?<d:resourcetype\/>/));

    const depthZero = await fetch(`${handle.url}/`, {
      method: "PROPFIND",
      headers: { Authorization: `Bearer ${token}`, Depth: "0" },
    });
    const bodyZero = await depthZero.text();
    assert.equal(depthZero.status, 207);
    assert.ok(bodyZero.includes("<d:href>/webdav/</d:href>"));
    assert.ok(!bodyZero.includes("README.md"));

    const depthInfinity = await fetch(`${handle.url}/`, {
      method: "PROPFIND",
      headers: { Authorization: `Bearer ${token}`, Depth: "infinity" },
    });
    assert.equal(depthInfinity.status, 403);
  } finally {
    await handle.close();
  }
});

test("MKCOL creates directories and refuses to clobber existing ones", async () => {
  const root = mkdtempSync(join(tmpdir(), "webdav-"));
  const token = "tok-mkcol";

  const handle = await spawn({ root, host: "127.0.0.1", port: 0, token });
  try {
    const created = await fetch(`${handle.url}/src/components/wizard`, {
      method: "MKCOL",
      headers: { Authorization: `Bearer ${token}` },
    });
    assert.equal(created.status, 201);
    const created_stat = await stat(join(root, "src", "components", "wizard"));
    assert.ok(created_stat.isDirectory());

    const conflict = await fetch(`${handle.url}/src/components/wizard`, {
      method: "MKCOL",
      headers: { Authorization: `Bearer ${token}` },
    });
    assert.equal(conflict.status, 405);
  } finally {
    await handle.close();
  }
});

test("DELETE removes files and directories recursively", async () => {
  const root = mkdtempSync(join(tmpdir(), "webdav-"));
  mkdirSync(join(root, "trash", "nested"), { recursive: true });
  writeFileSync(join(root, "trash", "nested", "file.txt"), "bye");
  const token = "tok-delete";

  const handle = await spawn({ root, host: "127.0.0.1", port: 0, token });
  try {
    const ok = await fetch(`${handle.url}/trash`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${token}` },
    });
    assert.equal(ok.status, 204);
    assert.equal(existsSync(join(root, "trash")), false);

    const missing = await fetch(`${handle.url}/trash`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${token}` },
    });
    assert.equal(missing.status, 404);
  } finally {
    await handle.close();
  }
});

test("MOVE renames within the workspace and rejects escapes", async () => {
  const root = mkdtempSync(join(tmpdir(), "webdav-"));
  mkdirSync(join(root, "src"), { recursive: true });
  writeFileSync(join(root, "src", "old.ts"), "export const x = 1;");
  const token = "tok-move";

  const handle = await spawn({ root, host: "127.0.0.1", port: 0, token });
  try {
    const ok = await fetch(`${handle.url}/src/old.ts`, {
      method: "MOVE",
      headers: {
        Authorization: `Bearer ${token}`,
        Destination: `${handle.url}/src/new.ts`,
      },
    });
    assert.equal(ok.status, 201);
    assert.equal(existsSync(join(root, "src", "old.ts")), false);
    assert.equal(existsSync(join(root, "src", "new.ts")), true);

    const escape = await fetch(`${handle.url}/src/new.ts`, {
      method: "MOVE",
      headers: {
        Authorization: `Bearer ${token}`,
        Destination: "http://attacker.invalid/other/x",
      },
    });
    assert.equal(escape.status, 400);
  } finally {
    await handle.close();
  }
});
