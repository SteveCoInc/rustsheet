---
id: T-4434
title: "Oracle sweep: F.DIST.RT, F.DIST and BETA.DIST where an incomplete-beta shape is huge but outside large_shape_beta (df2 1E4 to 2E6, or df1 80 to 400), the engine 1E-13 to 9E-8 off the math and no Excel cell"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A probes-numeric case, filled by the oracle, gives T-4435 the Excel values it needs. The case covers the two
incomplete-beta regions where the engine still uses Cephes' continued fraction at a huge shape. There the engine is
1E-13 to 9E-8 relative off the 50-digit math, and the eval has almost no Excel cell. With these values T-4435 can
tell, region by region, whether Excel is near correctly rounded (its `accuracy` stage) or is the inaccurate side
(T-4214's and T-3209's subject).

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 39f04d556, all suites fetched). Every failing cell of the run has
  an open owner, so this gap is an evidence gap. The engine was measured on a scratch case through
  `rustsheet-eval show --all`, against mpmath 1.3 `betainc` at 40 to 60 digits, with the exact ratio
  d2/(d2 + d1·x) for the F calls:
  - **Region 1, upper F tails at df2 1E5 to 2E6 and df1 3 to 79.** Here a0 = df2/2 is below
    `large_shape_beta.rs` `MIN_SHAPE` (1E6). Measured errors:
    - `F.DIST.RT(5,24,1E6)` (tail 9.7E-15) is 1.2E-11 off.
    - `F.DIST.RT(20,24,1E6)` is 3.5E-12 off.
    - `F.DIST.RT(5,30,1E6)` is 1.2E-12 off.
    - `F.DIST.RT(5,24,1E5)` is 2.0E-13 off.
    - At df2 1E8, inside `large_shape_beta`, the same calls are within 1E-14. At df2 1E3 to 1E4 they are within 1E-13.
    - Near the mean, at df1 40 to 79 and df2 1E6, `F.DIST.RT(1.5,40,1E6)` is 1.2E-11 off.
  - **Region 2, df1 80 to 400 at df2 1E6 to 1E10, near the mean.** Here b0 = df1/2 >= 40, which is
    `large_shape_beta.rs` `MAX_SECOND_SHAPE`. Measured errors:
    - `FDIST(1.5,80,1E10)` is 4.7E-8 off, `F.DIST.RT(1.4,80,1E10)` 8E-8, `F.DIST.RT(1.5,200,1E10)` 8.8E-8.
    - `F.DIST.RT(1.5,80,1E8)` is 6.6E-10 off and `F.DIST.RT(1.5,80,1E6)` 4.3E-12.
    - The same orientation in BETA.DIST, small first shape and huge second: `BETA.DIST(9.6E-9,40,5E9,TRUE)` is 1.9E-9
      off, `BETA.DIST(9.6E-7,40,5E7,TRUE)` 5.6E-11 and `BETA.DIST(4.8E-5,20,5E5,TRUE)` 1.6E-12.
- Excel cells in these regions today:
  - Region 1: only probes/t4045-f-dist-rt-zero-cut-fresh columns C to F (tails near 1E-286, where Excel itself is
    3E-15 to 2E-13 off the math), plus one gen-m9 cell and two t3354 cells.
  - Region 2: probes-numeric/t4030-f-dist-rt-df1-boundary A39, A40, A61 and A62, which T-4214 owns (Excel 9E-10 to
    7E-9 off the math), and probes-numeric/t4045-f-dist-rt-zero-cut E3 (Excel 1.0E-8 off, ours 4.7E-8).
  - No BETA.DIST cell has a first shape of 5 to 1000 and a second of 1E5 or more near the mean, except the six
    t3576-negbinom-cdf-huge-s cells and the twelve t3576-r2-ratio-overflow-sweep cells at b 1E13 and up.
  - So it is unknown whether Excel's TOMS 708 bratio is accurate in region 1 and in region 2 below df2 1E9. T-4214
    shows it is not accurate at a0 2E9 to 1E10 and b0 = 40.
