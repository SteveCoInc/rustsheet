---
id: T-3834
title: "Check whether T-3803 and T-3818 duplicate the same BETA.DIST density work"
milestone: M9
lane: research
status: todo
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
- [ ] The two tasks' cell lists and intended fixes are compared. Either both stay, each saying how
      it differs from the other, or one is merged into the other and the other is set to
      `status: dropped` with a pointer.
- [ ] `tasks.py validate` and `make exit-owners` still pass (no cell left unowned).
- [ ] Task-only PR (decision 0231): CI green.

## Out of scope
- Working either task.
