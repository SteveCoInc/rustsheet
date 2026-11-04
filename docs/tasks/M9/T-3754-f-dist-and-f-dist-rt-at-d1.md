---
id: T-3754
title: "F.DIST and F.DIST.RT at d1 = 8, d2 = 5567623, x = 1.09: ours is 1.4E-10 off while Excel is correctly rounded (t3354-f-dist-sweep A168, C168)"
milestone: M9
lane: functions-stat
status: done
priority: P2
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

## Resumed (2026-11-04)
An opus run resumed this claim-only branch (idle since 05:51 UTC) under the routine's step 2.

## Outcome
- Branch: bratio's L140 bup+bgrat (a0 = d2/2 after the swap, b0 = d1/2 from 2 up, x0 > 0.7). The port has it
  (`bratio/bup.rs` `l140_bup_bgrat`, T-3568) but refused a0 above `MAX_BUP_A0` = 3000, so these calls fell back to the
  Cephes kernel, which is 1.6E-12 to 3.7E-10 relative off the math for d1 4 to 20 and d2 1E5 to 1.5E7. The cap is now
  1E7. It is a range limit read off the accuracy grid against 50-digit math, not a score against Excel's bits, so no
  `FITTED` tag; above 1E7 the cap stays for T-3569's reason (t2528 A4 and A16 at a0 = 5E9).
- docs/research/t3754: `grid.py` (2,016 cells, mpmath expectations) and README with the before/after table per stratum:
  largest error 3.7E-10 before, 8.0E-15 after, every stratum under 1E-14.
- S!C168 is 0.36589363129097496 and S!A168 0.634106368709025: each 1 ulp from Excel's correctly rounded value, within half
  a unit of its 15th digit. Not bit-exact; the unit test `huge_df2_small_df1_takes_l140_bup_bgrat` pins both within
  15 digits and 1 ulp.
- Eval delta: gated suites unchanged (`rustsheet-eval check`: 0 fixed, 0 regressions). Ungated, over probes-numeric,
  probes, gen-m9 and ironcalc-calc: 52 cells move into 15 digits of Excel and none leaves (t3354-f-dist-sweep S!A168,
  S!C168; t4017-f-huge-df2-above-mean-1e7 46; t1538-f-inv-huge-df 2; t1538-f-quantile-switchover 2). 35 cells become
  bit-exact; 3 lose bit-exactness by 1 ulp, still within 15 digits (t4017 ...-1e7 A37, A97, A98; T-3209/T-4018's case).
  probes/t3354-f-dist-tiny-x-huge-d2 unchanged.
- T-3479's list drops S!A168 and S!C168; T-3209's evidence notes the 46 t4017 cells.
