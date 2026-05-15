import { createServer, IncomingMessage, ServerResponse, Server } from "node:http";
import { promises as fs, Stats } from "node:fs";
import { dirname, join, normalize, posix, resolve, sep } from "node:path";
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

export interface PropfindEntry {
  href: string;
  isDirectory: boolean;
  size: number;
  modifiedAt: Date;
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

function renderPropfindEntry(entry: PropfindEntry): string {
  const resourceType = entry.isDirectory
    ? "<d:resourcetype><d:collection/></d:resourcetype>"
    : "<d:resourcetype/>";
  return (
    "<d:response>" +
    `<d:href>${escapeXml(entry.href)}</d:href>` +
    "<d:propstat>" +
    "<d:prop>" +
    resourceType +
    `<d:getcontentlength>${entry.size}</d:getcontentlength>` +
    `<d:getlastmodified>${entry.modifiedAt.toUTCString()}</d:getlastmodified>` +
    "</d:prop>" +
    "<d:status>HTTP/1.1 200 OK</d:status>" +
    "</d:propstat>" +
    "</d:response>"
  );
}

export function buildPropfindXml(entries: PropfindEntry[]): string {
  const responses = entries.map(renderPropfindEntry).join("");
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

const WEBDAV_PREFIX = "/webdav";

function urlPath(req: IncomingMessage): string {
  const url = req.url ?? "/";
  if (url === WEBDAV_PREFIX || url === `${WEBDAV_PREFIX}/`) return "";
  if (url.startsWith(`${WEBDAV_PREFIX}/`)) return url.slice(WEBDAV_PREFIX.length + 1);
  return "";
}

export function parseDestinationHeader(header: string): string | null {
  if (!header) return null;
  let pathPart = header;
  try {
    pathPart = new URL(header).pathname;
  } catch {
    if (!header.startsWith("/")) return null;
  }
  if (pathPart === WEBDAV_PREFIX || pathPart === `${WEBDAV_PREFIX}/`) return "";
  if (!pathPart.startsWith(`${WEBDAV_PREFIX}/`)) return null;
  return decodeURIComponent(pathPart.slice(WEBDAV_PREFIX.length + 1));
}

function buildEntryHref(relParent: string, name: string, isDirectory: boolean): string {
  const base = relParent.length === 0
    ? `${WEBDAV_PREFIX}/${name}`
    : `${WEBDAV_PREFIX}/${posix.join(relParent, name)}`;
  return isDirectory ? `${base}/` : base;
}

function selfHref(relPath: string, isDirectory: boolean): string {
  if (relPath.length === 0) return isDirectory ? `${WEBDAV_PREFIX}/` : WEBDAV_PREFIX;
  const base = `${WEBDAV_PREFIX}/${relPath.replace(/\\+/g, "/")}`;
  return isDirectory ? `${base.replace(/\/+$/, "")}/` : base;
}

function entryFromStat(href: string, stat: Stats): PropfindEntry {
  return {
    href,
    isDirectory: stat.isDirectory(),
    size: stat.isDirectory() ? 0 : stat.size,
    modifiedAt: stat.mtime,
  };
}

async function buildDirectoryListing(
  target: string,
  relPath: string,
  depth: 0 | 1,
  rootStat: Stats,
): Promise<PropfindEntry[]> {
  const entries: PropfindEntry[] = [entryFromStat(selfHref(relPath, true), rootStat)];
  if (depth === 0) return entries;

  const names = (await fs.readdir(target)).sort();
  for (const name of names) {
    let childStat: Stats;
    try {
      childStat = await fs.stat(join(target, name));
    } catch {
      continue;
    }
    entries.push(entryFromStat(buildEntryHref(relPath, name, childStat.isDirectory()), childStat));
  }
  return entries;
}

function parseDepth(header: string | string[] | undefined): 0 | 1 | "infinity" {
  const value = Array.isArray(header) ? header[0] : header;
  if (value === "0") return 0;
  if (value === "1" || value === undefined || value === "") return 1;
  return "infinity";
}

async function handleDelete(target: string, res: ServerResponse): Promise<void> {
  try {
    await fs.stat(target);
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === "ENOENT") {
      res.statusCode = 404;
      res.end();
      return;
    }
    throw err;
  }
  await fs.rm(target, { recursive: true, force: true });
  res.statusCode = 204;
  res.end();
}

async function handleMkcol(target: string, res: ServerResponse): Promise<void> {
  try {
    await fs.access(target);
    res.statusCode = 405;
    res.end();
    return;
  } catch {
    // target does not exist yet, fall through to create it
  }
  await fs.mkdir(target, { recursive: true });
  res.statusCode = 201;
  res.end();
}

async function handleMove(
  req: IncomingMessage,
  res: ServerResponse,
  opts: WebdavOptions,
  source: string,
): Promise<void> {
  const destinationHeader = req.headers["destination"];
  const headerValue = Array.isArray(destinationHeader) ? destinationHeader[0] : destinationHeader;
  if (typeof headerValue !== "string") {
    res.statusCode = 400;
    res.end();
    return;
  }
  const relDestination = parseDestinationHeader(headerValue);
  if (relDestination === null) {
    res.statusCode = 400;
    res.end();
    return;
  }
  const destination = resolveWorkspacePath(opts.root, relDestination);
  if (destination === null) {
    res.statusCode = 400;
    res.end();
    return;
  }

  let destinationExisted = true;
  try {
    await fs.access(destination);
  } catch {
    destinationExisted = false;
  }
  await fs.mkdir(dirname(destination), { recursive: true });
  await fs.rename(source, destination);
  res.statusCode = destinationExisted ? 204 : 201;
  res.end();
}

async function handlePropfind(
  req: IncomingMessage,
  res: ServerResponse,
  target: string,
  relPath: string,
): Promise<void> {
  const depth = parseDepth(req.headers["depth"]);
  if (depth === "infinity") {
    res.statusCode = 403;
    res.end();
    return;
  }
  let stat: Stats;
  try {
    stat = await fs.stat(target);
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code === "ENOENT") {
      res.statusCode = 404;
      res.end();
      return;
    }
    throw err;
  }

