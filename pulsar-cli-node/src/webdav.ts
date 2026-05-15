import { createServer, IncomingMessage, ServerResponse, Server } from "node:http";
import { promises as fs } from "node:fs";
import { dirname, normalize, resolve, sep } from "node:path";
import { randomBytes } from "node:crypto";

export interface WebdavHandle {
  url: string;
  close: () => Promise<void>;
}

export interface WebdavOptions {
  root: string;
  host?: string;
  port?: number;
  token: string;
}

export function generateToken(): string {
  return `secret_${process.pid.toString(16)}_${randomBytes(8).toString("hex")}`;
}

export function resolveWorkspacePath(root: string, requestPath: string): string | null {
  const trimmed = requestPath.replace(/^\/+/, "");
  if (!trimmed) return resolve(root);
  if (trimmed.split(/[\\/]/).some((part) => part === "..")) return null;
  const candidate = resolve(root, normalize(trimmed));
  const rootResolved = resolve(root);
  if (candidate !== rootResolved && !candidate.startsWith(rootResolved + sep)) return null;
  return candidate;
}

function escapeXml(input: string): string {
  return input
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function buildPropfindXml(entries: string[]): string {
  const responses = entries
    .map(
      (entry) =>
        `<d:response><d:href>${escapeXml(entry)}</d:href><d:propstat><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>`,
    )
    .join("");
  return `<?xml version="1.0" encoding="utf-8"?><d:multistatus xmlns:d="DAV:">${responses}</d:multistatus>`;
}

function authorize(req: IncomingMessage, token: string): boolean {
  const header = req.headers["authorization"];
  if (typeof header !== "string") return false;
  return header === `Bearer ${token}`;
}

async function readBody(req: IncomingMessage): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(chunk as Buffer);
  }
  return Buffer.concat(chunks);
}

function urlPath(req: IncomingMessage): string {
  const url = req.url ?? "/";
  const prefix = "/webdav";
  if (url === prefix || url === `${prefix}/`) return "";
  if (url.startsWith(`${prefix}/`)) return url.slice(prefix.length + 1);
  return "";
}

async function handleRequest(req: IncomingMessage, res: ServerResponse, opts: WebdavOptions): Promise<void> {
  if (!req.url?.startsWith("/webdav")) {
    res.statusCode = 404;
    res.end();
    return;
  }
  if (!authorize(req, opts.token)) {
    res.statusCode = req.headers["authorization"] ? 403 : 401;
    res.end();
    return;
  }
  const relPath = urlPath(req);
  const target = resolveWorkspacePath(opts.root, relPath);
  if (target === null) {
    res.statusCode = 400;
    res.end();
    return;
  }

  const method = (req.method ?? "GET").toUpperCase();
  try {
    if (method === "GET") {
      const data = await fs.readFile(target);
      res.statusCode = 200;
      res.end(data);
      return;
    }
    if (method === "PUT") {
      const body = await readBody(req);
      await fs.mkdir(dirname(target), { recursive: true });
      await fs.writeFile(target, body);
      res.statusCode = 204;
      res.end();
      return;
    }
    if (method === "PROPFIND") {
      const stat = await fs.stat(target);
      if (!stat.isDirectory()) {
        res.statusCode = 404;
        res.end();
        return;
      }
      const entries = (await fs.readdir(target)).sort();
      res.statusCode = 207;
      res.setHeader("Content-Type", "application/xml; charset=utf-8");
      res.end(buildPropfindXml(entries));
      return;
    }
    res.statusCode = 405;
    res.end();
  } catch (err) {
    const code = (err as NodeJS.ErrnoException).code;
    if (code === "ENOENT" || code === "ENOTDIR") {
      res.statusCode = 404;
    } else {
      res.statusCode = 500;
    }
    res.end();
  }
}

export async function spawn(opts: WebdavOptions): Promise<WebdavHandle> {
  const canonicalRoot = resolve(opts.root);
  await fs.access(canonicalRoot);
  const options = { ...opts, root: canonicalRoot };

  const server: Server = createServer((req, res) => {
    handleRequest(req, res, options).catch(() => {
      if (!res.headersSent) {
        res.statusCode = 500;
      }
      res.end();
    });
  });

  await new Promise<void>((resolveListen, rejectListen) => {
    server.once("error", rejectListen);
    server.listen(opts.port ?? 0, opts.host ?? "127.0.0.1", () => resolveListen());
  });

  const address = server.address();
  if (!address || typeof address === "string") {
    throw new Error("failed to determine webdav listener address");
  }
  const host = options.host ?? "127.0.0.1";
  const url = `http://${host}:${address.port}/webdav`;

  return {
    url,
    close: () =>
      new Promise<void>((resolveClose, rejectClose) => {
        server.close((err) => (err ? rejectClose(err) : resolveClose()));
      }),
  };
}
