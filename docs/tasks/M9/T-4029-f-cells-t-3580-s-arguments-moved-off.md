---
id: T-4029
title: "F cells T-3580's arguments moved off 15 digits, and the fit-lab split for its R4.1 record"
milestone: M9
lane: functions-stat
status: done
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
- [x] Each listed cell is within 15 digits of Excel, or its branch is recorded here and named by the open task that owns
      that branch.
- [x] docs/research/t3580/RESULTS.md carries the fit-lab split table above, reproduced by a committed script.
- [x] No cell of t3567-r5-fdf1-hold or t3580-f-fresh is newly visibly off; no gated known failure is added.
- [x] `make check` green; reviewers approve.

## Out of scope
- The three gated bup cells of decision 0263, and T-4026's fresh-sweep cells.

## Outcome
Research and bookkeeping only; no engine change, so no eval delta (all 10 listed cells are still off by the same
amounts on origin/main 82b5cd14, and no cell of t3567-r5-fdf1-hold or t3580-f-fresh moves).

Each cell's branch, from `bratio::dispatch` on `fisher.rs` `f_beta_arguments` (a temporary unit test, not committed).
Old arguments (`f_kernel_arguments`) give the same value through `f_closed_or_ported_tails` on every cell, except
A704/B704 at 1 ulp. So the arguments did not move them: T-3580 sending F through the bratio port did, where the
fallback Cephes kernel happened to land closer. Every cell is first wrong at the 15th or 16th significant digit, the
region decision 0267 accepts until Phase 1 exits.

| cells | branch (oriented a0, b0, x0) | owner |
|---|---|---|
| t2220-f-dist-sweep-rt A974, t2220-f-dist-sweep-cdf A974 | L140 bup+bgrat (50, 10, 0.833) | T-3712 (n >= 2 bup terms) |
| t3568-r5-bup-fresh A565, A612 | L140 bup+bgrat (881, 6.5, 0.961), (838, 9.5, 0.987) | T-3712 |
| t3953-f-tail-bfrac-tiny A515, B515, A704, B704, A997, B997 | Bfrac at x0 = v: (1.5, 126, 1.6e-7), (5.5, 140, 5.8e-9), (2.5, 3260.5, 4.3e-15) | T-3954 |

T-3712 and T-3954 now list these cells in their Context. The fit-lab split is in docs/research/t3580/RESULTS.md,
"The fit lab's split (T-4029)", reproduced by `split.py` (cd tools && uv run python ../docs/research/t3580/split.py):
train 3,835/4,714/4,794 of 4,883, test 1,647/2,022/2,052 of 2,084, the review's counts exactly. `score_train.py` is
split into functions so `split.py` reuses its rows, engine values, 50-digit reference and tally (its own output is
unchanged: 2,647/3,350/3,403, correctly rounded 1,497/2,972/3,165).
