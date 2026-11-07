---
id: T-3988
title: "ODDFYIELD's long first period with a month-end schedule (T-3031 follow-up)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ODDFYIELD's two month-end long-first-period cells of the T-3031 fresh sweep come within 15 digits of Excel.

## Context
T-3031 fitted ODDFYIELD's slope for a long first period (financial.md "ODDFYIELD stops short of ODDFPRICE's root":
a regular coupon at every quasi-coupon date from the settlement's next one Q, every exponent shifted by the basis'
days from Q to the first coupon over E). Its fresh sweep probes-numeric/t3031-r4-oddfyield-long-first passes 157 of
160; two of the three misses have a 28 February first coupon and maturity, a month-end schedule:
- S!A61 `ODDFYIELD(DATE(2010,1,7),DATE(2037,2,28),DATE(2009,8,12),DATE(2010,2,28),0.0085,21.426,100,2,0)`: ours
  0.07748066606613516, Excel 0.07748066607022848 (4e-11 relative).
- S!A70 `ODDFYIELD(DATE(2017,12,3),DATE(2038,2,28),DATE(2017,3,2),DATE(2019,2,28),0.0179,23.169,134.68,1,2)`: ours
  0.1219250863222613, Excel 0.12192508631378622 (7e-11 relative).
Likely the shift's day count or `nq`: Q from `next_coupon_date` on a month-end schedule (28 February steps to 31
August, not 28 August) against the walk ODDFPRICE uses. Probe a few month-end long first periods (28/29 February,
30 April, 31 August first coupons; settlement before a quasi date) on excel-graph and test which Q the slope takes.
S!A80 (1 ulp, a zero coupon, basis 1) is within the reproducibility floor and not a target.

## Acceptance criteria
- [x] probes-numeric/t3031-r4-oddfyield-long-first S!A61 and S!A70 are within half a unit of the 15th significant digit of Excel, or the task is blocked time-boxed (decision 0056) with the candidates recorded in financial.md.
- [x] ODDFYIELD lines in eval/baseline only shrink.
- [x] `make check` green; reviewers approve.

## Out of scope
- ODDFPRICE's own digits (T-1374).

## Outcome
Both cells are bit-exact. The slope of a long first period now runs on its own schedule (`yield_solver::SlopeTerms`;
financial.md "ODDFYIELD stops short of ODDFPRICE's root"): `nq` regular coupons, one at each quasi-coupon date in
(settlement, first coupon) (`coupon_count(settlement, first_coupon) - 1`), and the lead `DFC'/E` periods away, `DFC'`
the basis' days from settlement to the first coupon. ODDFPRICE's `nq` (`coupNumber`) counts one quasi period more when
the first coupon is a month end, and its `DSC` (E minus the days from the previous quasi date) is not `DFC'` on 30/360;
T-3031's slope inherited both. On regular schedules the new form is T-3031's (its shift is `DFC' - DSC` there).
A structural model (decision 0188 R4 kind (c)), so no `FITTED` tag.
- How it was found: a scratch Newton on the engine's schedule scanned the slope's exponent shift in steps of 1/3600
  period with the extra-coupon count at `offset - 1`, `offset`, `offset + 1`: S!A61 and S!A70 each had exactly one
  hit, both at `offset - 1` (the true `nq`), at T-3031's shift -2/180 and +5/360, i.e. the lead at 51/180 (30/360 days
  to the first coupon) and 452/360 (actual days). A probe of 120 month-end long first periods
  (probes-numeric/t3988-r4-oddfyield-month-end, excel-graph) then scored the candidates: the true `nq` with `DSC/E +
  s` 116, with the price's `DSC` 99, T-3031's form 13; its S!A94 (a 31 August first coupon, a 29 February Q, basis 0)
  needs 352 30/360 days to the first coupon where the split at Q gives 351, so the model takes `DFC'` whole: 119 (S!A86
  is 1 ulp). Frozen there.
- R4 guards. Hold-out (R4.1, the fit lab's `is_holdout`, margin 5 points, applied to the cells the model was chosen on):
  t3031-r4-oddfyield-long-first train 116/117/117 of 117 and test 43/43/43 of 43, ODDFPRICE's 50-digit root rounded
  1/1/1 and 0/0/1; t3988-r4-oddfyield-month-end train 87/88/88 of 88, test 32/32/32 of 32, the root 0/0/0 on both.
  Fresh sweep (R4.2): probes-numeric/t3988-r4-oddfyield-fresh-2, 120 seeded calls drawn after the freeze (half with a
  month-end first coupon, a quarter on the 29th/30th, a quarter on days 1-28; every frequency and basis; 2 to 5 quasi
  periods), excel-graph 2026-11-06: 118 of 120 (115/115/115 of 117 numeric, 3 error cells match), T-3031's form 67, the
  root 0/0/0. Gate A envelope on it: 118 within 15 digits; S!A108 5.2e-11 relative of the root (inside 1e-9); S!A54
  2.4e-8 (outside: a yield near 0, where Excel itself is 2.4e-8 off the root; T-3031's form was 1.8e-7 off, so no point
  moves out). Roots: mpmath at 50 digits on the engine's ODDFPRICE schedule dumped from a scratch test (no committed
  ODDFYIELD math model, T-3999). Reproducibility floor: no step targets a difference below 4 ulp (A86, A80 left). No
  keep-matched branch: the counts are the schedule's own.
- One variant tried after the fresh sweep and not taken: keeping T-3031's split at Q with the 30/360 remainder
  `(DFC' - DSC)/E` passes fresh-2 S!A108 but loses two cells of the month-end case (117).
- Eval delta: gated suites unchanged (no ODDFYIELD cell gained or lost; `rustsheet-eval check` 0 fixed, 0 regressions).
  probes-numeric (ungated): t3031-r4-oddfyield-long-first 157 → 159 of 160; new cases t3988-r4-oddfyield-month-end
  (119 of 120) and t3988-r4-oddfyield-fresh-2 (118 of 120).
- Follow-up: T-4551 (fresh-2 S!A54 and S!A108).
