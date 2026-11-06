---
id: T-4527
title: "Retire the M14-owned path (desktop_owned.py, exit_owners' M14 rule, rustsheet-eval --desktop-owned, the report's M14-owned column and R7 wording) now that decision 0266 leaves no open M14 work"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
No code path depends on an open M14 task: the "only open M14 tasks own it" rule of decision 0143 is retired or rewritten
for a world where decision 0266 leaves no open M14 work, with tests updated.

## Context
Split from T-4207 (decision 0308, PR for the owner's request to list the desktop-only cells as Phase 1 rate exceptions).
Decision 0308 settled the cells with eval/milestones/phase1-exceptions.tsv lines owned by T-4373; it changed no code.
Still in place and now nearly inert: scripts/desktop_owned.py (lists cells only open M14 tasks own), the M14 rule in
scripts/exit_owners.py, `rustsheet-eval run --desktop-owned` (crates/rustsheet-eval/src/phase1_exceptions.rs
`DESKTOP_OWNED_PATH`), the report's "M14-owned, left out" column, and the R7 wording of report.md and
docs/milestones.md ("cells only open M14 tasks own leave the row"). T-4373 (M14, blocked) is the one open M14 task, so
the path still lists its cells; decide whether that is intended (it double-counts with decision 0308's lines) or retire
the path and rely on phase1-exceptions.tsv alone.

## Acceptance criteria
- [ ] A decision record retires or rewrites decision 0143's mechanism; the scripts, the eval flag, the report column and
      the R7 wording follow it, with tests updated.
- [ ] `make eval` then `make phase1-exit`: Gate B and the owners criterion are unchanged by the retirement.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which cells are exceptions (decision 0308).
