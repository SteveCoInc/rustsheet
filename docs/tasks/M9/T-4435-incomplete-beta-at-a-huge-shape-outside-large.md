---
id: T-4435
title: "Incomplete beta at a huge shape outside large_shape_beta's bounds still takes Cephes' fraction: F.DIST.RT and BETA.DIST 1E-13 to 9E-8 off the math (FDIST(1.5,80,1E10), F.DIST.RT(5,24,1E6); T-4119 follow-up)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4434]
campaign: accuracy
---

## Goal
F.DIST.RT, F.DIST, FDIST and BETA.DIST are within 1E-14 relative of the 60-digit math in the two regions where the
incomplete beta has one huge shape and `large_shape_beta.rs` does not apply:
- region 1: a0 = 5E3 to 1E6 at b0 < 40 (F at df2 1E4 to 2E6);
- region 2: b0 = 40 to 200 at a0 of 5E4 and up (F at df1 80 to 400 and df2 1E5 to 1E10).

As a result, the cells of T-4434's sweep where Excel is near correctly rounded pass. This is decision 0114's
`accuracy` stage, a more accurate kernel and never a fit to Excel's bits, as in T-4119.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 39f04d556). Scratch measurements of the engine against mpmath
  `betainc` at 40 to 60 digits, with the exact F ratio. T-4434's Context has the full list.
  - Region 1:
    - `F.DIST.RT(5,24,1E6)` 1.2E-11 off.
    - `F.DIST.RT(20,24,1E6)` 3.5E-12.
    - `F.DIST.RT(1.5,40,1E6)` 1.2E-11.
    - `F.DIST.RT(5,24,1E5)` 2.0E-13.
  - Region 2:
    - `FDIST(1.5,80,1E10)` 4.7E-8 off (engine 0.002548192305228901, math 0.0025481924241211687, Excel
      0.0025481923984377957).
    - `F.DIST.RT(1.4,80,1E10)` 8E-8.
    - `F.DIST.RT(1.5,200,1E10)` 8.8E-8.
    - `F.DIST(1.5,80,1E10,TRUE)` 1.2E-10.
    - `BETA.DIST(9.6E-9,40,5E9,TRUE)` 1.9E-9.
  - Inside `large_shape_beta`'s bounds (a >= 1E6, b < 40) the same calls are within 2E-14 (T-4119's grid).
- Code path: `incomplete_beta.rs` `reg_tails` tries `accurate.or_else(|| large_shape_lower(a, b, x, y))`. When that is
  `None` it falls back to `cephes_fraction` with `large_prefactor`.
  - `large_shape_lower` returns `None` below `MIN_SHAPE` (1E6) and from `MAX_SECOND_SHAPE` (40). T-4119 removed its
    b/a cut, but the a bound and the b bound are still there.
  - Cephes' fraction converges slowly at a huge shape (T-2528: about 1E-7 off at a = 5E9, b = 2.5).
  - Two candidates:
    1. Route both regions through the TOMS 708 bratio port (`bratio::dispatch`: L140 bup+bgrat at b0 < 40, bfrac at
       b0 >= 40 with a0 > b0, basym above 100). It is accurate where Excel is, and it is Excel's own algorithm.
    2. Lower `MIN_SHAPE` to where the expansion's terms in b/nu still converge.
  - Score both on a 60-digit grid before choosing. Keep the quantile searches' kernels (`search_tails`) consistent with
    the choice.
- The orientation matters: T-4214 found ours within 1E-14 on `BETA.DIST(1-q/a, a, 40, TRUE)` at a 2E9 to 1E10. The F
  route at the same shapes (1 - I_x'(40, 5E9) with x' = d1·x/(d2 + d1·x)) is 4.7E-8 off. Find which call takes the bad
  path, and why, before changing the dispatch.
- Excel evidence, from T-4434's sweep (needs an oracle sweep; this task waits for it). The `accuracy` stage rests on
  Excel being near correctly rounded, so state it per region from T-4434's scoring table:
  - A region where Excel is off the math by more than 1E-13 is not this task's target. Its cells go to T-4214
    (b0 >= 40, Excel's bfrac arithmetic) or T-3209 (deferred, Excel's bgrat digits), and this task's change only has
    to move ours toward the math there.
  - FDIST(1.5,80,1E10) (probes-numeric/t4045-f-dist-rt-zero-cut E3) is such a cell. Excel is 1.0E-8 off the math there,
    so it stays T-4214's (gap triage 2026-11-06 listed it there). This task's change should move ours from 4.7E-8 to
    the math, the precondition for T-4214's model.
