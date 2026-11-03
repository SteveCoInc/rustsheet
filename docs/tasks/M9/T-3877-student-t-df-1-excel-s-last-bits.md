---
id: T-3877
title: "Student t df 1: Excel's last bits at T.DIST.RT(60,1), T.DIST.2T(7.1,1) and inside the cut (t 1 to 3)"
milestone: M9
lane: functions-stat
status: todo
defer: phase-1b
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Bit parity for the df-1 Student-t tails that T-3187 left 1 or 2 ulp from Excel, all within 15 digits (Phase 1b).

## Context
- Follow-up of T-3187 (decision 0083 ulp-regression allowance, item 4). Since T-3187, df 1 and 2 past the closed-form
  cut (fl(1 - y) > 0.9) take the exact tail at t, rounded once (`exact_low_df_tail` in
  crates/rustsheet-engine/src/functions/statistical/student_t.rs). Excel is about 0.7 ulp from the exact tail at
  these cells, so they moved from bit-exact to 1 ulp off:
  - ironcalc-calc/STATISTICAL/T_DIST.xlsx `T.DIST T.INV T.DIST.RT!F2` (`T.DIST.RT(60,1)`, Excel 0.00530467363262327)
    and `T.DIST.2T T.INV.2T!C2` (`T.DIST.2T(60,1)`, 0.01060934726524654);
  - gen-m9/bbab6562e88b70e8 Calc!A113 (`T.DIST.2T(7.1,1)`, 0.08907881192950366).
- Inside the cut (t 1 to 3 at df 1) the tail at the rounded y stays. Excel is 1.8 ulp from exact at `T.DIST.RT(3,1)`
  and the rounded-y path matches it, but `T.DIST.RT(2,1)` and `T.DIST.2T(2,1)` (probes-numeric m9-t-dist-rt A5,
  m9-t-dist-2t A5) are 2 ulp from Excel.
- At the regressed cells, plain double atan(1/t)/pi gives Excel's bits, but not at `T.DIST.2T(1.005,1)` or
  `T.DIST.2T(1E15,1)`. No single form found (T-3187 scoring, docs/excel-semantics/statistics.md "T.DIST.RT at df 2 is
  BETA.DIST's closed form down to y = 0.1"). This is fitting Excel's last bits, so decision 0056's time-box applies.

## Acceptance criteria
- [ ] The three cells above and m9-t-dist-rt/-2t A5 are bit-exact, with no other cell regressed, or the time-box
      outcome is recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
