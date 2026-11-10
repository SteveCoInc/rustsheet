---
id: T-4908
title: "F lower tails and BETA.DIST below the mean at a huge b0: unswapped bfrac 1E-15 to 5E-15 off the math where Excel is on it (T-4435 follow-up, 11 t4434 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The 11 cells of probes-numeric/t4434-huge-shape-outside-large-shape-beta that T-4434 gave to T-4435 are within half a
unit of Excel's 15th significant digit. T-4435's change does not reach them, because bratio's a0 there is the
moderate shape (`F.DIST` below the mean, `BETA.DIST(x,a,b)` with a of 20 to 200 below a/b) or they are within an ulp
in the old large-shape region.

## Context
- T-4434's owner rule (`docs/research/t4434/score.py` `owner`) sends a cell to T-4435 wherever the correctly rounded
  math is within 15 digits of Excel. It does not test which kernel answers. After T-4435 (60-digit math, `score.py`),
  18 of its 29 cells match Excel. These 11 do not:
  - **8 on unswapped bfrac**, with a0 = df1/2 or a of 70 to 135 and b0 = df2/2 or b huge. Excel is within 1.4E-15
    of the math there, and ours is 7.7E-16 to 5.0E-15 off it:
    - B143 `F.DIST.RT(0.93301882157,140,310677000)`, ours 7.7E-16;
    - C14 `F.DIST(0.791767893144,151,15040300,TRUE)`, 2.8E-15;
    - C19 3.4E-15, C40 1.2E-15 (Excel 2.4E-15) and C42 1.6E-15;
    - D85 `BETA.DIST(8.2014509351E-08,72.022,642263000,TRUE)`, 9.2E-16;
    - D108 5.0E-15;
    - D129 2.5E-16.

    This is TOMS's own recurrence with Excel's density prefactor, outside T-3571's Lentz region (b above 5000).
    T-4435's region-2 route only replaces bfrac where a0 itself is huge.
  - **3 in the old large-shape region**: D32, D59 and D95, `BETA.DIST` with b of 4E7 to 1E9 above the mean (L140, then
    the expansion as T-4119 left it). Ours is within 2E-16 of the math and Excel within 5.1E-16. They differ by 5.7E-16 to
    7.1E-16 at values near 0.94, just past the 15th digit's half unit. That is Excel's last bit (a Phase 1b residue
    question), not an accuracy gap.
- Cells: probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json (11): Calc!B143, C14, C19, C40, C42, D32, D59, D85, D95, D108, D129

## Acceptance criteria
- [ ] The 8 unswapped-bfrac cells are within half a unit of Excel's 15th digit, from an accuracy fix to that branch at
      a huge b0 (decision 0114 `accuracy`; Excel is within 1.4E-15 of the math there). Or the task records why bfrac's
      digits there are Excel's own and blocks as Phase 1b.
- [ ] D32, D59 and D95 are resolved or recorded as Phase 1b residue (decision 0267: first wrong at the 16th digit).
- [ ] No cell of T-4434's case or of t3571-r5-bfrac-* moves away from Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The swapped bfrac calls at a huge a0 (T-4435) and Excel's own bfrac digits (T-4214).