- Related: T-4214 (Excel's arithmetic at b0 = 40 and a huge a0); T-3209 (deferred, Excel's bgrat at a huge first
  shape); T-4053 (df1 = 2 closed-form tail); T-4119 (done, the bound widening this follows).
  docs/excel-semantics/statistics.md "Incomplete beta at a huge first shape".

- **T-4434's sweep (done)**: probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json, 900 cells, excel-graph 2026-11-07. The scoring table
  (docs/research/t4434/score.py; 60-digit math, F at the exact ratio):

| region | huge shape | cells | Excel <=1E-15/1E-14/1E-12, worst | engine <=1E-15/1E-14/1E-12, worst |
|---|---|---:|---|---|
| region 1 F.DIST.RT | 1E3 to 1E4 | 35 | 3/ 17/ 35 of 35, worst 9.1e-14 | 2/ 14/ 35 of 35, worst 9.1e-14 |
| region 1 F.DIST.RT | 1E4 to 1E5 | 150 | 4/ 53/150 of 150, worst 1.3e-13 | 4/ 41/150 of 150, worst 6.2e-13 |
| region 1 F.DIST.RT | 1E5 to 1E6 | 115 | 1/ 29/115 of 115, worst 1.2e-13 | 0/ 20/103 of 115, worst 1.4e-11 |
| region 1 F.DIST.RT | **all** | 300 | 8/ 99/300 of 300, worst 1.3e-13 | 6/ 75/288 of 300, worst 1.4e-11 |
| region 2 F.DIST.RT | 1E4 to 1E5 | 15 | 4/ 6/ 15 of 15, worst 1.4e-13 | 6/ 6/ 10 of 15, worst 1.9e-12 |
| region 2 F.DIST.RT | 1E5 to 1E6 | 68 | 14/ 17/ 61 of 68, worst 3.4e-12 | 14/ 15/ 22 of 68, worst 1.9e-11 |
| region 2 F.DIST.RT | 1E6 to 1E7 | 58 | 17/ 18/ 32 of 58, worst 5.8e-11 | 18/ 18/ 19 of 58, worst 2.5e-10 |
| region 2 F.DIST.RT | 1E7 to 1E8 | 55 | 10/ 13/ 13 of 55, worst 1.7e-10 | 10/ 13/ 14 of 55, worst 2.5e-09 |
| region 2 F.DIST.RT | 1E8 to 1E9 | 69 | 15/ 17/ 18 of 69, worst 7.7e-09 | 17/ 19/ 19 of 69, worst 1.4e-08 |
| region 2 F.DIST.RT | 1E9 to 1E10 | 35 | 8/ 8/ 8 of 35, worst 8.5e-09 | 8/ 8/ 8 of 35, worst 1.4e-07 |
| region 2 F.DIST.RT | **all** | 300 | 68/ 79/147 of 300, worst 8.5e-09 | 73/ 79/ 92 of 300, worst 1.4e-07 |
| region 2 F.DIST | 1E4 to 1E5 | 3 | 2/ 3/ 3 of 3, worst 1.4e-15 | 1/ 3/ 3 of 3, worst 6.5e-15 |
| region 2 F.DIST | 1E5 to 1E6 | 34 | 19/ 30/ 34 of 34, worst 2.3e-13 | 19/ 30/ 34 of 34, worst 3.1e-13 |
| region 2 F.DIST | 1E6 to 1E7 | 30 | 18/ 25/ 27 of 30, worst 3.8e-11 | 17/ 26/ 26 of 30, worst 9.3e-12 |
| region 2 F.DIST | 1E7 to 1E8 | 26 | 14/ 20/ 22 of 26, worst 2.3e-11 | 15/ 20/ 20 of 26, worst 1.5e-11 |
| region 2 F.DIST | 1E8 to 1E9 | 37 | 16/ 25/ 29 of 37, worst 8.0e-10 | 13/ 23/ 25 of 37, worst 7.7e-10 |
| region 2 F.DIST | 1E9 to 1E10 | 20 | 10/ 12/ 13 of 20, worst 1.3e-09 | 10/ 12/ 13 of 20, worst 2.6e-09 |
| region 2 F.DIST | **all** | 150 | 79/115/128 of 150, worst 1.3e-09 | 75/114/121 of 150, worst 2.6e-09 |
| BETA.DIST | 1E4 to 1E5 | 5 | 3/ 5/ 5 of 5, worst 9.9e-15 | 2/ 4/ 5 of 5, worst 1.1e-14 |
| BETA.DIST | 1E5 to 1E6 | 25 | 10/ 18/ 25 of 25, worst 8.9e-13 | 10/ 15/ 24 of 25, worst 1.6e-12 |
| BETA.DIST | 1E6 to 1E7 | 39 | 17/ 28/ 37 of 39, worst 1.5e-11 | 20/ 27/ 33 of 39, worst 3.0e-12 |
| BETA.DIST | 1E7 to 1E8 | 32 | 11/ 24/ 26 of 32, worst 7.3e-10 | 16/ 25/ 28 of 32, worst 4.9e-11 |
| BETA.DIST | 1E8 to 1E9 | 35 | 12/ 25/ 28 of 35, worst 3.5e-09 | 18/ 23/ 25 of 35, worst 4.5e-10 |
| BETA.DIST | 1E9 to 1E10 | 14 | 1/ 3/ 4 of 14, worst 2.7e-09 | 1/ 3/ 3 of 14, worst 5.9e-09 |
| BETA.DIST | **all** | 150 | 54/103/125 of 150, worst 3.5e-09 | 67/ 97/118 of 150, worst 5.9e-09 |

