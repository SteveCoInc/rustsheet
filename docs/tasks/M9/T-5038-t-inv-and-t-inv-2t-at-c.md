---
id: T-5038
title: "T.INV and T.INV.2T at c = 0.08: Excel is the true quantile, ours is the rounded-y form (T-3370 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
T.INV and T.INV.2T at a central probability c = |2p - 1| (T.INV) or 1 - p (T.INV.2T) of 0.08 give Excel's value (the true quantile), as the cases below do.

## Context
T-3370's probe `probes-numeric/m9-t-inv-centre` (excel-graph) shows Excel is the true quantile (mpmath at the double p, within 3.1E-15 relative) at c = 0.08, while `t_centre_quantile` stops at `T_CENTRE_C` = 0.075 and the rounded-y form beyond is off by 2.2E-14 (df 5), 3.8E-14 (df 30), 9.3E-13 (df 1000) and 1.2E-9 (df 1E6; Excel -0.10043374587317014, ours -0.10043374575030144). Cells (Calc sheet): A5, A8, A9, A15, A18, A19, A25, A28, A29, A35, A38, A39 and A45, A48, A49 (T.INV(0.46|0.54,df), T.INV.2T(0.92,df) at df 2, 5, 30, 1000, 1E6). The switch to the rounded-y form lies between c = 0.08 and c = 0.1 (T-3061: rounded-y at q = 0.45); probe c = 0.085, 0.09, 0.095 to locate it, then move `T_CENTRE_C` there. See statistics.md "Centre quantiles are accurate".

## Acceptance criteria
- [ ] The listed cells pass; a probe at c = 0.085, 0.09 and 0.095 locates the switch and statistics.md records it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- df 1 (T-2858).
