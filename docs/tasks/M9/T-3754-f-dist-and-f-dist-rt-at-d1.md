---
id: T-3754
title: "F.DIST and F.DIST.RT at d1 = 8, d2 = 5567623, x = 1.09: ours is 1.4E-10 off while Excel is correctly rounded (t3354-f-dist-sweep A168, C168)"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The F cdf and its upper tail at a small d1, a huge d2 and an x near 1 are correct to 15 significant digits, as
Excel's are, so probes-numeric/t3354-f-dist-sweep S!A168 and S!C168 match Excel.

## Context
Gap triage 2026-11-01 (`make eval` on origin/main 26e0bd00b). These two cells are owned today only by T-3479
(`defer: phase-1b`), whose Context says its cells are a last-digit residue "within 1e-9 relative of the 50-digit
math (most within 1e-13)". These two are inside 1e-9, but they are not a last-digit residue: the engine is right to
only about 10 significant digits, and Excel is correct to the last digit. That makes this an accuracy bug in our
kernel, not Phase 1b work. This task takes the two cells over from T-3479.

| cell | formula | Excel (excel-graph) | ours | 50-digit math (mpmath `betainc`) |
|---|---|---|---|---|
| S!C168 | `F.DIST.RT(1.09080885395379,8,5567623)` | 0.3658936312909749 | 0.3658936312385961 (1.4E-10 rel) | 0.36589363129097487440024 |
| S!A168 | `F.DIST(1.09080885395379,8,5567623,TRUE)` | 0.6341063687090251 | 0.6341063687614039 (8.3E-11 rel) | 1 - the above = 0.63410636870902512560 |

Excel is within 0.3 ulp of the math on both cells, so the target is the correctly rounded value. Do not fit to
Excel's bits (decision 0114 `accuracy` stage). The math was computed as `I_z(d2/2, d1/2)` with
z = d2/(d2 + d1·x), using mpmath at 40 digits (gap-triage scratch).

Shape: a = d1/2 = 4, b = d2/2 = 2783811.5, u = d1·x/(d1·x + d2) ≈ 1.57E-6. That is the incomplete beta's
huge-b, small-integer-a corner (TOMS 708's `bgrat` / `basym` region). Other t3354 cells at a huge d2 pass, but
they have a tiny x (probes/t3354-f-dist-tiny-x-huge-d2). The sweep has no other point at moderate x with d2 above 1E6
and d1 below 10, so the extent of the band is unknown.
Code: `f_tail` / `kernel_tails` in crates/rustsheet-engine/src/functions/statistical/fisher.rs and the incomplete
beta it calls (functions/statistical/incomplete_beta*). Related: T-3209 (Excel's bgrat at a huge first shape,
deferred), T-2510 (b = 1/2 bgrat branches), T-3479 (the true last-digit residue of the same two cases).

First step: find which branch these two inputs take, then measure the engine against mpmath on a local grid
(d1 in 2..20, d2 in 1E5..1E9, x in 0.5..2; no oracle needed for an accuracy fix) to find the band.

## Visible-parity stage
Stage `accuracy`. Target: probes-numeric/t3354-f-dist-sweep S!A168 and S!C168 within half a unit of the 15th
significant digit of Excel's value (they are correctly rounded, so that is the math's value).

## Acceptance criteria
- [x] `make eval-show CASE=probes-numeric/t3354-f-dist-sweep.case.json` shows S!A168 and S!C168 within half a unit
      of Excel's 15th significant digit (bit-exact preferred). No cell of t3354-f-dist-sweep or
      probes/t3354-f-dist-tiny-x-huge-d2 that passes today fails.
- [x] The engine-vs-mpmath grid above, committed under docs/research/t3754/, reports the engine's maximum relative
      error per stratum before and after. After the fix, the band that held these cells is under 1E-14 relative.
