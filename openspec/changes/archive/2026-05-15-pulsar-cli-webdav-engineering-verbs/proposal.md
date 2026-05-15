# Proposal: WebDAV Engineering Verbs for the Node.js CLI

## Why
The Node.js WebDAV bridge only answers `GET`, `PUT`, and `PROPFIND`. Pulsar
agents performing real engineering — refactoring, scaffolding new components,
deleting obsolete files — receive `405 Method Not Allowed` whenever they emit
`DELETE`, `MKCOL`, or `MOVE`. The current `PROPFIND` response also lacks
`<d:resourcetype>` and ignores the `Depth` header, so an agent cannot tell a
file from a directory when walking the workspace.

## What Changes
Extend `pulsar-cli-node/src/webdav.ts` with the minimum slice of RFC 4918
required for engineering workflows:

* `DELETE` — recursively remove the target file or directory.
* `MKCOL` — create the target directory (and any missing parents).
* `MOVE` — rename within the workspace using the `Destination` HTTP header,
  routed through `resolveWorkspacePath` so traversal stays blocked.
* `PROPFIND` — emit `<d:resourcetype>` (with `<d:collection/>` for directories),
  `<d:getcontentlength>`, and `<d:getlastmodified>`. Honor `Depth: 0` (return
  metadata for the requested resource only) and `Depth: 1` (default — direct
  children). `Depth: infinity` is not supported.

Locking (`LOCK`/`UNLOCK`) remains out of scope — the orchestrator and
cognitive-GC already arbitrate concurrent access.

## Impact
* `pulsar-cli-node` gains four new request handlers and an enriched XML
  serializer; the unused `webdav-server` runtime dependency is dropped because
  the implementation is now hand-rolled on `node:http`.
* No changes to the orchestrator or the VS Code extension are required; both
  already speak WebDAV over HTTP.
