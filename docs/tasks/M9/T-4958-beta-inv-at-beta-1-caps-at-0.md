---
id: T-4958
title: "BETA.INV at beta 1 caps at 0.999999 and the flipped quantile rounding to 1 is 1 (T-4713 split)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
BETA.INV matches Excel at a huge alpha where beta is exactly 1, and where the flipped (p > 1/2) quantile rounds to 1.

## Context
- probes/t4713-beta-inv-beta-above-1-cut F12:F17 `BETA.INV(0.1,a,1)` for a = 5.6E+16 to 1E+18: Excel 0.999999, ours
  0.9999999999999999 (F1:F11, a = 1E+14 to 3.2E+16, agree). 0.999999 = 1 - 1E-6 is the cap T-4700 sees at a tiny beta
  (docs/excel-semantics/statistics.md "BETA.INV is #NUM! once the square of AS 109's first shape overflows (T-3862)",
  "Below the cut" bullet): find where it applies at beta = 1 (`beta_quantile_shape_one` in
  functions/statistical/beta.rs takes a closed form there).
- probes/t4713-beta-inv-beta-above-1-cut E41 `BETA.INV(0.9,1E+16,2)`: Excel 1, ours `#NUM!` (E33:E40, Excel 1 and ours
  1 - 2^-53, one ulp). Our "a quantile that rounds to 1 is #NUM!" (beta.rs `beta_inv_of`, T-2255) does not hold
  for the flipped search here.
- statistics.md "BETA.INV and BETA.DIST's density are #NUM! where Loader's n - k is not positive (T-4713)", "In
  passing" bullet.

Cells this task owns:
- probes/t4713-beta-inv-beta-above-1-cut.case.json (7): S!F12:F17, S!E41

## Acceptance criteria
- [ ] The cells above match Excel, or a probe recorded in statistics.md shows why not.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4700 (the cap at a tiny beta), T-4713's density rule.