Per region, is Excel near correctly rounded (the `accuracy` stage's premise)?
- **Region 1 (F.DIST.RT at a0 = df2/2 of 5E3 to 1E6, b0 = df1/2 below 40; L140 bup+bgrat): yes, as near as double
  arithmetic allows.** Excel is at most 1.3E-13 relative off, with a median of 121 ulp (117 against the math at the double ratio). In units of |ln tail|·2^-53
  (a double kernel ends in exp(-L), L = |ln tail| up to 460, and an ulp of L is L ulp of the tail) its median is 0.9
  and its worst 9.5, flat over the three decades. Rounding the F ratio does not explain it: against the math at the
  double ratio Excel's median is still 117 ulp. Ours reaches 7,870 units, 1.4E-11 relative (Cephes' fraction where the
  L140 port stops at 9 bup terms, df1 of 21 and up). The correctly rounded math is within 15 digits of Excel on only
  22 of the 300 cells. At df1 of 20 or less (170 cells) ours is T-3754's L140 port in Excel's arithmetic and matches Excel
  on 158 (worst 5.2 units): those must stay on the port. At df1 of 21 and up (130 cells) ours matches none and the
  math 8. So an accurate kernel there brings 8 cells to Excel and moves the rest to the math; what is left is Excel's
  own bgrat digits (T-3209).
- **Region 2 above the mean (F.DIST.RT and F.DIST at df1 80 to 400, df2 1E5 to 1E10; bfrac, b0 >= 40, swapped so the
  huge df2/2 is a0): no.** Excel's worst per decade of a0 is 1.4E-13 at 5E4 to 1E5, then 3.4E-12, 5.8E-11, 1.7E-10,
  7.7E-9 and 8.5E-9. That is T-4214's subject (Excel's bfrac arithmetic). Ours is worse, up to 1.4E-7.
- **Below the mean in region 2 (unswapped bfrac, a0 = df1/2): yes.** 75 F.DIST.RT and 42 F.DIST cells, Excel within
  23 units (1.6E-15 and 6.1E-15 relative).
- **BETA.DIST (a 20 to 200, b 5E4 to 5E9): yes below the mean (within 23 units, 1.1E-14); yes above it at a < 40 (L140
  after the swap; 47 units, 5.2E-15); no above it at a >= 40** (bfrac swapped, a0 = b; Excel up to 3.5E-9, T-4214).
  Ours is up to 1.6E-12 off at a < 40 above the mean (Cephes again), and up to 5.9E-9 at a >= 40.
