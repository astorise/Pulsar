# Tasks: Foundation & OCI Integration

- [x] Create `/rust-toolchain.toml` with the pinned compiler version and `wasm32-wasip1` target.
- [x] Create `/Cargo.toml` to define the workspace members and centralize dependencies.
- [x] Search and replace `edition = "2024"` with `edition = "2021"` across all `faas/*/Cargo.toml` and `pulsar-cli/Cargo.toml`.
- [x] Update all leaf `Cargo.toml` files to use `{ workspace = true }` for shared crates.
- [x] Add `[package.metadata.component.dependencies]` blocks to all FaaS agents targeting the `ghcr.io/astorise/tachyon-mesh-wit:0.9.0-rc.1` OCI registry.
- [x] Run `cargo fetch` and `cargo build --target wasm32-wasip1` to generate the unified `Cargo.lock`.
- [x] Commit the new `Cargo.lock` to version control.
