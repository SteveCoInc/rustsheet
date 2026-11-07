---
id: T-5038
title: "Probe T.INV and T.INV.2T at c = 0.085, 0.09 and 0.095 to locate Excel's switch to the rounded-y form (T-3370 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence pins where T.INV and T.INV.2T switch from the true quantile to the rounded-y form between c = 0.08 and c = 0.1.

## Context
T-3370's probe `probes-numeric/m9-t-inv-centre` (excel-graph) shows Excel is the true quantile (mpmath at the double p,
within 3.1E-15 relative) at c = 0.08, while the engine's rounded-y form beyond `T_CENTRE_C` (0.075) is off by 2.2E-14
(df 5), 3.8E-14 (df 30), 9.3E-13 (df 1000) and 1.2E-9 (df 1E6). Moving `T_CENTRE_C` is T-3477's (R4 guards, opus); this
task only adds the cells that locate the cut. See statistics.md "Centre quantiles are accurate".

## Acceptance criteria
- [ ] A probes-numeric case with `T.INV(0.5+c/2,df)` and `T.INV.2T(1-c,df)` at c = 0.085, 0.09, 0.095, 0.098 and 0.099 (the t3349 sweep and `T_NEGATIVE_CENTRE_C` evidence point to 0.0985..0.1) for df 5, 30, 1000
      and 1E6, filled by the oracle, committed; statistics.md records which are the true quantile.
- [ ] T-3477's Context names the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine or `T_CENTRE_C` (T-3477); df 1 (T-2858).
