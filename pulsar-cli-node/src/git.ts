import { promises as fs } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { simpleGit, SimpleGit } from "simple-git";

export interface Sandbox {
  repoRoot: string;
  worktreePath: string;
  branch: string;
}

export function sanitizeSessionId(input: string): string {
  let out = "";
  for (const ch of input) {
    if (/[A-Za-z0-9]/.test(ch)) {
      out += ch.toLowerCase();
    } else if (out.length > 0 && !out.endsWith("-")) {
      out += "-";
    }
  }
  const trimmed = out.replace(/^-+|-+$/g, "");
  return trimmed.length === 0 ? "session" : trimmed.slice(0, 48);
}

export async function repoRoot(cwd: string): Promise<string | null> {
  try {
    const git = simpleGit({ baseDir: cwd });
    const top = (await git.revparse(["--show-toplevel"])).trim();
    return top.length > 0 ? top : null;
  } catch {
    return null;
  }
}

export async function ensureLocalExclude(repoPath: string): Promise<void> {
  const exclude = join(repoPath, ".git", "info", "exclude");
  let existing = "";
  try {
    existing = await fs.readFile(exclude, "utf8");
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== "ENOENT") throw err;
  }
  if (existing.split(/\r?\n/).some((line) => line.trim() === ".pulsar/")) return;

  await fs.mkdir(dirname(exclude), { recursive: true });
  let next = existing;
  if (next.length > 0 && !next.endsWith("\n")) next += "\n";
  next += ".pulsar/\n";
  await fs.writeFile(exclude, next);
}

export async function createSandbox(cwd: string, sessionId: string): Promise<Sandbox | null> {
  const root = await repoRoot(cwd);
  if (!root) return null;

  await ensureLocalExclude(root);

  const safeId = sanitizeSessionId(sessionId);
  const branch = `pulsar/sess-${safeId}`;
  const worktreePath = join(root, ".pulsar", "worktrees", safeId);
  await fs.mkdir(dirname(worktreePath), { recursive: true });

  const git = simpleGit({ baseDir: root });
  await git.raw(["worktree", "add", "-b", branch, worktreePath, "HEAD"]);

  return { repoRoot: root, worktreePath, branch };
}

export async function diffStat(sandbox: Sandbox): Promise<string> {
  const git = simpleGit({ baseDir: sandbox.worktreePath });
  return (await git.raw(["diff", "--stat", "HEAD"])).trim();
}

export async function applyPatchToRepo(sandbox: Sandbox): Promise<void> {
  const sandboxGit = simpleGit({ baseDir: sandbox.worktreePath });
  const patch = await sandboxGit.raw(["diff", "--binary", "HEAD"]);
  if (!patch.trim()) return;

  const { spawn } = await import("node:child_process");
  await new Promise<void>((resolveApply, rejectApply) => {
    const child = spawn("git", ["-C", sandbox.repoRoot, "apply", "--index"], {
      stdio: ["pipe", "inherit", "pipe"],
    });
    let stderr = "";
    child.stderr?.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("error", rejectApply);
    child.on("close", (code) => {
      if (code === 0) resolveApply();
      else rejectApply(new Error(stderr.trim() || `git apply exited with code ${code}`));
    });
    child.stdin!.write(patch);
    child.stdin!.end();
  });
}

export async function cleanupSandbox(sandbox: Sandbox): Promise<void> {
  const git = simpleGit({ baseDir: sandbox.repoRoot });
  const errors: unknown[] = [];
  try {
    await git.raw(["worktree", "remove", "--force", sandbox.worktreePath]);
  } catch (err) {
    errors.push(err);
  }
  try {
    await git.raw(["branch", "-D", sandbox.branch]);
  } catch (err) {
    errors.push(err);
  }
  if (errors.length === 2) throw errors[0];
}

export async function mergeWorktree(repoPath: string, branch: string): Promise<string[]> {
  const git: SimpleGit = simpleGit({ baseDir: repoPath });
  try {
    await git.raw(["merge", branch, "--no-commit", "--no-ff"]);
  } catch {
    return conflictedFiles(repoPath);
  }
  return conflictedFiles(repoPath);
}

export async function conflictedFiles(repoPath: string): Promise<string[]> {
  const git = simpleGit({ baseDir: repoPath });
  const output = await git.raw(["diff", "--name-only", "--diff-filter=U"]);
  return output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

export function resolveWorkspaceRoot(): string {
  return resolve(process.cwd());
}
