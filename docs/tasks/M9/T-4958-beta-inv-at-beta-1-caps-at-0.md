---
id: T-4958
title: "BETA.INV at beta 1 caps at 0.999999 and the flipped quantile rounding to 1 is 1 (T-4713 split)"
milestone: M9
lane: functions-stat
status: done
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
- [x] The cells above match Excel, or a probe recorded in statistics.md shows why not.
- [x] `make check` green; reviewers approve.

## Out of scope
- T-4700 (the cap at a tiny beta), T-4713's density rule.

## Outcome
- F12:F17: at beta = 1 with p up to 1/2, BETA.INV is now 0.999999 where the closed form p^(1/alpha) rounds to 1
  (`shape_one_quantile_cap`, functions/statistical/beta.rs). Below that point it keeps the crossing, which F1:F11 show
  is Excel's value, down to 1 - 2^-53. The guard is only beta = 1, the evidenced case. The new statistics.md entry
  "BETA.INV at beta 1 caps at 0.999999 where its quantile rounds to 1 (T-4958)" records the rule, its evidence and
  the open mechanism, which may be a clamped AS 109 start (T-4700).
- E41 (`BETA.INV(0.9,1E+16,2)`, Excel 1) already passed on main: T-3351 (#3911) fixed it with its p > 1/2 rule
  (1 - y with y normal). Nothing changed there.
- Unit test: `beta_one_quantile_caps_at_0_999999_where_it_rounds_to_1` (beta.rs tests). No fitted constant:
  0.999999 is Excel's output value.
- Eval delta: probes known failures -6 (t4713-beta-inv-beta-above-1-cut S!F12:F17), 0 regressions.
