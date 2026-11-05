---
id: T-2555
title: "Excel's BETA.INV early Newton stop where the quantile is tiny at a huge shape (T-1538 split)"
milestone: M9
lane: functions-stat
status: blocked
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Excel's BETA.INV (and F.INV/F.INV.RT through it) at a huge shape match Excel where the quantile is tiny
(a small first shape, a huge second one, p <= 1/2), e.g. `BETA.INV(0.025,2.5,5E9)` = 8.312123853710714E-11
and `F.INV(0.05,3,1E8)` = 0.11728210553634685.

## Context
Split from T-1538. Excel's BETA.INV is AS 109's structure (p > 1/2 is 1 - BETA.INV(1-p,b,a), bit for bit),
and where the quantile is near 1 it is the converged, correctly rounded root, which T-1538 implements
(`inverse_beta.rs` `beta_quantile_crossing`). Where the quantile is tiny (x near 1E-10 at b = 5E9) Excel's
value is near the second Newton iterate from AS 109's start (1E-12 to 4E-8 relative from ours) and up to 1E-6
relative off the true root (8 of the 24 BETA.INV cells of probes-numeric/t1538-f-inv-huge-df: A133, A135,
A137, A139, A145, A153, A155; A147 within 1 ulp). Neither a fixed acu (1E-26 to 1E-40), statrs' acu, nor
ln B from double lgamma reproduces it; the start formula, the density in the Newton step and the stop
need fitting (scratch analysis in the T-1538 PR). Owned visibly-off gen-m9 cells that stay after T-1538
are listed in its Outcome (those whose beta variable is tiny, e.g. gen-m9/146b46e74f2dac3b A131).

Owned gen-m9 cells (moved from T-1538, still visibly off after it): gen-m9/146b46e74f2dac3b A131,
7c20351562a2d71d A254 and A37, 85f48d5cdfb24555 A315, 589f42e67f4b4b8d A319, 8dd2312c96effb1b A169,
f34f66fb7f81e673 A209. A37, A319 (`F.INV(1E-200,...)` at a huge df) and A169 (`BETAINV(1E-100,123456789.123,0.1,...)`)
are where Excel's search fails and its value is not the root (statistics.md, UNVERIFIED); A209 has b = 1 and a root
almost halfway between two doubles, Excel taking the far one (a tie its own exp/ln rounding decides?). Cell list for the owner check (the stem alone is shared with a probes/t2436 rerun):
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A209
Also watch:
probes-numeric t1538-f-inv-huge-df A86 (0.53 units, just past 15 digits after T-1538) and
t1538-f-quantile-switchover (all 24 cells within 2E-13 relative, none bit-exact, either side of df 1E5).

Cell-list line (gap triage 2026-10-31i: the bare hash above does not name the case to `scripts/exit_owners.py`,
which needs `gen-m9/<hash>` because the hash is not unique in the report, so A209 read as unowned; the line only
restates ownership). `F.INV.RT(Data!E7,2,(Data!$C9+Data!E2))` = `F.INV.RT(0.5,2,123456789.1229)`: Excel
0.6931471803615475, ours 0.6931471872147749; with d1 = 2 the root is (d2/2)(2^(2/d2) - 1), about 0.69314718445
(mpmath), so Excel is 6E-9 relative off it and ours 4E-9.
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A209

Also owned (gap triage 2026-11-04). These are the df 1E10 quantiles of the T-1530 probe. Until now they were named
only by the watch lists and no-regression criteria of T-2642, T-3209 and T-3766, and none of those tasks fixes
them. All three go through Excel's `BETA.INV(0.5,2.5,5E9)`, the tiny-quantile Newton stop this task fits
(statistics.md "F quantile at a huge df goes through Excel's BETA.INV"; reciprocal identity
`F.INV.RT(p,d1,d2)` = 1/`F.INV(p,d2,d1)`). In `F.INV(0.5,5,1E10)`, Excel is 0.8702920369171896 and ours is
0.8702920382779128, the true quantile (Excel is 1.6E-9 relative off). In `F.INV.RT(0.5,1E10,5)` = `FINV(0.5,1E10,5)`,
Excel is 1.1490395839335394 and ours is 1.1490395821369874.
- probes-numeric/t1530-f-df-cut.case.json (3): Calc!A2, A3, A5

