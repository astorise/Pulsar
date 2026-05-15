## ADDED Requirements

### Requirement: WebDAV DELETE
The Node.js workspace bridge SHALL accept `DELETE` requests against the workspace WebDAV endpoint, recursively remove the resolved file or directory, and respond `204 No Content`.

#### Scenario: Agent deletes an obsolete file
- **GIVEN** the workspace contains `src/legacy.ts`
- **WHEN** an authenticated `DELETE /webdav/src/legacy.ts` request is received
- **THEN** the bridge removes `src/legacy.ts` from the worktree and responds `204`

### Requirement: WebDAV MKCOL
The Node.js workspace bridge SHALL accept `MKCOL` requests, create the target directory (including missing parents), and respond `201 Created`.

#### Scenario: Agent scaffolds a new component directory
- **GIVEN** the workspace does not yet contain `src/components/wizard`
- **WHEN** an authenticated `MKCOL /webdav/src/components/wizard` request is received
- **THEN** the bridge creates the directory tree and responds `201`

### Requirement: WebDAV MOVE Within the Workspace
The Node.js workspace bridge SHALL accept `MOVE` requests, parse the `Destination` HTTP header, resolve it through the same path guard used for source paths, and rename the source onto the destination.

#### Scenario: Agent renames a file
- **GIVEN** the workspace contains `src/old.ts` and a `Destination` header pointing at `/webdav/src/new.ts`
- **WHEN** an authenticated `MOVE /webdav/src/old.ts` request is received
- **THEN** the bridge renames the file and responds `201` (or `204` if the destination already existed)

#### Scenario: Destination escapes the workspace
- **GIVEN** the `Destination` header points outside the workspace root
- **WHEN** the bridge resolves the path
- **THEN** the request is rejected with `400 Bad Request`

## MODIFIED Requirements

### Requirement: Authenticated Workspace WebDAV Bridge
The CLI SHALL expose the current working directory through a Node.js WebDAV server (built on `node:http`) that supports `GET`, `PUT`, `PROPFIND`, `DELETE`, `MKCOL`, and `MOVE` with bearer-token authorization. `PROPFIND` responses SHALL include `<d:resourcetype>` (with `<d:collection/>` for directories), `<d:getcontentlength>`, and `<d:getlastmodified>` for every listed resource, and SHALL honor `Depth: 0` (metadata for the requested resource only) and `Depth: 1` (direct children, the default). `Depth: infinity` SHALL be rejected with `403 Forbidden`.

#### Scenario: PROPFIND distinguishes files from directories
- **GIVEN** a workspace whose root contains a file `README.md` and a directory `src`
- **WHEN** an authenticated `PROPFIND /webdav` request arrives without a `Depth` header
- **THEN** the response is `207 Multi-Status` and contains a `<d:response>` for `src` carrying `<d:resourcetype><d:collection/></d:resourcetype>`
- **AND** a `<d:response>` for `README.md` carrying an empty `<d:resourcetype/>` together with `<d:getcontentlength>` and `<d:getlastmodified>`

#### Scenario: PROPFIND honors Depth 0
- **GIVEN** a workspace directory `src`
- **WHEN** an authenticated `PROPFIND /webdav/src` request arrives with `Depth: 0`
- **THEN** the response describes only `src` itself, without enumerating its children
