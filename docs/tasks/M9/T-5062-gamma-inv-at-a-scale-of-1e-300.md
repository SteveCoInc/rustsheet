---
id: T-5062
title: "GAMMA.INV at a scale of 1E-300: gen-m9 d8266bc15533702a A296 is 2.7E-4 off Excel"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`GAMMA.INV(p, a, 1E-300)` matches Excel at a tiny scale.

## Context
- Found by the Phase 1 owners check (2026-11-07): a visibly-off cell attributed to no function and owned by no open task.
  It was T-1891's, which is done.
- gen-m9/d8266bc15533702a.case.json Calc!A296:
  `GAMMA.INV(((2^Data!A2)^(Data!C9-Data!B$9)),AVERAGE((Data!A2*7),Data!B4:B6,Data!E2:E6),1E-300)`, Excel
  1.4289520614558458E-286, ours 1.4285723133892187E-286 (2.7E-4 relative, the 4th digit). The result is scale times the
  unit quantile, so the gap is in the unit quantile or in how Excel applies a 1E-300 scale (an underflow on the way?).
  Compare the unit quantile at the same p and shape first.
- Code: crates/rustsheet-engine/src/functions/statistical (GAMMA.INV); T-1891, T-3311, T-1634 for the search.

## Acceptance criteria
- [ ] gen-m9 d8266bc15533702a A296 within 15 digits of Excel, or blocked as time-boxed (decision 0056) with the
      evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
