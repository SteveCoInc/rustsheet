---
id: T-3832
title: "Each worktree gets its own Cargo target directory"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
No agent builds or tests from another worktree's binaries: concurrent worktrees never share a
Cargo target directory.

## Context
In the 2026-11-02 cleanup, parallel repair agents shared one `CARGO_TARGET_DIR` to save disk. Twice
a worktree ran a binary built from another worktree's sources. Once, FORECAST.ETS returned
`#N/IMPL!`, although the merged tree implements it. Another time a baseline was recorded with the
wrong engine, and decision 0245's new-case-owner gate caught it only by luck before push. Cargo
fingerprints by package, and the `rustsheet-eval` release binary and its output paths are shared.

## Acceptance criteria
- [x] `docs/process/agent-workflow.md` (and the work-loop skill where agents run in parallel, e.g.
      subagents with `isolation: worktree`) says that each worktree uses its own target directory,
      and why.
- [x] `make check-fast` / `make eval-gates` refuse, or warn loudly, when `CARGO_TARGET_DIR` points
      outside the current worktree. Or record in the Outcome why a guard is not worth it.
- [x] A decision record if a protected path changes; `make check-fast` green; reviewers approve.

## Out of scope
- Disk-space tuning of the session containers.

## Outcome
- `scripts/target_dir_guard.py` (tests in `scripts/test_target_dir_guard.py`) asks Cargo for the
  workspace's target directory (`cargo metadata --no-deps --offline`, so `CARGO_TARGET_DIR`,
  `CARGO_BUILD_TARGET_DIR` and `.cargo/config.toml` all count; the environment, then `target/`,
  when Cargo cannot answer), resolves symlinks and fails loudly when it lies outside the worktree.
- `make target-dir-guard` runs it; `make build` (so every eval target and `make eval-gates`) and
  `make check-fast` depend on it. No override switch.
- `docs/process/agent-workflow.md` gains a "Worktrees" paragraph (one target directory per
  worktree, and why); the work-loop skill's step 4 points subagents with `isolation: worktree` at
  it. Decision 0285 records the protected-path change.
- Eval delta: none (process and tooling only).
