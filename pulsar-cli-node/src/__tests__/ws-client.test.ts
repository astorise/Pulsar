import { test } from "node:test";
import { strict as assert } from "node:assert";
import { handleWsText } from "../ws-client.js";

test("stream_token is printable content", () => {
  assert.equal(handleWsText('{"type":"stream_token","content":"abc"}'), "abc");
});

test("action_event is rendered as system log", () => {
  assert.equal(
    handleWsText('{"type":"action_event","action":"write_file","target":"src/lib.rs"}'),
    "[Agent write_file: src/lib.rs]",
  );
});

test("escalated message is high visibility", () => {
  const text = handleWsText('{"type":"escalated","report":"# Situation"}');
  assert.ok(text.includes("RABBIT HOLE DETECTED"));
  assert.ok(text.includes("# Situation"));
});
