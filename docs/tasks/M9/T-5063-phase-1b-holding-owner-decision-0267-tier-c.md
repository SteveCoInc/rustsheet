---
id: T-5063
title: "Phase 1b holding owner: decision 0267 Tier C/D cells whose only owners are deferred"
milestone: M9
lane: functions-stat
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Give an open owner to the visibly-off cells whose only owners are tasks decision 0267 deferred to Phase 1b, so the
Phase 1 owners criterion (decision 0187) reads them as owned, without reopening the deferred work.

## Context
- The owner chose this (2026-11-07): one blocked holding task rather than live tasks or a change to the owners rule.
  The Phase 1 owners check counts a cell owned only by a `defer:` task as unowned. This task carries no `defer:`, and its
  `blocked` status keeps it open and out of the ready queue.
- Split from the deferred owners that already list these cells, which keep the work: T-2765, T-2830, T-2924 (F.TEST),
  T-3232 (GAMMAINV), T-3239 (CHIINV) and T-4662 (the t2436 F.INV.RT chain). This task adds no work of its own.
- Cells (main e560a1bb, `scripts/exit_owners.py --all-off`):
  - probes/t2436/abc2939ebd1e2365.case.json (6): Calc!A1, A16, T1, T16, U1, U16 (the `F.INV.RT(1E-200,1,1.999)` chain)
  - gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A306 (F.TEST)
  - probes/t2829-ftest-a306-shapes.case.json (7): S!A2, A7, A8, A9, A10, A11, A12 (F.TEST and 2*F.DIST.RT)
  - gen-m9/0a614cfe974b59c3.case.json (1): Calc!A327 (GAMMAINV)
  - gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A338 (CHIINV)

- Also held here for the M9 exit row: probes/t2436/e85daedce5c2675e.case.json (2): Calc!A31, Calc!A46. These are
  decision 0329 superseded October cells (accepted known failures, T-4736), counted by M9's probe row.

## Acceptance criteria
- [ ] Each cell above is fixed by the deferred task that owns it, or stays listed here until Phase 1b reopens it.

## Out of scope
- The fixes themselves (the deferred tasks' work, decision 0267 item 4).

## Blocked because
Blocked because: decision 0267 Tier C/D cut. These cells are Phase 1b work (`defer: phase-1b` on their owning tasks);
this task only holds them so the Phase 1 owners check sees an open owner. Unblock when Phase 1b starts.
