# Proposal: Pulsar Foundation, Workspace, and OCI WIT Resolution

**Date**: 2026-05-14
**Status**: Archived
**Category**: Build System & Architecture

## Why
A recent comprehensive audit of the Pulsar architecture revealed critical build and dependency management issues. Currently, the project attempts to target a non-existent Rust edition (`2024`), lacks a root Cargo workspace, and relies on opaque, untracked WebAssembly Interface Type (WIT) contracts from the Tachyon-Mesh host environment.

These issues lead to broken builds, massive dependency duplication across the 8 FaaS crates, and severe ABI drift risks between Pulsar agents and the Tachyon host.

## What Changes
1. **Edition Downgrade**: Revert all FaaS crates from `edition = "2024"` to the stable `edition = "2021"`.
2. **Workspace Consolidation**: Introduce a root `Cargo.toml` workspace to centralize and share core dependencies (`serde`, `tokio`, `anyhow`, `wit-bindgen`).
3. **Toolchain Pinning**: Introduce a `rust-toolchain.toml` to guarantee reproducible builds across all CI/CD environments.
4. **OCI Component Resolution**: Replace local or implicit WIT dependencies with standardized OCI artifact resolution via GitHub Container Registry (GHCR), relying on the newly published `tachyon-mesh-wit` packages.

## Impact
* **Deterministic Builds**: Guaranteed identical builds locally and in CI.
* **Reduced Compile Times**: Shared dependency resolution via the workspace cache.
* **ABI Safety**: The Wasm component compilation will strictly validate against the immutable, versioned OCI WIT contracts published by the Tachyon team.
