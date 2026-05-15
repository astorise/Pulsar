import { promises as fs } from "node:fs";
import { basename, extname, join } from "node:path";
import { pack } from "msgpackr";

export interface SkillDef {
  name: string;
  description_embedding: number[];
  system_prompt: string;
  allowed_tools: string[];
}

export function splitFrontmatter(content: string): { frontmatter: Record<string, string>; body: string } {
  const frontmatter: Record<string, string> = {};
  if (!content.startsWith("---\n")) return { frontmatter, body: content };
  const rest = content.slice(4);
  const closingIndex = rest.indexOf("\n---\n");
  if (closingIndex < 0) return { frontmatter, body: content };
  const block = rest.slice(0, closingIndex);
  const body = rest.slice(closingIndex + 5);
  for (const line of block.split(/\r?\n/)) {
    const idx = line.indexOf(":");
    if (idx < 0) continue;
    const key = line.slice(0, idx).trim();
    let value = line.slice(idx + 1).trim();
    if (value.startsWith('"') && value.endsWith('"')) value = value.slice(1, -1);
    frontmatter[key] = value;
  }
  return { frontmatter, body };
}

export function firstHeading(body: string): string | null {
  for (const line of body.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed.startsWith("# ")) return trimmed.slice(2);
  }
  return null;
}

export function embedDescription(description: string): number[] {
  const vector = new Array<number>(16).fill(0);
  for (const token of description.split(/[^A-Za-z0-9]+/).filter(Boolean)) {
    let hash = 2166136261 >>> 0;
    for (const char of token.toLowerCase()) {
      hash ^= char.charCodeAt(0);
      hash = Math.imul(hash, 16777619) >>> 0;
    }
    vector[hash % vector.length] += 1;
  }
  const norm = Math.sqrt(vector.reduce((sum, v) => sum + v * v, 0));
  if (norm > 0) {
    for (let i = 0; i < vector.length; i += 1) vector[i] /= norm;
  }
  return vector;
}

export function compileSkill(filePath: string, content: string): SkillDef {
  const { frontmatter, body } = splitFrontmatter(content);
  const fallbackName = basename(filePath, extname(filePath));
  const name = frontmatter.name ?? fallbackName;
  const description = frontmatter.description ?? firstHeading(body) ?? name;
  const allowedTools = frontmatter.allowed_tools
    ? frontmatter.allowed_tools
        .split(",")
        .map((tool) => tool.trim())
        .filter((tool) => tool.length > 0)
    : [];
  return {
    name,
    description_embedding: embedDescription(description),
    system_prompt: body.trim(),
    allowed_tools: allowedTools,
  };
}

async function visitSkillDir(dir: string, out: string[]): Promise<void> {
  let entries: import("node:fs").Dirent[];
  try {
    entries = await fs.readdir(dir, { withFileTypes: true });
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === "ENOENT") return;
    throw err;
  }
  for (const entry of entries) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) await visitSkillDir(full, out);
    else if (entry.isFile() && entry.name.endsWith(".md")) out.push(full);
  }
}

export async function discoverSkillFiles(root: string): Promise<string[]> {
  const out: string[] = [];
  await visitSkillDir(root, out);
  out.sort();
  return out;
}

export async function run(workspaceRoot: string): Promise<string> {
  const skillsDir = join(workspaceRoot, ".pulsar", "skills");
  const files = await discoverSkillFiles(skillsDir);
  const compiled: SkillDef[] = [];
  for (const file of files) {
    const content = await fs.readFile(file, "utf8");
    compiled.push(compileSkill(file, content));
  }
  const registry = join(workspaceRoot, ".pulsar", "skill-registry", "pulsar_skill_registry.msgpack");
  await fs.mkdir(join(workspaceRoot, ".pulsar", "skill-registry"), { recursive: true });
  await fs.writeFile(registry, pack(compiled));
  return `Compiled ${compiled.length} skills into ${registry}`;
}
