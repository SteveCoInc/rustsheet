---
id: T-4435
title: "Incomplete beta at a huge shape outside large_shape_beta's bounds still takes Cephes' fraction: F.DIST.RT and BETA.DIST 1E-13 to 9E-8 off the math (FDIST(1.5,80,1E10), F.DIST.RT(5,24,1E6); T-4119 follow-up)"
milestone: M9
lane: functions-stat
status: in-progress
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
| region 1 F.DIST.RT | 5E3 to 1E4 | 35 | 3/ 17/ 35 of 35, worst 9.1e-14 | 2/ 14/ 35 of 35, worst 9.1e-14 |
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
  arithmetic allows.** Excel is at most 1.3E-13 relative off, with a median of 109 ulp. In units of |ln tail|·2^-53
  (a double kernel ends in exp(-L), L = |ln tail| up to 460, and an ulp of L is L ulp of the tail) its median is 0.9
  and its worst 9.5, flat over the three decades. Rounding the F ratio does not explain it: against the math at the
  double ratio Excel's median is still 117 ulp. Ours reaches 7,870 units, 1.4E-11 relative (Cephes' fraction where the
  L140 port stops at 9 bup terms, df1 of 20 and up). The correctly rounded math is within 15 digits of Excel on only
  22 of the 300 cells. At df1 below 20 (170 cells) ours is T-3754's L140 port in Excel's arithmetic and matches Excel
  on 158 (worst 5.2 units): those must stay on the port. At df1 of 20 and up (130 cells) ours matches none and the
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
  - probes-numeric/t4434-huge-shape-outside-large-shape-beta.case.json (29): Calc!A21, A29, A58, A75, A77, A104, A152, A230, B143, C14, C19, C40, C42, C63, C66, C82, C119, D17, D32:D33, D57, D59, D85, D95, D97, D108, D115, D129, D132

## Acceptance criteria
- [ ] A grid under docs/research/t4435/ (seeded, committed, 60-digit mpmath at the double arguments and the exact F
      ratio) has at least 600 F tails and 150 BETA.DIST points over both regions. Its worst relative error goes from
      today's value (stated in the PR) to at most 1E-14 in each region. If a sub-band cannot reach 1E-14, the reached
      bound and the reason are stated. No point gets worse.
- [ ] Every cell of T-4434's case that T-4434 lists for this task is within half a unit of Excel's 15th significant
      digit.
- [ ] No cell of probes-numeric t4017-f-huge-df2-above-mean-*, t4030-f-dist-rt-df1-boundary, t4080-beta-num-band-edge,
      t1530-f-df-cut, t3354-f-dist-sweep or probes/t4045-f-dist-rt-zero-cut-fresh moves away from Excel. The PR lists
      every cell that moves, as T-4119's Outcome did. No known-failure line is added to eval/baseline/ (decision 0267
      item 6 if a family cell regresses).
- [ ] Not a fit (decision 0188 R4): the dispatch bounds come from the regions' math and TOMS 708's own dispatch, with
      no constant scored against Excel's bits and no branch picked by the cells that already match. The PR says so,
      with the grid as the evidence. Gate A (`rustsheet-eval accuracy`) moves no BETA.DIST, F.DIST or F.DIST.RT point
      out of its envelope.
- [ ] The engine change has a comment citing the grid and T-4434's case. statistics.md "Incomplete beta at a huge
      first shape" states Excel's accuracy per region from T-4434.
- [ ] `make check-fast` green locally, CI green; reviewers approve (excel-fidelity included).

## Out of scope
- Modelling Excel's own inaccurate digits (T-4214 at b0 >= 40, T-3209 for bgrat).
- The `#NUM!` band at df1 61 to 79 and df2 1E10 (T-4044, done) and tails below DBL_MIN (T-4045, T-4254).

## Visible-parity stage
Every cell of T-4434's sweep that T-4434 lists for this task is within half a unit of Excel's 15th significant digit
(decisions 0083, 0114 `accuracy`).
