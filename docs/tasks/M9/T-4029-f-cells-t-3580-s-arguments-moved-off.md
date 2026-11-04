---
id: T-4029
title: "F cells T-3580's arguments moved off 15 digits, and the fit-lab split for its R4.1 record"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3580]
campaign: accuracy
---

## Goal
The ungated F cells that T-3580's r >= 1 arguments moved from within 15 digits of Excel to visibly off are back within
15 digits, or each is explained by the open bratio branch that owns it. T-3580's R4.1 record also gains the fit lab's
split.

## Context
- Found by T-3580's excel-fidelity review (PR #3085), comparing origin/main with T-3580's head on all 304 F cases. Each
  move is a few ulp, against several hundred cells newly within 15 digits in the same cases:
  - probes-numeric t2220-f-dist-sweep-rt A974 `F.DIST.RT(1,100,20)`: Excel 0.5307910401772583, before 0.5307910401772582,
    after 0.5307910401772578.
  - probes-numeric t2220-f-dist-sweep-cdf A974 `F.DIST(1,100,20,TRUE)`: Excel 0.4692089598227417, before
    0.4692089598227418, after 0.46920895982274224.
  - probes-numeric t3568-r5-bup-fresh A565 and A612.
  - probes-numeric t3953-f-tail-bfrac-tiny A515, B515, A704, B704, A997 and B997.
- First step: label each cell with its bratio branch on `fisher.rs` `f_beta_arguments` (`bratio::dispatch`). Expected:
  L140 bup+bpser or bfrac, whose arithmetic on Excel's arguments is open (statistics.md "F's incomplete-beta arguments
  keep r >= 1", decision 0263).
- The review reproduced the fit lab's split (`fitlab/holdout.py`, salt `rustsheet-fitlab-holdout/v1`, 30%) over
  cells.tsv's F cdf/tail cells together with t3567-r5-fdf1-hold (6,967 cells). Train: 3,835/4,714/4,794 of 4,883
  (78.5% bit-exact), correctly rounded 2,004/3,778/4,099. Test: 1,647/2,022/2,052 of 2,084 (79.0%), correctly rounded
  855/1,639/1,762. Record it in docs/research/t3580/RESULTS.md next to the dataset split.

## Acceptance criteria
- [ ] Each listed cell is within 15 digits of Excel, or its branch is recorded here and named by the open task that owns
      that branch.
- [ ] docs/research/t3580/RESULTS.md carries the fit-lab split table above, reproduced by a committed script.
- [ ] No cell of t3567-r5-fdf1-hold or t3580-f-fresh is newly visibly off; no gated known failure is added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The three gated bup cells of decision 0263, and T-4026's fresh-sweep cells.
