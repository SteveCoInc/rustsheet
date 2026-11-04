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

## Outcome
- Branch found: bratio's L140 bup+bgrat at a0 = 2783811.5, b0 = 4 is past the port's a0 cap (3000), so
  `beta_tails` answered, and the large-shape expansion (`large_shape_beta.rs`) was skipped because b/a = 1.4E-6 was
  above its 1E-6 cut: Cephes' continued fraction with `large_prefactor` answered, right to about 10 digits.
- Fix: the expansion now applies from a = 1E4 (was 1E6) with b/a up to 1E-3 (was 1E-6), and its Q(b, z)/r is
  Legendre's fraction from z = 0.9 b up (was b + 1; `gamma_ur` there was up to 2.4E-15 off and left the task's cell
  5 ulp off). Both cuts are accuracy cuts against the math (40/50-digit mpmath), not fits to Excel's bits.
- Grid (docs/research/t3754, d1 2..20, d2 2E4..1E9, x 0.5..2): the d1 <= 10 band went from up to 4.5E-10 to
  9.8E-15 relative at most; d1 12..20 from 6.1E-10 to 3.4E-14 (the points above 1E-14 are x = 1 exactly, bratio's
  ported bfrac, unchanged; T-3479).
- Unit test `f_tails_at_a_small_df1_and_a_huge_df2_are_correctly_rounded` pins both cells bit-exact.
- Eval delta (probes-numeric, not baseline-gated): 518380 -> 518395 passing cells (16 fixed: S!A168, S!C168,
  t1538-f-quantile-switchover A3, 13 t4017 F.DIST cells at d2 = 1E7/3E7; 1 newly off by ulps:
  `F.DIST.RT(1.2128860145816052,5,1E10)`, t1538-beta-dist-at-excel-inv A23, where Excel is 4.0 ulp off the math and
  ours now 1.0, recorded in statistics.md). Every other changed t4017 cell is closer to its 120-digit reference.
  Gated suites: no change (`make eval-check`: 0 regressions, 0 fixed). T-3479 drops S!A168 and S!C168.
