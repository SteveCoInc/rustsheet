---
id: T-4307
title: "Near-mode incomplete gamma follow-ups from T-4000: probe shapes 20 to 21, rename the double-double lower_series, fix two wordings"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The incomplete gamma's near-mode band has evidence at its lower edge: Excel's value is established at non-integer
shapes from 20 to 21 near the mode, so the band start (`near_mode.rs` `BAND_SHAPE_MIN`, UNVERIFIED there) is either
confirmed or moved. The double-double kernel's names and docs are not misleading.

## Context
Follow-ups from T-4000's reviewers (PR #3298, all MINOR):
- `BAND_SHAPE_MIN` = 21 comes from the task's range. At shape 20, Excel is the double kernel's bits:
  `POISSON.DIST(19,20.9,TRUE)` is 2.3 ulp below the truth (probes t3252-gamma-prefactor-window A103). No cell has a
  shape between 20 and 21. Only the quantiles reach that range on the accurate pair: GAMMA.INV, CHISQ.INV and
  CHISQ.INV.RT at df 41, say. Probe them near the mode with the `excel-probe` skill, at shapes 20.25, 20.5 and 20.75
  and x from 0.6a to 1.4a, and score the double kernel against the double-double pair.
- `near_mode.rs` `lower_series` has the same name as the double kernel's `incomplete_gamma::lower_series`. Rename it,
  e.g. `lower_series_dd`, or cross-reference the two.
- The `near_mode.rs` module doc says "Excel is within 1.5 ulp of the true value there". Excel is 3.5 and 6.0 ulp off at
  t3252-gamma-prefactor-window A126 `POISSON.DIST(100,130,TRUE)` and A137 `POISSON.DIST(200,260,TRUE)`. Name those
  cells in the doc.
- statistics.md "Incomplete gamma: accurate kernel" and T-4000's Outcome say "No cell's verdict changes". It should
  read "no pass/fail change": A137 went from numeric_ulp to numeric_close, and t1048-poisson A9, B9 and t3252 window
  A125, A136 went from numeric_close to numeric_ulp.

## Acceptance criteria
- [ ] A probe case at non-integer shapes 20 to 21 near the mode, filled by the oracle. The band start is kept or moved
      on its evidence, with no gated cell regressing, and the result is recorded in statistics.md.
- [ ] The rename and the two wording fixes are in.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA.DIST/CHISQ.DIST near the mode (T-3252's open form).