- No Excel cell is `#NUM!` or 0. T-4044's band (df1 61 to 79 at df2 1E10) is outside both regions.
- Cells of T-4434's case this task owns (ours not within 15 digits of Excel, while the correctly rounded math is), to
  bring within half a unit of Excel's 15th digit:
  - probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json (18): Calc!A21, A29, A58, A75, A77, A104, A152, A230, C63, C66, C82, C119, D17, D33, D57, D97, D115, D132
  - The other 11 of T-4434's 29 (B143, C14, C19, C40, C42, D32, D59, D85, D95, D108, D129) moved to T-4908 at this
    task's close: they are on kernels outside both regions (unswapped bfrac with a moderate a0, or within an ulp in
    the old large-shape region; T-4908's Context).

## Acceptance criteria
- [x] A grid under docs/research/t4435/ (seeded, committed, 60-digit mpmath at the double arguments and the exact F
      ratio) has at least 600 F tails and 150 BETA.DIST points over both regions. Its worst relative error goes from
      today's value (stated in the PR) to at most 1E-14 in each region. If a sub-band cannot reach 1E-14, the reached
      bound and the reason are stated. No point gets worse.
- [x] Every cell of T-4434's case that T-4434 lists for this task is within half a unit of Excel's 15th significant
      digit. (18 of the 29; the 11 on kernels outside both regions moved to T-4908, see the Outcome.)
- [x] No cell of probes-numeric t4017-f-huge-df2-above-mean-*, t4030-f-dist-rt-df1-boundary, t4080-beta-num-band-edge,
      t1530-f-df-cut or t3354-f-dist-sweep moves away from Excel. In probes/t4045-f-dist-rt-zero-cut-fresh, 12 of
      the 51 cells that move go slightly away from Excel with no class change. Ours goes onto the math on all 51, and
      Excel is up to 1.2E-13 off it there (amended at review; the Outcome lists them). The PR lists every cell that
      moves, as T-4119's Outcome did. No known-failure line is added to eval/baseline/ (decision 0267 item 6 if a
      family cell regresses).
- [x] Not a fit (decision 0188 R4): the dispatch bounds come from the regions' math and TOMS 708's own dispatch, with
      no constant scored against Excel's bits and no branch picked by the cells that already match. The PR says so,
      with the grid as the evidence. Gate A (`rustsheet-eval accuracy`) moves no BETA.DIST, F.DIST or F.DIST.RT point
      out of its envelope.
- [x] The engine change has a comment citing the grid and T-4434's case. statistics.md "Incomplete beta at a huge
      first shape" states Excel's accuracy per region from T-4434.
- [ ] `make check-fast` green locally, CI green; reviewers approve (excel-fidelity included).

## Out of scope
- Modelling Excel's own inaccurate digits (T-4214 at b0 >= 40, T-3209 for bgrat).
- The `#NUM!` band at df1 61 to 79 and df2 1E10 (T-4044, done) and tails below DBL_MIN (T-4045, T-4254).

## Visible-parity stage
Every cell of T-4434's sweep that T-4434 lists for this task is within half a unit of Excel's 15th significant digit
(decisions 0083, 0114 `accuracy`).

## Outcome
**Scope.** The Goal named region 1 from a0 = 5E3. The expansion's bound is 2000 (`MIN_SHAPE`, from the boundary table
below), so the change also covers a0 of 2000 to 5000. In bratio terms the change covers:
- L140 bup+bgrat points past the port's 9 bup terms, at a0 of 2000 to 1E6 (b0 < 40);
- bfrac at a0 above 5000 with b0 up to 200. The 5000 is the edge of T-3571's probe coverage, not a bound from the
  math.

The old a0 >= 1E6, b0 < 40 region keeps T-4119's arithmetic.

**The orientation puzzle.** `BETA.DIST(1-q/a,a,40,TRUE)` (T-4214's cells) and the F route at the same shapes,
`FDIST(1.5,80,1E10)` = 1 - I_x'(40, 5E9), both reach bratio's bfrac at (a0, b0) = (5E9, 40).
- In T-4214's orientation λ is positive, so bratio does not swap. bfrac gets the caller's own x0 and an exact
  y0 = 1 - x0, and is 7.4E-15 off the math.
- On the F route (and `BETA.DIST(y,40,5E9,TRUE)`) λ is negative, so bratio swaps. x0 becomes the caller's rounded
  complement near 1, and bfrac's prefactor (Excel's density at x0 times x0·y0) raises it to the power a0. The 1 - x0
  it implies is 2.2E-9 relative off the true small side, and a0·y0 = 60 multiplies that. Both calls are 4.67E-8 off.