- T-4017's sweep (probes-numeric/t4017-f-huge-df2-above-mean-*) stops at df1 79 and starts at df2 1E7, so it covers
  neither region. Excel is `#NUM!` at df1 79 and df2 1E10 (T-4044; statistics.md "Incomplete beta at a huge first
  shape"). The sweep should expect `#NUM!` cells near that band and keep them: they are evidence too.
- Probe freeze (decision 0267 item 5): this case is the evidence T-4435 needs before it can be staged. Name T-4435 in
  the PR.

## Acceptance criteria
- [x] A seeded generator under docs/research/t4434/ writes one probes-numeric case (the `excel-probe` skill,
      excel-graph fill, oracle sidecar committed). It has at least 900 cells:
  - 300 `F.DIST.RT(x,df1,df2)` in region 1: df1 log-uniform in [3, 79], df2 log-uniform in [1E4, 2E6], and x chosen so
    the tail is log-uniform in [1E-200, 0.5].
  - 300 `F.DIST.RT(x,df1,df2)` in region 2: df1 log-uniform in [80, 400], df2 log-uniform in [1E5, 1E10], and x from
    0.7 to 3 times the mean.
  - 150 `F.DIST(x,df1,df2,TRUE)` at the same points as 150 of the region-2 rows, in the next column.
  - 150 `BETA.DIST(x,a,b,TRUE)` with a log-uniform in [20, 200], b log-uniform in [5E4, 5E9], and x from 0.7 to 1.5
    times a/b.
- [x] Every argument is a literal that round-trips to the intended double (17 significant digits). The case
      description names the generator, its seed and this task.
- [x] A script under docs/research/t4434/ scores Excel and the engine against mpmath at 50 or more digits. It uses the
      double arguments, and for F calls the exact ratio. It reports, per region and per decade of the huge shape,
      the counts within 1E-15, 1E-14 and 1E-12 relative for each side, and the worst error of each side. The table is
      in this task's Outcome and in T-4435's Context.
- [x] The case's failing cells are recorded through `make eval-baseline` (new known failures of a new case,
      decision 0083); probes-numeric has no baseline file (decision 0203), so it adds no line. Each visibly-off cell is
      listed in T-4435 where the correctly rounded math is within 15 digits of Excel, else, where Excel is the side
      off the math, in T-4214 (b0 >= 40) or T-3209 (b0 < 40). Amended at review (the Outcome says why): the criterion
      first sent cells away from T-4435 only where Excel is more than 1E-13 off, and named T-4076 for cells both
      sides have within 1E-9.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-4435).
- Tails below DBL_MIN and the zero cut (T-4045, done; T-4254).
- df1 79 or less at df2 1E7 and up, which T-4017's sweep covers.

## Outcome
- probes-numeric/t4434-huge-shape-outside-large-shape-beta: 900 cells written by docs/research/t4434/gen_probe.py
  (seed 4434), filled on excel-graph 2026-11-07, with its oracle and math sidecars (`rustsheet-accuracy`'s 897 math
  cells agree with `mathref.py` to 4E-40; it skips B27, B118 and B153 as not converged). Every literal has at most 12
  significant digits, so it is read as exactly the double it names (Excel reads literals at 15 digits) and round-trips
  through `repr` and `%.17g`. Decision 0334 names the case as an exception to decision 0267 item 5's probe freeze, on
  the owner's request of 2026-11-07.
- docs/research/t4434/score.py scores both sides against 60-digit math (`mathref.py`: DLMF 8.17.8's positive-term 2F1
  sum, two precisions agreeing to 65 digits; the F math at the exact ratio d1 x/(d2 + d1 x)). Counts within 1E-15 /
  1E-14 / 1E-12 relative and the worst cell, per decade of the huge shape (a0 = df2/2 for F, b for BETA.DIST):

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

Error in units of max(1, |ln value|)·2^-53 (median / 90th percentile / worst for Excel, median / worst for ours):

