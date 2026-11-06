---
id: T-4361
title: "Oracle sweep: BINOM.DIST.RANGE single-point and short ranges at a tiny p (1E-300 to 1E-6), with a hold-out, for the A360 fit"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-graph values for BINOM.DIST.RANGE where p is tiny, enough points for T-4362 to fit Excel's form there
(decision 0188 R4: a training sweep and a separate hold-out).

## Context
- The only Excel point today is gen-m9 9cca17438a7e6f8a Calc!A360, `BINOM.DIST.RANGE(7,1E-20,6)`: Excel
  6.999999999999857E-120, engine 7.000000000000068E-120. The 50-digit value C(7,6)·p^6·(1-p) is
  6.99999999999999999993E-120, so Excel is about 2E-14 off the math and the engine about 1E-14 off it, in the other
  direction. One point cannot fit a form (decision 0026 and the gap-triager rubric: no blind fits).
- statistics.md "BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST are close to the exact value of their finite sums"
  (line about 1642) has the p grid already probed; no probe has p below 0.1 for RANGE.

## Acceptance criteria
- [ ] A probes-numeric case `t4361-binom-range-tiny-p` (training) with `.oracle.json` and `.math.json`: n in
      {1, 2, 3, 5, 7, 10, 30, 100, 1000}, p in {1E-300, 1E-200, 1E-100, 1E-50, 1E-20, 1E-16, 1E-10, 1E-6} and 2^-k
      neighbours of each, s = s2 for s in {0, 1, n-1, n} and short ranges [s, s+1], [0, 1], [n-1, n].
- [ ] A disjoint hold-out case `t4361-binom-range-tiny-p-hold` (other n and p, drawn after the training grid).
- [ ] Both oracle-filled on excel-graph and canary-verified; `make eval` runs them; the Outcome lists how many cells
      the current engine misses and the largest Excel-to-math error.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-4362).
- BINOM.DIST.RANGE past the mean at ordinary p (T-3740).
