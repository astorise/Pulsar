import { test } from "node:test";
import { strict as assert } from "node:assert";
import { encodeClientMessage, decodeServerMessage, isFinishAction } from "../protocol.js";

test("init message uses expected JSON shape", () => {
  const json = encodeClientMessage({
    type: "init",
    workspace_url: "http://127.0.0.1:9000/webdav",
    workspace_token: "secret",
  });
  assert.ok(json.includes('"type":"init"'));
  assert.ok(json.includes('"workspace_url":"http://127.0.0.1:9000/webdav"'));
  assert.ok(json.includes('"workspace_token":"secret"'));
});

test("server stream_token decodes", () => {
  const msg = decodeServerMessage('{"type":"stream_token","content":"hello"}');
  assert.deepEqual(msg, { type: "stream_token", content: "hello" });
});

test("lsp hover messages round-trip", () => {
  const response = encodeClientMessage({ type: "lsp_hover_response", id: "hover-1", markdown: "fn run()" });
  assert.ok(response.includes('"type":"lsp_hover_response"'));

  const request = decodeServerMessage(
    '{"type":"lsp_hover_request","id":"hover-1","path":"src/lib.rs","line":3,"character":4}',
  );
  assert.equal(request.type, "lsp_hover_request");
});

test("finish action is detected", () => {
  assert.equal(isFinishAction('{"type":"action_event","action":"finish","target":"session"}'), true);
  assert.equal(isFinishAction('{"type":"action_event","action":"write_file","target":"src/lib.rs"}'), false);
});