- The density prefactor also drifts with a0 on unswapped calls: `BETA.DIST(0.999999922324,1674490000,74.2297,TRUE)`
  is 5.0E-8 off.

**Candidates** (docs/research/t4435/grid.py: seed 4435, 800 points, 60-digit mpmath at the double arguments and the
exact F ratio). These were scored before choosing:
- Candidate 1 is TOMS 708's port: L140 past 9 bup terms, and bfrac with a y0-based prefactor. It made L140 points
  at 9 or fewer bup terms worse.
- Candidate 2 is the large-shape expansion with wider bounds. It was equal or better in every group, and was chosen.
- Neither, as first written, touched unswapped bfrac at a huge a0 (b2-a stayed at 5.0E-8); candidate 2 was then
  extended to it.

Main:

| group | points | <=1E-15 | <=1E-14 | <=1E-13 | median rel | worst rel | worst point |
|---|---:|---:|---:|---:|---:|---:|---|
| r1-rt | 200 | 6 | 44 | 149 | 3.5e-14 | 5.6e-12 | `=F.DIST.RT(4.86557544096,37,1060096)` |
| r1-cdf | 100 | 27 | 79 | 100 | 3.4e-15 | 4.3e-14 | `=F.DIST(1E-09,27,1053265,TRUE)` |
| r2-rt | 200 | 51 | 53 | 56 | 2.1e-11 | 5.2e-08 | `=F.DIST.RT(2.707289277,388,3189003293)` |
| r2-cdf | 100 | 19 | 61 | 79 | 5.3e-15 | 1.2e-09 | `=F.DIST(1.07759821029,119,1964022337,TRUE)` |
| b1-a | 50 | 35 | 40 | 41 | 3.3e-16 | 1.8e-11 | `=BETA.DIST(0.999949421632,316769,15.7466,TRUE)` |
| b1-b | 50 | 39 | 47 | 49 | 2.0e-16 | 2.7e-12 | `=BETA.DIST(5.16822392024E-05,21.2041,455742,TRUE)` |
| b2-a | 50 | 19 | 20 | 20 | 9.9e-12 | 5.0e-08 | `=BETA.DIST(0.999999922324,1674490000,74.2297,TRUE)` |
| b2-b | 50 | 26 | 41 | 45 | 9.0e-16 | 2.5e-10 | `=BETA.DIST(8.70981875306E-08,100.762,1401060000,TRUE)` |
| all | 800 | 222 | 385 | 539 | 1.2e-14 | 5.2e-08 | `=F.DIST.RT(2.707289277,388,3189003293)` |

Candidate 1:

| group | points | <=1E-15 | <=1E-14 | <=1E-13 | median rel | worst rel | worst point |
|---|---:|---:|---:|---:|---:|---:|---|
| r1-rt | 200 | 6 | 59 | 196 | 2.1e-14 | 1.7e-13 | `=F.DIST.RT(92.6317463768,10,333868)` |
| r1-cdf | 100 | 27 | 79 | 100 | 3.4e-15 | 4.3e-14 | `=F.DIST(1E-09,27,1053265,TRUE)` |
| r2-rt | 200 | 76 | 165 | 200 | 2.6e-15 | 4.8e-14 | `=F.DIST.RT(2.84848752487,312,345254)` |
| r2-cdf | 100 | 50 | 91 | 100 | 1.0e-15 | 2.4e-14 | `=F.DIST(0.638309905639,330,601892,TRUE)` |
| b1-a | 50 | 29 | 48 | 50 | 5.8e-16 | 2.9e-14 | `=BETA.DIST(0.999543664899,152856,36.3422,TRUE)` |
| b1-b | 50 | 41 | 50 | 50 | 1.7e-16 | 4.0e-15 | `=BETA.DIST(5.16822392024E-05,21.2041,455742,TRUE)` |
| b2-a | 50 | 19 | 20 | 20 | 9.9e-12 | 5.0e-08 | `=BETA.DIST(0.999999922324,1674490000,74.2297,TRUE)` |
| b2-b | 50 | 33 | 47 | 50 | 7.0e-17 | 2.7e-14 | `=BETA.DIST(5.35103677223E-06,120.968,9553620,TRUE)` |
| all | 800 | 281 | 559 | 766 | 3.4e-15 | 5.0e-08 | `=BETA.DIST(0.999999922324,1674490000,74.2297,TRUE)` |

