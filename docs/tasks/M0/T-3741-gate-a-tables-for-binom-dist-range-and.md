---
id: T-3741
title: "Gate A tables for BINOM.DIST.RANGE and BINOM.INV (fresh sweep at n to 2^31 - 2)"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3444]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Committed, oracle-filled Gate A tables for BINOM.DIST.RANGE and BINOM.INV (and CRITBINOM), each with a cited mpmath
definition, and `make check` running them: the fresh-sweep criterion split out of T-3444.

## Context
- Split out of T-3444 (2026-11-01): T-3444's probe (probes/t3444-binom-range-inv-large-n, 756 cells, excel-graph)
  established Excel's forms, but its acceptance criterion "Gate A floor on a fresh sweep (decision 0188 R2): 0 points
  outside both 15 digits of Excel and 1E-9 of the math" needs Definitions these functions do not have yet, more than
  fits in T-3444 (as T-3553 split the discrete distributions out of T-3358).
- Add the Definitions to `tools/src/rustsheet_tools/accuracy/defs/discrete.py` beside BINOM.DIST: reuse `binom_cdf`,
  `binom_sf`, `binom_pmf` and `integer_quantile` (BINOM.INV is the smallest k with cdf(k) >= alpha, statistics.md
  "BINOM.DIST.RANGE and BINOM.INV at a large n sit on BINOM.DIST's own cdf (T-3444)"). The range needs
  number_s <= number_s2 <= trials in its strata. Generate with `cd tools && uv run rustsheet-accuracy generate
  <FUNCTION>`, fill with `uv run rustsheet-oracle fill -b excel-graph --suite accuracy`, check with
  `rustsheet-eval accuracy` (SPEC-0029).
- A failing point is carried by a `known-failures.tsv` line naming its owner task, only for points added in the same
  diff (T-3379); never by weakening ε or the 15-digit rule. BINOM.DIST's own last digits are T-3447; the upper-tail
  range rows T-3740.

## Acceptance criteria
- [x] Gate A tables for BINOM.DIST.RANGE and BINOM.INV, oracle-filled (excel-graph), with cited definitions.
- [x] 0 points outside both 15 digits of Excel and 1E-9 of the math, or each such point carried by a known-failures
      line naming an open owner task added in the same diff.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing BINOM.DIST.RANGE or BINOM.INV (T-3740, T-3447).

## Outcome
- New Gate A family `tools/src/rustsheet_tools/accuracy/defs/binom_range.py`: BINOM.DIST.RANGE, BINOM.INV and
  CRITBINOM, with cited sources (statistics.md T-3444, T-2435, T-2645, T-1048 entries; Microsoft documentation;
  A&S 26.1.20) and per-stratum draws (small ranges, deep tails, trial counts from 1E6 to 2^31 - 2 near the mean,
  the bounds and the documented #NUM! orders). mpmath's incomplete beta does not converge near the mean at n near
  2^31 (a 2F1 with a = n - k), so the definitions sum the pmf outward from k with a geometric tail bound (under
  500,000 terms, each charged to the work budget), and BINOM.INV walks one pmf step at a time from a normal start,
  then checks the crossing with two fresh sums. `discrete.py` (BINOM.DIST's definition) is unchanged.
- Tables accuracy/BINOM.DIST.RANGE, BINOM.INV and CRITBINOM s1, 200 points each, 0 dropped, oracle-filled from
  excel-graph (2026-11-04); 47, 53 and 50 points at 1E6 trials or more.
- Gate A: 600 new points, 591 in the envelope, 0 failing. 9 BINOM.DIST.RANGE points above the mean are carried by
  known-failures lines owned by T-3740 (ours cancels in the cdf difference; Excel is within 7E-12 of the math). BINOM.INV
  and CRITBINOM: 200 of 200 each in the envelope; one point each is in by Excel's 15 digits only (Excel's count 3 and 4
  below the math's at alpha within 3E-14 of 1; ours is Excel's).
- New semantics (statistics.md, T-3444 entry): Excel's upper-tail ranges stay within 7E-12 of the math down to
  1.9E-134; BINOM.INV's near-1 alpha off-by-a-few is the double cdf; BINOM.INV(0,p,alpha) = CRITBINOM(0,..) = 0 and
  BINOM.DIST.RANGE(0,p,0) = 1. T-3740's context lists its 9 points.
- No numeric fit: the definitions are the math, chosen before scoring.
- Eval delta: none (Gate A tables are outside eval/suites).
