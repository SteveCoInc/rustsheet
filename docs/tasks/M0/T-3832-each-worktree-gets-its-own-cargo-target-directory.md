---
id: T-3832
title: "Each worktree gets its own Cargo target directory"
milestone: M0
lane: harness
status: todo
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
- [ ] `docs/process/agent-workflow.md` (and the work-loop skill where agents run in parallel, e.g.
      subagents with `isolation: worktree`) says that each worktree uses its own target directory,
      and why.
- [ ] `make check-fast` / `make eval-gates` refuse, or warn loudly, when `CARGO_TARGET_DIR` points
      outside the current worktree. Or record in the Outcome why a guard is not worth it.
- [ ] A decision record if a protected path changes; `make check-fast` green; reviewers approve.

## Out of scope
- Disk-space tuning of the session containers.
