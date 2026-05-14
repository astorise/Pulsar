# Tasks: Secrets, CI, and Hygiene

- [x] Locate the `SESSIONS` Mutex in `faas/orchestrator/src/lib.rs` and remove the plaintext `workspace_token`.
- [x] Refactor the orchestrator to fetch the token securely from the environment or host interface right before making an API call.
- [x] Update `.github/workflows/test.yml` to run `clippy` and `test` on the entire `--workspace`.
- [x] Add the `gitleaks` GitHub Action step to the CI pipeline.
- [x] Add the `LICENSE` file (MIT) to the repository root.
- [x] Rewrite `README.md` to distinguish between vision and current reality, and explicitly mention the dependency on Tachyon-Mesh.
- [x] Add minimal `README.md` files to each current FaaS crate describing its specific bounded context.
- [x] `git mv` the `knowledge-graph` and `token-killer` specs into the `openspec/archive/` folder.
