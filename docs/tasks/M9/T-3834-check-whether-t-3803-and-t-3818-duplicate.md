---
id: T-3834
title: "Check whether T-3803 and T-3818 duplicate the same BETA.DIST density work"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Settle whether T-3803 and T-3818 own the same BETA.DIST density work, and leave exactly one task per
piece of work.

## Context
Two task-only PRs merged during the 2026-11-02 cleanup filed overlapping tasks: T-3803 (#2821)
covers 11,307 failing density cells on the t3568-r5-bup-terms probes, and T-3818 (#2828) covers
7,026 unowned visibly-off cells at one large shape on the same probes. `tasks.py validate` found no
cell overlap, but the repair session flagged that they may describe the same underlying fix.

## Acceptance criteria
- [x] The two tasks' cell lists and intended fixes are compared. Either both stay, each saying how
      it differs from the other, or one is merged into the other and the other is set to
      `status: dropped` with a pointer.
- [x] `tasks.py validate` and `make exit-owners` still pass (no cell left unowned).
- [ ] Task-only PR (decision 0231): CI green.

## Out of scope
- Working either task.

## Outcome
They duplicated each other. Both owned probes-numeric/t3568-r5-bup-terms-tune and -hold whole (decision 0136 case
lines), and both meant the same fix: the shared BETA.DIST density kernel (Loader saddle point for min(a,b) > 1,
`beta_density_small_shape` for min(a,b) <= 1). T-3818's 7,026 visibly-off cells are a subset of T-3803's 11,307
failing cells. `validate` found no overlap because both name whole cases, not cells.
T-3803 stays as the one owner because it covers every failing cell and maps each shape band to its kernel owner. T-3818
is `status: dropped` with a pointer to it. T-3818's own content (counts, examples, the mpmath accuracy sample,
root-cause notes, the "start from T-3220/T-3244/T-3250, no second model" rule, the statistics.md criterion,
`campaign: visible` with its visible-parity stage) moved into T-3803, so the visible-stage task counts do not change.
Ownership is unchanged: `tasks.py validate` and `make exit-owners` pass. Eval delta: none (task files only).
