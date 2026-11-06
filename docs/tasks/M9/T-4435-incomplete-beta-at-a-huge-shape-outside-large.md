---
id: T-4435
title: "Incomplete beta at a huge shape outside large_shape_beta's bounds still takes Cephes' fraction: F.DIST.RT and BETA.DIST 1E-13 to 9E-8 off the math (FDIST(1.5,80,1E10), F.DIST.RT(5,24,1E6); T-4119 follow-up)"
milestone: M9
lane: functions-stat
status: todo
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
