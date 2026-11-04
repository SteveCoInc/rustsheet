---
id: T-3988
title: "ODDFYIELD's long first period with a month-end schedule (T-3031 follow-up)"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] probes-numeric/t3031-r4-oddfyield-long-first S!A61 and S!A70 are within half a unit of the 15th significant digit of Excel, or the task is blocked time-boxed (decision 0056) with the candidates recorded in financial.md.
- [ ] ODDFYIELD lines in eval/baseline only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODDFPRICE's own digits (T-1374).