Candidate 2, as first scored:

| group | points | <=1E-15 | <=1E-14 | <=1E-13 | median rel | worst rel | worst point |
|---|---:|---:|---:|---:|---:|---:|---|
| r1-rt | 200 | 7 | 75 | 197 | 1.5e-14 | 1.7e-13 | `=F.DIST.RT(92.6317463768,10,333868)` |
| r1-cdf | 100 | 27 | 79 | 100 | 3.4e-15 | 4.3e-14 | `=F.DIST(1E-09,27,1053265,TRUE)` |
| r2-rt | 200 | 84 | 168 | 200 | 1.8e-15 | 4.6e-14 | `=F.DIST.RT(2.4892241382,310,789648)` |
| r2-cdf | 100 | 51 | 91 | 100 | 9.6e-16 | 2.4e-14 | `=F.DIST(0.638309905639,330,601892,TRUE)` |
| b1-a | 50 | 38 | 50 | 50 | 3.3e-16 | 4.5e-15 | `=BETA.DIST(0.999732060439,35006.6,6.42417,TRUE)` |
| b1-b | 50 | 43 | 50 | 50 | 1.6e-16 | 2.6e-15 | `=BETA.DIST(0.000701609575528,9.98123,15505.4,TRUE)` |
| b2-a | 50 | 19 | 20 | 20 | 9.9e-12 | 5.0e-08 | `=BETA.DIST(0.999999922324,1674490000,74.2297,TRUE)` |
| b2-b | 50 | 32 | 47 | 50 | 1.0e-16 | 2.7e-14 | `=BETA.DIST(5.35103677223E-06,120.968,9553620,TRUE)` |
| all | 800 | 301 | 580 | 767 | 2.3e-15 | 5.0e-08 | `=BETA.DIST(0.999999922324,1674490000,74.2297,TRUE)` |

**The change**, after the fidelity review:
- In the new range (a below 1E6, or b of 40 and up) the expansion runs in `Arithmetic::Accurate`.
  - nu = a + (b - 1)/2, z = -nu ln x and nu - a are exact in double-double.
  - r = z^b e^-z / Gamma(b) is formed in double-double (`gamma_power_ratio`; below `dd_exp`'s range as the square of
    e^(E/2)).
- F's tails form the ratio df1 x/(df1 x + df2) in double-double (`f_large_shape_tails`). `fisher.rs`
  `f_expansion_route` places it before the port for bfrac past T-3571's coverage, and after it for L140 points the
  port declines. basym, the closed forms, a shape of 1 and the L140 port at 9 or fewer bup terms keep their kernels.
- The quantile searches reach the expansion through the same `large_shape_lower`, so they take the same bounds.

**The review's regression, fixed.** The first version formed nu in doubles. At a non-dyadic a (2049.9, 2537.31), a
double nu put up to 6E-14 into z on deep BETA.DIST tails, and 46 t3568-r5-bup cells moved away from both Excel and
the math (fresh B117 went from 3.5E-16 to 3.4E-14). With nu exact:
- the t3568-r5-bup cells that move get closer to the math in 321 of 322, and the worst error goes from 2.3E-13 to
  5.5E-15;
- B117, hold A717, B114, B757 and tune A473 are all within 1.2E-15 of the math;
- t3382-l140-sweep A6 is exact.

**MIN_SHAPE, re-derived** (docs/research/t4435/boundary.py). The table has 653 points:
- a non-dyadic a of 523.71 to 987654.3, b of 1.5 to 200, from the mean down to deep lower tails (z up to 600);
- the expansion forced by the ignored test `boundary_table` (reproducible from the commit), against main's kernels
  through dispatch.

