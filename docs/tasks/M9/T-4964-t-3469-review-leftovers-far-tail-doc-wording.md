---
id: T-4964
title: "T-3469 review leftovers: far-tail doc wording and the R4.1 note"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3469]
---

## Goal
The MINOR findings the reviewers left on PR #3905 (T-3469, approved at 8cb776bae) are fixed, so the far-tail docs say
what the code does.

## Context
- `incomplete_gamma/excel_tail.rs` module doc (lines 6-9) still calls the prefactor "Stirling's ratio form" right
  before saying its exponent B is not the ratio form: say "Stirling's prefactor".
- `FAR_TAIL_MIN_X`'s doc (about lines 29-31) still says "the x above which Excel takes the ratio form"; the counts in
  `FAR_TAIL_MAX_SHAPE`'s doc (80/12 and 1/3, about lines 36-42) were scored on the ratio form: reword, or say they
  were not re-scored with the direct deviance.
- docs/excel-semantics/statistics.md, "Excel's far upper tail (T-3269)": "Above shape 11 Excel is not this form ...
  (where it switches there is open, T-3469)" should name T-3640, the owner of that question; re-wrap the over-long
  lines of the "Correction (T-3469)" bullet.
- docs/tasks/M9/T-3469-excel-s-far-tail-q-at-shape-6.md Outcome, R4.1 bullet: state the comparison with the correctly
  rounded Q (test bit-exact 44 vs 24 of 266) and the 5-point train/test margin check (20.8% against 16.5%), both now
  only in docs/research/t3469/holdout.out, and note that three hold-out test cells (t3269-shape6-misses A46,
  t3284-gamma-deep-upper-tail A165 and A166) drove the choice of formula; tick its last acceptance box.

## Acceptance criteria
- [ ] Each point above fixed (wording only; no code behavior change).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-scoring the FAR_TAIL_* cuts (T-3640).
