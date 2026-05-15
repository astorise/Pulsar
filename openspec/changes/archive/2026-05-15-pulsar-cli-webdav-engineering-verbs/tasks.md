# Implementation Tasks

- [x] **Engineering verbs**
  - [x] Add a `DELETE` handler that calls `fs.rm(target, { recursive: true, force: true })` and replies `204 No Content`.
  - [x] Add a `MKCOL` handler that calls `fs.mkdir(target, { recursive: true })` and replies `201 Created`.
  - [x] Add a `MOVE` handler that resolves the `Destination` header through `resolveWorkspacePath` before calling `fs.rename`. Reply `201` when the destination did not exist and `204` when it did.

- [x] **Enriched PROPFIND**
  - [x] Replace `buildPropfindXml(entries)` with a structure-aware serializer that emits `<d:resourcetype>` (with `<d:collection/>` for directories), `<d:getcontentlength>`, and `<d:getlastmodified>` per entry.
  - [x] Honor the `Depth` header: `0` returns metadata for the requested resource only; `1` (the default) lists direct children; reject `infinity` with `403`.

- [x] **Cleanup**
  - [x] Remove the unused `webdav-server` dependency from `pulsar-cli-node/package.json`.
  - [x] Cover the new verbs and Depth handling with unit tests under `pulsar-cli-node/src/__tests__/webdav.test.ts`.