  let entries: PropfindEntry[];
  if (stat.isDirectory()) {
    entries = await buildDirectoryListing(target, relPath, depth, stat);
  } else {
    entries = [entryFromStat(selfHref(relPath, false), stat)];
  }

  res.statusCode = 207;
  res.setHeader("Content-Type", "application/xml; charset=utf-8");
  res.end(buildPropfindXml(entries));
}

async function handleRequest(req: IncomingMessage, res: ServerResponse, opts: WebdavOptions): Promise<void> {
  if (!req.url?.startsWith(WEBDAV_PREFIX)) {
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
    switch (method) {
      case "GET": {
        const data = await fs.readFile(target);
        res.statusCode = 200;
        res.end(data);
        return;
      }
      case "PUT": {
        const body = await readBody(req);
        await fs.mkdir(dirname(target), { recursive: true });
        await fs.writeFile(target, body);
        res.statusCode = 204;
        res.end();
        return;
      }
      case "PROPFIND":
        await handlePropfind(req, res, target, relPath);
        return;
      case "DELETE":
        await handleDelete(target, res);
        return;
      case "MKCOL":
        await handleMkcol(target, res);
        return;
      case "MOVE":
        await handleMove(req, res, opts, target);
        return;
      default:
        res.statusCode = 405;
        res.end();
        return;
    }
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
  const url = `http://${host}:${address.port}${WEBDAV_PREFIX}`;

  return {
    url,
    close: () =>
      new Promise<void>((resolveClose, rejectClose) => {
        server.close((err) => (err ? rejectClose(err) : resolveClose()));
      }),
  };
}

