import { test } from "node:test";
import { strict as assert } from "node:assert";
import { sanitizeSessionId } from "../git.js";

test("session id is git-branch safe", () => {
  assert.equal(sanitizeSessionId("secret_123 ABC/../x"), "secret-123-abc-x");
});

test("empty session id has fallback", () => {
  assert.equal(sanitizeSessionId("!!!"), "session");
});