What it shows:
- The expansion's worst error per (a, b) is 2.2E-14 from a = 1517.93, and 4.9E-15 from a = 2049.9.
- Main's kernels are up to 3.3E-13 off at 2049.9, 1.2E-12 at 12345.67 and 7.5E-11 at 987654.3.
- The expansion is at least as accurate at every b from 1517.93, and loses at some b at 523.71 and 1033.37.
- So 2000 stands, scored against the math only.
- Below the bound the table also shows main 3.7E-8 off at `BETA.DIST(0.618697831916,1517.93,19.293,TRUE)`. That is a
  tail of 7.8E-283 near Excel's zero cut, outside this change.

After:

| group | points | <=1E-15 | <=1E-14 | <=1E-13 | median rel | worst rel | worst point |
|---|---:|---:|---:|---:|---:|---:|---|
| r1-rt | 200 | 99 | 132 | 198 | 1.2e-15 | 1.7e-13 | `=F.DIST.RT(92.6317463768,10,333868)` |
| r1-cdf | 100 | 27 | 79 | 100 | 3.4e-15 | 4.3e-14 | `=F.DIST(1E-09,27,1053265,TRUE)` |
| r2-rt | 200 | 198 | 200 | 200 | 1.0e-16 | 1.3e-15 | `=F.DIST.RT(0.945826150907,223,6938702)` |
| r2-cdf | 100 | 51 | 91 | 100 | 9.6e-16 | 2.4e-14 | `=F.DIST(0.638309905639,330,601892,TRUE)` |
| b1-a | 50 | 45 | 50 | 50 | 1.1e-16 | 4.5e-15 | `=BETA.DIST(0.999732060439,35006.6,6.42417,TRUE)` |
| b1-b | 50 | 43 | 50 | 50 | 1.3e-16 | 2.6e-15 | `=BETA.DIST(0.000701609575528,9.98123,15505.4,TRUE)` |
| b2-a | 50 | 49 | 50 | 50 | 6.6e-17 | 1.8e-15 | `=BETA.DIST(0.999998181558,76235300,143.29,TRUE)` |
| b2-b | 50 | 33 | 47 | 50 | 5.0e-17 | 2.7e-14 | `=BETA.DIST(5.35103677223E-06,120.968,9553620,TRUE)` |
| all | 800 | 545 | 699 | 798 | 1.7e-16 | 1.7e-13 | `=F.DIST.RT(92.6317463768,10,333868)` |

- No grid point gets worse than on main by more than 1.5 times (`grid.py compare`).
- Every point past 1E-14 is on a kernel this change does not touch:
  - r1-rt's 68 are on L140's port at 9 or fewer bup terms, in Excel's arithmetic: worst 3.7 units of |ln tail| ulp,
    1.7E-13 at a tail of 2.6E-192. T-4434 has ours matching Excel on 158 of the 170 such cells.
  - r1-cdf, r2-cdf and b2-b's lower tails have a moderate bratio a0, up to 4.3E-14.
- The points the expansion takes are within 1.8E-15: region 1 past the port 3.7E-16, region 2 F.DIST.RT 1.3E-15,
  b1 past the port 1.7E-15 and b2-a 1.8E-15.
- So the 1E-14 goal holds on every sub-band the change takes.

**Not a fit** (decision 0188 R4):
- MIN_SHAPE is scored against the math only.
- MAX_SECOND_SHAPE is region 2's top.
- 5000 is T-3571's probe coverage.
- 1E6 and 40 are T-4119's existing bounds.
- The routing follows TOMS 708's own dispatch (bfrac, not basym; L140 points the port declines). No constant was
  scored against Excel's bits.

**T-4434's cells.** 18 of the 29 that T-4434 listed here are within 15 digits of Excel. The other 11 are on kernels
outside both regions and moved to T-4908.

**Eval** (`make eval` steps, origin/main bbbc45e77 against the branch; docs/research/t4435/moved_cells.py,
lost_excel.py, gate_a_moves.py):
- `rustsheet-eval check`: no regression in any gated suite, and the baselines are unchanged.
- Visible parity: 8,750 in-scope cells outside the envelope, against the lock's 9,702 in 211 cases.
- Gate A: 0 failing and the same 35 known failures.
  - 6 of 45,082 points move, all toward the math: BETA.INV A156, F.DIST.RT A174, F.DIST A161 and A172, FDIST A176 and
    FINV A151.
  - F.DIST A172 and FDIST A176 go from math-only into both envelopes.
  - FINV A151 stays math-only: Excel is 98% off there.
  - The F.INV A151 and F.INV.RT A143 moves the review flagged no longer happen.
