---
id: T-3364
title: "GAMMA.INV's huge-shape closed-form branch follows the math (decision 0188 R3); F density and Student-t fits moved to T-4876"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-3355, T-3361]
---

## Goal
The fitted pieces that are applied beyond the inputs they were measured on stop at their measured range and fall back to
accurate math outside it (decision 0188 R4.3), and GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV follow the math on
the huge-shape Wilson-Hilferty closed-form branch, where Excel is about 2.7e-4 off (decision 0188 R3); their `#NUM!`
and true-quantile branches keep matching Excel.

## Context
- Owner's 2026-10-31 assessment (decision 0188, code audit): extrapolated fits.
  - statistical/incomplete_gamma/huge_shape.rs `WILSON_HILFERTY_SPREAD` and its terms: fitted at shape 1E13, applied
    from 2E10. Excel's GAMMA.INV at a huge shape is about 2.7e-4 off the math, an R3 case (`MATH-OVER-EXCEL`,
    statistics.md).
  - statistical/f_density.rs `STIRLING_ERROR` table: used past df 56, beyond its measured cells.
  - statistical/student_t.rs df bands (`LOG_GAMMA_BETA_MIN` 6.5, `LOG_GAMMA_BETA_MAX`, `SERIES_MAX_DF`,
    `ASYMPTOTIC_MIN_SHAPE`).
- The `FITTED(T-XXXX, n_cells, range)` tag comes from T-3361; this task uses each site's recorded range.
- T-3354 (P0) may touch the F density first; build on it.
- From T-3350 (2026-10-31), cells on the R3 closed-form branch (ours is Excel's closed form, about 2.66E-4 from the
  math; owned with T-3272 for the digits):
  - probes/t3350-chisq-inv-huge-df A301:A351 (visibly off: A301, A313, A325, A326, A337, A338, A349, A350, A351);
  - probes-numeric/t3350-r4-chisq-inv-sweep A122, A127, A155 (outside both Gate A bounds).

## Acceptance criteria
- [x] Moved to T-4876 for the F density and Student-t (2026-11-07): each piece above applies only inside its
      recorded `FITTED` range. GAMMA.INV's start fit now only picks the branch, so it no longer gives a value.
- [x] On the closed-form branch (statistics.md's `MATH-OVER-EXCEL` entry; t3070 A110, t2982 A7-A23, and the closed-form numeric cells of the five functions among t3070 A304-A327; A315-A317 are CHISQ.DIST/CHISQ.DIST.RT and stay `#NUM!`)
      the five functions return the math within 1e-9 relative; the `#NUM!` and true-quantile branches keep Excel's
      results.
- [x] (moved to T-4876 for the F density and Student-t; the GAMMA.INV branch is the math) A fresh log-uniform sweep over each affected function's domain (R4.2) shows the Gate A envelope (R2).
- [x] No gated cell regresses beyond decision 0083's allowance; cells moved to the math by R3 are listed in the Outcome.
- [x] Before any engine change follows the math, the region's evidence points (oracle-filled, with 50-digit math
      values) and its bounds are committed and the `MATH-OVER-EXCEL` entry cites them; the engine change comes after,
      in the same PR or a later one.
- [x] The math path keeps directive 5's caps (bounded iterations, no panic); decision 0188 R3 never overrides them.
- [x] R3 cells are marked as the Gate A spec (T-3355) says. Interim route until T-3359 lands: the cells that leave
      Excel's value are listed in the Outcome, they still count as visibly off in the report, and their baseline
      regression is recorded with a decision record citing decision 0188 R3.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-validating the in-range form selectors (T-3365).

## Outcome
Done 2026-11-07 (decision 0337). `huge_shape_quantile` (incomplete_gamma/huge_shape.rs) returns the true quantile where
Excel's start density underflows, as the unit-scale quantile times the scale (refining at the scale itself lost
`GAMMAINV(0.5,1E300,1E8)`). `START_SHAPE_MIN` is gone. The `#NUM!` and true-quantile branches are unchanged.
- Gate A: the 132 known failures this task owned pass (321 → 189 known failures).
- R3 cells moved to the math (marked in eval/accuracy/math-over-excel.tsv): 206 probe cells from shape 1E9.
  - probes/t3070-gamma-inv-band: 122 cells, including A110 and the closed-form cells of A304-A327.
  - probes/t2982-gamma-inv-huge-shape: 64 cells, including A7-A23, A92 and A93.
  - probes/t3350-chisq-inv-huge-df: 19 cells, including A301-A351.
  - probes/incomplete-gamma-cap: 1 cell.
  - 59 of these are new probe known failures against Excel, recorded under decision 0337.
- Without a math value: five gen-m9 multi-call cells, which raise their visible-parity lock by one each. They are
  0a614cfe974b59c3 A327, 4e55500bb8ad3cd8 A341, 7bd219ec0c0c5e22 A239, a013a45a8fbcab4f A338 and
  d8266bc15533702a A296; three of them (A341, A239, A296) are also new gen-m9 known failures.
- Unchanged: the cells below shape 1E9 (t3350 A205-A289) keep the true quantile, outside R3, under the owner's
  T-3409 ruling (T-4205).
- The F density `STIRLING_ERROR` and Student-t df-band parts moved to T-4876.
- The probes-numeric/t3350-r4-chisq-inv-sweep cells A122, A127 and A155 are not probe-suite cells and are not marked.
