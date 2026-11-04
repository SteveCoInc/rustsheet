---
id: T-4218
title: "Oracle sweep: GAMMA.DIST cdf at shapes 3E7 to 1E10 between 0.5 and 8 standard deviations from the mean, to fit Excel's tail where t2982 A145, A146 are 14 to 18 ulp off the math"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
probes-numeric holds an oracle-filled sweep of the incomplete gamma cdf at huge shapes away from the mean, with
50-digit references, so that T-4219 can tell Excel's tail arithmetic around the 1E8 shape apart from the math. Today
two cells exist there.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 0516b489). `make exit-owners MILESTONE=M9` lists two failing M9
  exit cells with no owner: probes/t2982-gamma-inv-huge-shape S!A145 `GAMMA.DIST(99950000,1E8,1,TRUE)` and S!A146
  `GAMMA.DIST(99950000,99999999,1,TRUE)`. Both points are 5 standard deviations below the mean. Excel is 3.4E-15 and
  -2.5E-15 relative off the math there; ours is -1.7E-16 and +8E-17. T-4219 has the numbers.
- Excel cells at a shape of 1E8 to 1E12 are few, and nearly all of them are at the mean or are densities: t2435 (18),
  t2982 (15), incomplete-gamma-cap (3), fit-audit-t3271-density-cut (34, densities), six gen-m9 cells. The tail cdf a
  few sigma out is unprobed, which leaves too few points for a model.
- docs/excel-semantics/statistics.md: from a shape of 1E8 ours is Temme's uniform asymptotic expansion
  (`incomplete_gamma.rs` `temme_pair`, T-1891). Below 1E8 it is the series or the continued fraction. Shapes 3E7 to
  99999999 cover the cut from the side below.

## Acceptance criteria
- [ ] A probes-numeric case `t4218-gamma-huge-shape-tails` (milestone tag M9). Shapes a in {3E7, 9.99E7, 99999999,
      1E8, 100000001, 3E8, 1E9, 1E10}. z in {-8, -6, -5, -4, -3, -2, -1, -0.5, 0.5, 1, 2, 3, 4, 5, 6, 8}. Integer
      x = ROUND(a + z·SQRT(a), 0) written as a literal. Cells `GAMMA.DIST(x,a,1,TRUE)` (128) and
      `CHISQ.DIST.RT(2x,2a)` for a <= 1E9 (112; 2a = 2E10 exceeds the df limit).
- [ ] A hold-out sibling `t4218-gamma-huge-shape-tails-hold`: 200 `GAMMA.DIST(x,a,1,TRUE)` cells with log-uniform a in
      [3E7, 1E10] and uniform z in [-8, 8], from a fixed seed. The generating script is committed under
      docs/research/t4218/. Nobody tunes on these cells.
- [ ] Both cases are oracle-filled (`rustsheet-oracle fill -b excel-graph --suite probes-numeric`) and have `.math.json`
      sidecars (`rustsheet-accuracy math --suite probes-numeric`). Where the sidecar gives no value ("no single value",
      or mpmath's 1F1 series not converging, as it does not at these shapes), docs/research/t4218/reference.py gives
      50-digit values by quadrature or by `gammainc` with a raised `maxterms`. Two precisions must agree to 30 digits,
      and the values are committed as a table.
- [ ] docs/excel-semantics/statistics.md, the huge-shape incomplete gamma entry, gains one paragraph with the counts:
      cells correctly rounded, within 4 ulp and within 15 digits of the math, per shape, for Excel and for ours. Name
      the shape and z ranges where Excel is more than 4 ulp off.
- [ ] `make eval-baseline` records the new cases' failures as new coverage only (probes-numeric is not gated); no gated
      suite changes; `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-4219). The quantiles at these shapes (T-3272, T-3804).
