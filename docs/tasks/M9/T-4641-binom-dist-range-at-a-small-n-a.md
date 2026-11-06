---
id: T-4641
title: "BINOM.DIST.RANGE at a small n: a range not past the mean is Excel's own cdf difference, not the exact walk (11 visibly-off cells of the t2145 range sweep; Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
BINOM.DIST.RANGE(n, p, s1, s2) with 0 < s1 < s2 and s1 - 1 <= n·p returns Excel's own
`BINOM.DIST(s2,n,p,TRUE) - BINOM.DIST(s1-1,n,p,TRUE)`, the difference of the engine's BINOM.DIST cdfs, also inside
the walk's budget. Today the engine returns the exact sum rounded once there, and Excel does not.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main b5f8613c1). The visibly-off cells below were owned only through
  T-3743, a sonnet docs follow-up that cites their case as evidence and puts the cell work out of scope. No other
  open task or pushed branch names them. Every one is decision 0188 R1(c) Phase 1b evidence: Excel and the engine
  are both within 1E-9 of the 50-digit math, in an ungated dense sweep. So R5 gives the task `defer: phase-1b` and
  no `campaign:` key.
- The rule comes from the excel-graph sidecars already in eval/ (no new probe). It is the small-n bullet that this
  triage added to docs/excel-semantics/statistics.md "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's
  own cdf (T-3444)". The T-2145 binomial sweep holds Excel's BINOM.DIST cdf at the same n and p as the range sweep,
  n 1 to 12. Against it, every interior range not past the mean is the difference of Excel's two cdfs to the bit,
  156 of 156. Past the mean only 23 of 54 are.
- Cells this task owns. Each value is shown as Excel, then ours (the exact walk), then the 50-digit math.
  - probes-numeric/t2145-binom-range-sweep.case.json (11): Calc!A291, A431, A494, A508, A564, A648, A704, A711, A718, A781, A788
  - A291 `BINOM.DIST.RANGE(5,0.99,1,3)`: 0.0009801495000000006 / ...017 / ...017. That is Excel's cdf cells A384 -
    A378 of the binomial sweep.
  - A494 `BINOM.DIST.RANGE(8,0.9,2,5)`: 0.03809106000000007 / 0.038091059999999975 / same as ours. That is A838 -
    A830 of the binomial sweep.
  - The other nine are also Excel's cdf difference, but one of their two cdfs is off Excel in the engine today (1 to 4
    ulp, BINOM.DIST's own residue). After the change they equal the engine's own cdf difference bit for bit, and they
    pass once those cdfs do. For example A431 `BINOM.DIST.RANGE(7,0.99,2,4)` uses the binomial sweep's A678, which
    is within 4 ulp.
- What changing all 156 interior ranges not past the mean to the engine's own cdf difference does (computed from
  today's report): 104 bit-exact become 136. One passing cell goes to 1 ulp off,
  probes-numeric/t2145-binom-range-sweep Calc!A683 `BINOM.DIST.RANGE(11,0.2,3,6)`. The reason is that the engine's
  `BINOM.DIST(2,11,0.2,TRUE)` (the binomial sweep's A1402) is 1 ulp off Excel (ours 0.6174015488, Excel
  0.6174015487999999). The A1402 cdf residue is not this task's: report it and leave it.
- Not owned here:
  - The single-point and from-0 ranges already take the engine's BINOM.DIST pmf and cdf. Their misses are BINOM.DIST's
    own residue (T-4365, deferred).
  - The ranges past the mean. Five of them are visibly off (A467, A537, A607, A670 and A747, all ending at n, s1 at
    mean + 3.3 to 4.6 sd). They are T-3740's band at a small n: Excel's form there is not identified.
- Code: crates/rustsheet-engine/src/functions/statistical/binom.rs `binom_dist_range_of`. It takes the pmf for one
  point, the cdf from 0, then `binomial_range` (discrete.rs, the double-double walk), and only past the walk's budget
  the cdf difference with `binom_cdf_or_statrs`. The change orders the cdf difference (`excel_binom_cdf`) before the
  walk for interior ranges not past the mean. Reuse the existing difference code; do not add a second one
  (directive 3).
- The guard s1 - 1 <= n·p is a cut read from evidence, so decision 0188 R4 applies. The cached points are enough to
  choose it, so the task needs no separate sweep task. The fresh sweep below is the after-freeze check.
- Gated cells that take this path: ironcalc-calc STATISTICAL/BINOM.xlsx (its BINOM.DIST.RANGE sheet) and the gen-m9
  BINOM.DIST.RANGE cells. Check them per cell, main against HEAD.

## Acceptance criteria
- [ ] probes-numeric/t2145-binom-range-sweep Calc!A291 and A494 pass bit-exact, as `make eval-show` of the case
      shows.
- [ ] A unit test pins it: for each of the other nine owned cells, the engine's BINOM.DIST.RANGE equals its own
      `BINOM.DIST(s2,..,TRUE) - BINOM.DIST(s1-1,..,TRUE)` bit for bit.
- [ ] The Outcome reports the range sweep's interior ranges not past the mean, before and after (bit-exact, within
      4 ulp, within 15 digits), and names every cell that gets worse with its cause. Expected: 104 to 136 bit-exact,
      A683 the only cell that gets worse.
- [ ] No gated cell regresses (ironcalc-calc BINOM.xlsx, gen-m9, probes; main against HEAD per cell). Known failures
      in eval/baseline/ only shrink (`make eval-baseline` refuses regressions).
- [ ] R4 guards (decision 0188; reviewer-excel-fidelity check 7), each shown in the Outcome:
  - [ ] Hold-out: split the 156 interior ranges not past the mean and the 54 past it by a cell hash before scoring.
        Show train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the cdf
        difference and for the exact walk. The cdf difference beats the walk on the hold-out.
  - [ ] Fresh sweep: after the guard is frozen, a new oracle-filled probe that is never used for choosing. Use
        excel-graph and a committed generator, under eval/suites/probes/cases/, since probes-numeric takes no new case
        (decision 0267 item 5). It covers interior ranges at n 13 to 1000, with s1 - 1 on both sides of n·p within
        2 sd, and BINOM.DIST cdf cells beside each range. Alternatively use an `r4-T-4641` batch of the Gate A
        BINOM.DIST.RANGE table.
  - [ ] Gate A envelope: the fresh sweep's count inside the envelope (`rustsheet-eval accuracy`), with every point
        outside it listed.
  - [ ] `FITTED` tag and range: the guard carries `FITTED(T-4641, n_cells, range)` with the evidenced n range, and
        applies only inside it. Outside it the code keeps today's path.
  - [ ] Reproducibility floor: no choice rests on differences below the oracle's floor (web against desktop up to
        4 ulp, decision 0112).
  - [ ] No keep-matched branch: the walk is not kept for the cells where it happened to match (decisions 0056,
        0082).
- [ ] statistics.md: the small-n bullet of the T-3444 entry records what the fresh sweep shows and the guard's
      evidenced range.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Ranges past the mean (T-3740), and BINOM.DIST's own pmf and cdf last digits (T-4365), including the binomial
  sweep's A1402.
- The BINOM.DIST.RANGE lift weight and the stale statistics.md passages (T-3743).