| region | huge shape | cells | Excel median | Excel 90th pct | Excel worst | engine median | engine worst |
|---|---|---:|---:|---:|---:|---:|---:|
| region 1 F.DIST.RT | 1E3 to 1E4 | 35 | 0.585 | 2.11 | 4.14 | 0.743 | 8.26 |
| region 1 F.DIST.RT | 1E4 to 1E5 | 150 | 0.935 | 2.36 | 9.49 | 1.24 | 1.86e+03 |
| region 1 F.DIST.RT | 1E5 to 1E6 | 115 | 0.914 | 2.09 | 4.91 | 1.41 | 7.87e+03 |
| region 2 F.DIST.RT | 1E4 to 1E5 | 15 | 3.87 | 202 | 498 | 151 | 962 |
| region 2 F.DIST.RT | 1E5 to 1E6 | 68 | 22.9 | 2.02e+03 | 1.57e+04 | 1.03e+03 | 1.74e+04 |
| region 2 F.DIST.RT | 1E6 to 1E7 | 58 | 183 | 3.68e+04 | 5.23e+05 | 6.01e+03 | 1.42e+05 |
| region 2 F.DIST.RT | 1E7 to 1E8 | 55 | 2.63e+03 | 9.49e+04 | 4.33e+05 | 6.07e+04 | 1.07e+06 |
| region 2 F.DIST.RT | 1E8 to 1E9 | 69 | 3.75e+04 | 8.73e+06 | 6.49e+07 | 1.02e+06 | 1.94e+07 |
| region 2 F.DIST.RT | 1E9 to 1E10 | 35 | 1.48e+05 | 9.68e+06 | 3.82e+07 | 6.63e+06 | 7.56e+07 |
| region 2 F.DIST | 1E4 to 1E5 | 3 | 4.74 | 11 | 11 | 15.1 | 58.1 |
| region 2 F.DIST | 1E5 to 1E6 | 34 | 1.31 | 298 | 2.05e+03 | 0.931 | 2.76e+03 |
| region 2 F.DIST | 1E6 to 1E7 | 30 | 0.627 | 6.81e+04 | 3.46e+05 | 0.627 | 8.39e+04 |
| region 2 F.DIST | 1E7 to 1E8 | 26 | 2.75 | 3.09e+04 | 2.04e+05 | 2.46 | 1.36e+05 |
| region 2 F.DIST | 1E8 to 1E9 | 37 | 10.5 | 4.35e+06 | 7.22e+06 | 11.6 | 6.91e+06 |
| region 2 F.DIST | 1E9 to 1E10 | 20 | 3.66 | 4.59e+06 | 1.2e+07 | 7.54 | 2.37e+07 |
| BETA.DIST | 1E4 to 1E5 | 5 | 2.39 | 76.2 | 76.2 | 12.6 | 55.6 |
| BETA.DIST | 1E5 to 1E6 | 25 | 7.16 | 1.83e+03 | 7.98e+03 | 25.3 | 1.46e+04 |
| BETA.DIST | 1E6 to 1E7 | 39 | 7.16 | 8.36e+03 | 1.35e+05 | 5.92 | 2.7e+04 |
| BETA.DIST | 1E7 to 1E8 | 32 | 6.77 | 3.54e+04 | 6.53e+06 | 5.49 | 4.45e+05 |
| BETA.DIST | 1E8 to 1E9 | 35 | 10.6 | 3.76e+05 | 3.11e+07 | 3.89 | 4.05e+06 |
| BETA.DIST | 1E9 to 1E10 | 14 | 3.12e+05 | 1.51e+07 | 2.43e+07 | 2e+06 | 5.32e+07 |

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

Owners (`score.py` `owner`, the cells where ours is not within 15 digits of Excel). Review choice (b): the
criterion's rule (away from T-4435 only where Excel is more than 1E-13 off) is amended. T-4435 is the `accuracy`
stage and can bring a cell within 15 digits of Excel only where the correctly rounded math is; a cell where Excel is
1E-14 to 1E-13 off and the math not within 15 digits of Excel is Excel's own digits, which only a model of Excel's
arithmetic reaches. So T-4435's acceptance lists exactly the cells an accurate kernel can match. How far Excel is off
the math on each list (`score.py`'s last block):
- T-4435: all 29 within 1E-14 (worst 3.5E-15).
- T-4214: 261 of the 342 more than 1E-13 off, 42 from 1E-14 to 1E-13, 39 within 1E-14 (Excel's bfrac digits).
- T-3209: 2 of the 146 more than 1E-13 off, 100 from 1E-14 to 1E-13, 44 within 1E-14; worst 1.2E-13.
- T-4435 (29): the correctly rounded math is within 15 digits of Excel, so an accurate kernel matches.
- T-4214 (342): Excel is the side off the math and bratio's b0 is 40 or more (bfrac). Listed in its "T-4434 cells".
- T-3209 (146, deferred): Excel is the side off the math at b0 below 40 (bgrat digits, mostly 1E-14 to
  1.2E-13). Listed in its Evidence. Both sides are within 1E-9 of the math on every one of them, so they are Phase 1b
  evidence (decision 0188 R1(c)) and not counted.
- The owner gate (`scripts/new_case_owners.py`) counts 82 cells once the math sidecar is in (the rest are R1(c)
  evidence), all in T-4214's list. The task's text named T-4076 for cells both sides have within 1E-9; the owner
  asked for T-4214 or T-3209 instead, and R1(c) already leaves those out.
- Baseline: probes-numeric has no eval baseline (ungated; decision 0203), so `make eval-baseline` adds no line for the
  case. Its counts show in `make eval`'s report.
- statistics.md "Incomplete beta at a huge first shape" has the region verdicts (the T-4434 item).