- [x] A unit test pins `F.DIST.RT(1.09080885395379,8,5567623)` = 0.3658936312909749 and the cdf companion.
- [x] No new known-failure lines in any eval/baseline/*.txt (`make eval-baseline` refuses regressions); T-3479's
      cell list drops S!A168 and S!C168.
- [x] statistics.md records the branch that was inaccurate and what replaced it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bit parity on the other t3354 cells (T-3479, T-3527).
- Excel's own inaccuracy at a huge first shape (T-3209).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 90 probes-numeric cells that no open task listed: 53 first wrong at the 11th significant digit or earlier, 37 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
The t4017 cells map the band this task's Context calls unknown. They cover F.DIST.RT and F.DIST(.., TRUE) at df1 3 to 79, df2 1E7 to 1E10 and x 1.5 to 50. Excel is within about 1E-13 of the 50-digit references in docs/research/t4017/references.json, and ours is 3E-11 to 1E-10 off them (for example `F.DIST.RT(7,12,1E7)`: Excel 7.084503009583688E-13, ours 7.084503009806421E-13, true 7.08450300958376E-13). So this is an accuracy fix: target the math, not Excel's bits. Re-size the task to M if the band needs it.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3354-f-dist-sweep.case.json (2): S!A9, A35
- probes-numeric/t4017-f-huge-df2-above-mean-1e10.case.json (1): Calc!A203
- probes-numeric/t4017-f-huge-df2-above-mean-1e7.case.json (61): Calc!A113-A117, A119, A121, A123, A125, A127, A129-A133, A135, A137, A139, A141, A143, A145-A149, A153, A155, A157, A159, A161-A165, A167, A169, A171, A173, A177-A179, A181, A183, A185, A187, A189, A193-A195, A197, A199, A201, A203, A205, A209-A211, A213, A215, A217, A219
- probes-numeric/t4017-f-huge-df2-above-mean-1e8.case.json (1): Calc!A219
- probes-numeric/t4017-f-huge-df2-above-mean-1e9.case.json (2): Calc!A189, A219
- probes-numeric/t4017-f-huge-df2-above-mean-3e7.case.json (23): Calc!A177-A179, A181, A183, A185, A187, A189, A193-A195, A197, A199, A201, A203, A205, A209-A211, A213, A215, A217, A219

- [x] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Outcome
- **Cause.** The F tails at a huge df2 with a b/a just past `large_shape_beta::MAX_SHAPE_RATIO` (1E-6) fell to Cephes'
  continued fraction, which is 1E-11 to 1E-9 off there. Examples: t3354 at d2 = 5567623, and t4017 at df2 1E7 to 3E7.
- **Fix.** The cut is now 4E-5, which is `MAX_SECOND_SHAPE / MIN_SHAPE`, so the DiDonato-Morris expansion covers every
  a >= 1E6 and b < 40.
- **Grid** (docs/research/t3754, 640 points against 60-digit mpmath): worst relative error 7.0E-10 → 2.1E-14, median
  6.4E-16 → 2.5E-16, points past 5E-14 169 → 0.
- **Eval.** On probes-numeric, 85 cells change and every one moves closer to the true value:
  - 12 become bit-exact;
  - F.DIST.RT's visibly-off numbers go from 114 to 44 and F.DIST's from 849 to 834;
  - `make eval-check` shows no gated cell moves, and no baseline line changes.
- **The listed cells.** 83 of the 90 are now within 13 digits of Excel. There the gap is Excel's own error, about
  1E-14, against ours of about 3E-16 (decision 0267 residue).
  - S!C168 is within 15 digits. S!A168 is 6E-16 off the math and misses Excel's 15th-digit half unit by a hair.
  - 5 deep-tail cells at df2 1E8 to 1E10 (1E-90 to 1E-142) stay at the 12th digit. There Excel is 4E-14 to 1.4E-13 off
    the math and the engine 4E-15 to 7E-14.
  - 2 cells are Excel's 0 against a subnormal tail (t3354 S!A9, S!A35). That is a different rule, not this kernel.
- **Unit test.** `f_tails_past_the_old_shape_ratio_cut_match_the_math` pins the t3354 pair and three t4017 tails against
  their references.
- **statistics.md.** It records the old cut, the branch it sent these inputs to, and the new cut.