## Visible-parity stage
Decision 0083: the cells above are visibly off; the target is that each is within half a unit of the 15th
significant digit of Excel's value, or recorded as a DEVIATION where Excel's value cannot be derived.

## Acceptance criteria
- [ ] Visible parity: the 10 owned cells above (7 gen-m9, 3 t1530-f-df-cut) reach 15-digit parity or are DEVIATION/UNVERIFIED entries with evidence.
- [ ] The 8 small-x BETA.INV cells of t1538-f-inv-huge-df are within half a unit of the 15th digit, or the
      stop rule is recorded as UNVERIFIED with the hypotheses ruled out.
- [ ] No regressions in any baseline; `make check` green; reviewers approve.

## Out of scope
-

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 30 probes-numeric cells that no open task listed: 23 first wrong at the 11th significant digit or earlier, 7 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t1538-f-inv-huge-df.case.json (30): Calc!A1, A4, A15, A18, A23, A26, A37, A40, A42-A43, A46-A47, A60-A61, A63, A66, A111, A114, A119, A122, A125, A128-A129, A132-A133, A135, A137, A139, A153, A155

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Attempts (decision 0056 time-box)
- Attempt 1 (2026-11-05, opus run; docs/research/t2555/as109_replay.py): AS 109 (R's old `qbeta`, AS R19's acu
  `max(fpu, 10^(-13 - 2.5/pp^2 - 0.5/alpha^2))`, the Majumder-Bhattacharjee start, the step-thirding loop, return
  of the iterate before the last step or after it) replayed in doubles on an mpmath CDF rounded to a double, on the
  8 tiny-quantile cells A133, A135, A137, A139, A145, A147, A153, A155. Moved no cell. Ruled out:
  - every stop rule over these iterates: Excel's value is no iterate. It is nearest iterate 2 (A133 1E-12, A135
    2E-13 relative with log1p; A137 4E-8, A139 2E-8), and A145 and A147 sit at full convergence (6E-16 and 0, so
    only A147 is exact). A153 and A155 lie 1E-8 from every iterate past the first;
  - acu fixed at 1E-13 to 1E-30 and the R19 formula, each with the old or the new iterate returned: at best 1 of 8
    cells (A147) within an ulp;
  - ln B exact or from double lgamma (1.2E-5 off at (2.5, 5E9)), and ln(1 - x) as log(1 - x) or log1p;
  - a constant error in ln B: the offset that puts iterate 2 on Excel's value is -1.6E-8 at A133, +3.3E-5 at A137 and
    +1.0E-3 at A145, all at the shapes (2.5, 5E9), and no offset within 1E-3 reaches A153 or A155.
  The next attempt should change the CDF that the search evaluates (Excel's own BETA.DIST at each iterate, e.g. its
  value at iterate 1, which is 1E-3 to 5E-2 off the root, where an inaccurate tail would move iterate 2 by about
  what the cells show) or the start, not the stop. A second attempt that moves nothing blocks the task as time-boxed
  under decision 0267.
- Attempt 2 (2026-11-05, opus run; docs/research/t2555/gen_iterates_probe.py, iterates.py, excel_cdf_probe.*.json):
  Excel's own BETA.DIST at the replay's start and first iterates x0 to x3 of the 8 cells (31 cells, excel-graph
  2026-11-05). Moved no cell. Ruled out: Excel's CDF as the cause. It is within 2E-14 relative of the mpmath CDF at
  every iterate, and a Newton step taken on Excel's own value lands where the exact one does: 1E-12 (A133), 2E-13
  (A135) and 1E-12 (A145) from Excel's BETA.INV at one iterate, but 4E-8 to 9.3E-7 from it at every iterate of A137
  and A139 and 1E-8 at A153 and A155. Excel's value lies between iterate 2 and the root of its own CDF (for A137,
  4 percent of the way from x2), so it is on no trajectory of AS 109 from this start, whatever its stop and CDF.

## Blocked because
Blocked because: time-boxed (decision 0056). Two attempts (Attempts above) moved none of the owned cells. Under
decision 0267 the cells this task still lists are accepted, and no follow-up is filed for them. What is left to try
is another start or another search (a secant or bisection that ends inside the last Newton bracket), against the
8 cells and the 31 CDF values in docs/research/t2555; statistics.md "BETA.INV at a huge shape is AS 109's Newton
search" records the ruled-out models.

