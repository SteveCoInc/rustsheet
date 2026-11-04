---
id: T-4080
title: "Probe the BETA.DIST/F #NUM! band edge at a huge first shape: a 1E9..1E10 by b 30..39.5 by tail, to pin the edge as a function of a and x"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
The edge of Excel's `#NUM!` band for the incomplete beta at a huge first shape is pinned as a rule in (a, b, x), or
shown to have none. T-4044 can then implement the band from evidence, not from a cut fitted to its cells.

## Context
- statistics.md "Incomplete beta at a huge first shape", T-4030 paragraph (probes-numeric/t4030-f-dist-rt-df1-boundary,
  235 cells, excel-graph):
  - `BETA.DIST(x,a,b,TRUE)` and the F functions at (a, b) = (df2/2, df1/2) are `#NUM!` in a band of b below 40.
  - The first `#NUM!` b by a: none to 39.5 at a = 1E9 and 2E9; 39.5 at 2.5E9 and 3E9; 38.5 at 4E9 and 5E9; 37.5
    at 6E9. From b = 40 the result is a number again.
  - x differed by row: tails of 0.04 to 0.06, except at 5E9, where they were about 5E-7 to 7E-6, and two rows at
    0.99999 and 0.999999.
- T-4044 (blocked on this task) found the probed points do not pin a rule:
  - No threshold of the form b + g(a) separates them for g = k ln a, k·a or k·sqrt(a), nor b ln a - k ln Gamma(b + 1),
    for any k in [0, 10).
  - A monotone step g is consistent with the points, but nothing pins it, and the x dependence is open.
  - TOMS 708's L140 path in doubles (bup with bratio's n = int(b0) terms, brcmp1, and bgrat's error exits on b·z = 0,
    u = 0 and sum <= 0) has no exit that explains the half-integer b cells: bup's exp(708) scaling runs only at an
    integer b.
- Probe with the excel-probe skill. BETA.DIST only is enough: the F functions take the same type at the same shapes.
  - a: 2E9, 2.25E9, 2.5E9, 2.75E9, 3E9, 3.5E9, 4E9, 4.5E9, 5E9, 5.5E9, 6E9, 7E9, 8E9 and 1E10.
  - b: 30 to 39.5 in steps of 1/2, and 40.
  - At least three tails per (a, b): x chosen for I of about 1E-6, 0.05 and 0.5, and x = 1 - 1E-12 (a tail near 0).
  - The cdf side only, plus a few `BETA.DIST(..,FALSE)` density cells to see whether the pdf shares the band.
- Then state the rule in statistics.md (replacing "the exact edge function of a is not pinned"). Name the TOMS
  quantity it tracks if one fits, for example a bup term count k = min(n - 1, (a0 - 1)·y0/x0 - b0) or a bgrat z.

## Acceptance criteria
- [x] A probe case under eval/suites/probes-numeric, filled by the oracle, covering the grid above.
- [x] statistics.md gives the edge as a rule with its evidence, or records that the cells admit none and why.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the band (T-4044). Excel's digits at huge df2 (T-3209, T-4018).

## Outcome
- probes-numeric/t4080-beta-num-band-edge (1,232 cells, excel-graph 2026-11-04): BETA.DIST cdf at 14 a (2E9 to 1E10)
  by 21 b (30 to 40 by 1/2) by four tails, plus 56 density cells.
- Rule (statistics.md "Incomplete beta at a huge first shape", "The edge (T-4080)"): the band is in bup's term count
  n = int(b) (less 1 at an integer b), `#NUM!` exactly when n >= n_min(a) (39 at 2.25E9 to 3E9, 38 at 3.5E9 to 5.5E9,
  37 at 6E9 to 8E9, 36 at 1E10, none at 2E9), only on tails that take TOMS 708's L140 path (lambda > 0); I near 0.5,
  x = 1 - 1E-12 and the density are numbers. On the probed a the edge separates as n + 2 ln a > T, T in (81.86, 81.95)
  (weight any value in (1.79, 2.21)); a separating line, not an identified TOMS quantity, so T-4044 should treat it as
  a fit (decision 0056 guards) or probe a finer a grid.
- Eval: gated suites unchanged (`make eval-baseline`: 0 fixed, 0 regressions; probes-numeric is not gated). In the
  report the new case adds 108 known failures: the 82 band cells (owned by T-4044, amended) and 26 b = 40 cells 1.3E-9
  to 4.6E-8 relative off (deferred T-3209, amended); no other change.
