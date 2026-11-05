# 0285 — Each worktree builds into its own Cargo target directory

- **Status:** accepted (2026-11-05, T-3832)
- **Context:** in the 2026-11-02 cleanup, parallel repair agents shared one `CARGO_TARGET_DIR` to
  save disk. Cargo fingerprints by package, not by source tree, and the Makefile runs
  `target/release/rustsheet-eval` by path, so twice a worktree ran a binary built from another
  worktree's sources: FORECAST.ETS returned `#N/IMPL!` although the merged tree implements it, and
  a baseline was recorded with the wrong engine (decision 0245's new-case-owner gate caught it by
  luck). The rule lives in `docs/process/agent-workflow.md`, the work-loop skill and the Makefile,
  all protected paths.
- **Decision:** every worktree, a subagent's `isolation: worktree` copy included, builds into its
  own target directory inside the worktree. A new `make target-dir-guard`
  (`scripts/target_dir_guard.py`) asks Cargo for the target directory (`cargo metadata`, which
  honours `CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` and `.cargo/config.toml`), resolves
  symlinks, and fails when it lies outside the worktree root. `make build`, and so every eval
  target and gate, and `make check-fast` run it first. There is no override switch: a target
  directory inside the worktree is always available.
- **Consequences:** a session short of disk deletes stale worktrees' `target/` directories instead
  of sharing one. CI is unaffected (it builds in the checkout's own `target/`). The guard costs one
  `cargo metadata --no-deps` call, under 0.1 s. No gate is weakened.
