---
id: T-4434
title: "Oracle sweep: F.DIST.RT, F.DIST and BETA.DIST where an incomplete-beta shape is huge but outside large_shape_beta (df2 1E4 to 2E6, or df1 80 to 400), the engine 1E-13 to 9E-8 off the math and no Excel cell"
milestone: M9
lane: research
status: todo
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
  `rustsheet-eval show --all`, against mpmath 1.4 `betainc` at 40 to 60 digits, with the exact ratio
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
- [ ] A seeded generator under docs/research/t4434/ writes one probes-numeric case (the `excel-probe` skill,
      excel-graph fill, oracle sidecar committed). It has at least 900 cells:
  - 300 `F.DIST.RT(x,df1,df2)` in region 1: df1 log-uniform in [3, 79], df2 log-uniform in [1E4, 2E6], and x chosen so
    the tail is log-uniform in [1E-200, 0.5].
  - 300 `F.DIST.RT(x,df1,df2)` in region 2: df1 log-uniform in [80, 400], df2 log-uniform in [1E5, 1E10], and x from
    0.7 to 3 times the mean.
  - 150 `F.DIST(x,df1,df2,TRUE)` at the same points as 150 of the region-2 rows, in the next column.
  - 150 `BETA.DIST(x,a,b,TRUE)` with a log-uniform in [20, 200], b log-uniform in [5E4, 5E9], and x from 0.7 to 1.5
    times a/b.
- [ ] Every argument is a literal that round-trips to the intended double (17 significant digits). The case
      description names the generator, its seed and this task.
- [ ] A script under docs/research/t4434/ scores Excel and the engine against mpmath at 50 or more digits. It uses the
      double arguments, and for F calls the exact ratio. It reports, per region and per decade of the huge shape,
      the counts within 1E-15, 1E-14 and 1E-12 relative for each side, and the worst error of each side. The table is
      in this task's Outcome and in T-4435's Context.
- [ ] The case's failing cells are added to eval/baseline/probes-numeric.txt through `make eval-baseline` (new
      known failures of a new case, decision 0083). Each visibly-off cell is listed in T-4435, or, where Excel is the
      side off the math by more than 1E-13, in T-4214 (b0 >= 40) or T-4076 (both sides within 1E-9 of the math).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-4435).
- Tails below DBL_MIN and the zero cut (T-4045, done; T-4254).
- df1 79 or less at df2 1E7 and up, which T-4017's sweep covers.
