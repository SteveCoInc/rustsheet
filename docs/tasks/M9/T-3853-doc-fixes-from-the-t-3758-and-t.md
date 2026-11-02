---
id: T-3853
title: "Doc fixes from the T-3758 and T-3808 reviews: reduce's 28-bit n and T-3847's product"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3758]
---

## Goal
Two comments and a task file say what the code and the evidence show, after MINOR review findings that did not start a push.

## Context
- T-3758 fidelity review: the doc comment of `reduce` (crates/rustsheet-engine/src/functions/math_and_trigonometry/trig.rs)
  says "n has at most 27 bits" and "|n| < 2^27 (the `trig_arg` cut)". Since T-3758, `double_angle_sin_cos`
  (functions/engineering/complex/trig.rs) passes 2x for |x| < 2^27, so |n| < 2^28. The product n·X87_HALF_PI_TAIL is
  then at most 38 bits and stays exact, and the i64 cast is exact; only the wording is stale.
- T-3808 fidelity review: docs/tasks/M9/T-3847-*.md says "The product 8.1E+314"; 9E+307·9E+307·1E-300 is 8.1E+315.
  It should also say that Excel is correctly rounded there: the 50-digit cube root, 2.00829885024650856565...E+105,
  rounds to Excel's 2.0082988502465085E+105 (probes/t3808-median-geomean-direct-overflow A6), so T-3847's fix is
  accurate math, not a fit.
- The T-3808 Outcome's AGGREGATE sentence is broader than the grid (numbers.md gives the exact combinations); leave
  done task Outcomes alone.

## Acceptance criteria
- [ ] `reduce`'s comment states |n| < 2^28 (28 bits) and names the double-angle callers; no code change.
- [ ] T-3847's Context has 8.1E+315 and the 50-digit reference value.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine behaviour change.