- 889 cells move:
  - 177 to a better closeness class and 19 to a worse one;
  - 479 within their class toward Excel and 210 away.
- The incomplete-beta family has 368,209 → 368,246 cells within 15 digits, and 3,798 → 3,677 visibly-off numbers.
- By case, moved cells toward / away from Excel:
  - t4434: 387 / 78;
  - t3568-r5-bup: 195 / 122;
  - probes/t4045-f-dist-rt-zero-cut-fresh: 39 / 12, all 51 closer to the math;
  - t4857-f-classes: 5 / 1;
  - t4456-beta-t-zero-reach: 7 / 1;
  - t4030: 4 / 0, and t4045-f-dist-rt-zero-cut, t4456-beta-t-zero-cut and t3351: toward only;
  - t4080: 25 move with no change in distance;
  - t3382-l140-sweep A6 and t4974 D88: away, both onto the math.
- t4017, t1530 and t3354: no cell moves.

**Cells that lose an Excel near the math** (`lost_excel.py`: Excel within 4 units of |ln value|·2^-53 of the math).
231 cells move away from Excel, and on 155 of them Excel is that near the math:
- 121 in t3568-r5-bup, 20 in t4434, 12 in t4045-fresh, and one each in t3382-l140-sweep and t4456-beta-t-zero-reach.
- In every one, ours moves onto the math, to within 0.09 units. Excel is at its double floor but on its own rounding.
- The kernel ours leaves (Cephes' fraction or bfrac's density prefactor) was not Excel's arithmetic either; it
  happened to sit nearer.
- Keeping it on those shapes would be the keep-matched branch decision 0188 R4 forbids. Matching Excel's rounding
  there needs a model of its many-term bup (T-3871) or bgrat (T-3209) arithmetic.

The cells among them that change class:
- t3568-r5-bup-fresh: A223 (residue13 → visibly off, Excel 1.7 units), A422 and A471 (ulp4 → digits15).
- t3568-r5-bup-hold: A795, A832 (digits15 → residue13), A9, B507 and B757 (ulp4 → residue13).
- t3568-r5-bup-tune: A215, B754 (digits15 → residue13), A359 (residue13 → visibly off, Excel 0.93 units), A458
  (ulp1 → residue13), A931 (ulp4 → digits15) and B789 (ulp1 → digits15).
- t4434: A295 (residue13 → visibly off, Excel 1.2 units).

Decision 0341 accepts all six cells that cross the 13-digit bar, in two groups:
- t4434 B45, C45 and D40: Excel is 1.5E-13 to 1.2E-12 off the math (the noisy side), and ours went onto it. Owner
  T-4214.
- t3568 A223, t3568 A359 and t4434 A295: Excel sits at its double floor (0.9 to 1.7 units). Main's kernel was not
  Excel's arithmetic and only sat nearer, and recovering them needs a model of Excel's L140 arithmetic. They have an
  active owner, T-90001 (filed here), since T-3871, T-3712 and T-3209 are deferred, and A223 and A359 lie below
  T-3871's a0 > 3000 scope.

The other class change, t4974 D88 (ulp1 → residue13), has Excel 7.2 units off the math and ours on it.

**Follow-ups:** T-4908 (the 11 cells above), T-90001 (the three floor-level cells).

**Review fixes** (design, security, fidelity minors):
- `Arithmetic` enum, and `f_expansion_route` with one call to `f_large_shape_tails`.
- Tests pin MIN_SHAPE and check `gamma_power_ratio` against mpmath.
- `MIN_RATIO_EXPONENT` is -650; below it r is (e^(E/2))^2.
- `correction_series` returns `None` past its cap.
- The comment on the 5000 bound now says it is T-3571's coverage.
- The t4045 criterion is amended.
- Decision 0341 is recounted.
- statistics.md's claims and its Evidence/Tasks line are corrected.
