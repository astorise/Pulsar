import { test } from "node:test";
import { strict as assert } from "node:assert";
import { pack, unpack } from "msgpackr";
import { compileSkill, splitFrontmatter } from "../skillify.js";

test("frontmatter is split from body", () => {
  const { frontmatter, body } = splitFrontmatter(
    "---\nname: Rust Fixer\nallowed_tools: search, edit\n---\n# Body",
  );
  assert.equal(frontmatter.name, "Rust Fixer");
  assert.equal(body.trim(), "# Body");
});

test("skill compiles to msgpack-safe struct", () => {
  const skill = compileSkill(
    "SKILL.md",
    "---\nname: Rust Fixer\ndescription: Fix Rust tests\nallowed_tools: search, edit\n---\n# Prompt",
  );
  const encoded = pack(skill);
  const decoded = unpack(encoded) as typeof skill;
  assert.equal(skill.name, "Rust Fixer");
  assert.deepEqual(skill.allowed_tools, ["search", "edit"]);
  assert.equal(decoded.name, "Rust Fixer");
  assert.ok(encoded.length > 0);
});
