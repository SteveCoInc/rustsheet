---
id: T-3960
title: "Oracle sweep: BINOM.DIST and NEGBINOM.DIST cdf at p below 0.01 with a huge n or failure count, with a hold-out, for T-3961"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Before T-3961 fits anything, it has dense excel-graph evidence of the BINOM.DIST and NEGBINOM.DIST cdf in the regime
where Excel is hundreds to tens of thousands of ulp off the math: p below 0.01 with a huge trial or failure count.
The evidence is a train case and a separate hold-out case (decision 0188 R4.1), so no fit is attempted blind.

## Context
- Gap triage 2026-11-03b. T-3961 owns 11 visibly-off cells there (probes/t3352-binom-negbinom-large-n A34, A52, A70,
  A88; probes-numeric/t3352-binom-negbinom-sweep A132, A147, A156, A171, A184, A190, A208). Excel is up to 8.1E-12
  relative off the math (A132 `BINOM.DIST(2461,16194261,0.000154374041567456,TRUE)`) and ours is close to it. These 11
  are every cached point of the regime, too few for a model with a hold-out.
- The existing sweep (probes-numeric/t3352-binom-negbinom-sweep, seed 3352, with `.oracle.json` and `.math.json`
  sidecars) shows the case layout. The new cases follow it. Write 15-significant-digit p literals: Excel truncates
  formula literals to 15 digits, so a longer literal is not the double the generator meant.

## Acceptance criteria
- [ ] A new case `eval/suites/probes-numeric/cases/t3960-binom-negbinom-small-p-cdf.case.json` (`rustsheet-case/v1`,
      sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the function, k, n or r, and p), built by
      a seeded generator `docs/research/t3960/sweep.py`. Grid of about 400 cdf cells:
  - BINOM.DIST(k, n, p, TRUE): p log-uniform on [1E-5, 1E-2], n log-uniform on [1E5, 1E9], k = round(n p + z sqrt(n p q))
    at z drawn uniformly on [-3, 2] (200 cells);
  - NEGBINOM.DIST(k, r, p, TRUE): p log-uniform on [1E-5, 1E-2], r log-uniform on [10, 1E4], k at z on [-3, 2] around
    the mean r (1 - p) / p (200 cells);
  - 20 of each family at the 11 owned cells' own (n or r, p), with k stepped by 1 around the owned k, as controls.
- [ ] A second case `t3960-binom-negbinom-small-p-cdf-hold.case.json` (200 cells, the same grid rule, another seed and
      no shared point) is in the same PR. Its description says it is T-3961's hold-out and is never used for fitting.
- [ ] Both cases are filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`
      (excel-probe skill). Both `.oracle.json` sidecars are committed with no pending cells, and both math sidecars
      are generated with `rustsheet-accuracy math --suite probes-numeric` and committed.
- [ ] statistics.md "BINOM.DIST and NEGBINOM.DIST at a large n" cites both cases with counts against today's engine
      (bit-exact, within 4 ulp, within 15 digits) and the count of Excel cells correctly rounded, per family.
- [ ] Owners for the new cases' visibly-off cells (decision 0245): T-3961's cell list names both cases.
- [ ] probes-numeric is ungated, so the cases add no eval/baseline/ line; `make check` green; reviewers approve.

## Out of scope
- Any engine change, and any fitting (T-3961).
