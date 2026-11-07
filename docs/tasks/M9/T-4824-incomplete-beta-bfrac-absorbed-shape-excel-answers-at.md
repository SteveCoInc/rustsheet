---
id: T-4824
title: "Incomplete beta bfrac absorbed shape: Excel answers at x0 0.999 where 0.99 is #NUM!"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BETA.DIST's and NEGBINOM.DIST's cdf match Excel where bratio's bfrac branch has an absorbed second shape
(fl(a0 + b0) = a0) and x0 is near 1: Excel answers there (0, or 1 on the swapped call), from a mechanism found by a
probe with committed predictions, not a fitted x0 cut.

## Context
- T-4238 made bfrac `#NUM!` where the oriented first shape absorbs the second (`bratio::bfrac_shapes_absorbed`),
  verified at x0 0.1, 0.32, 0.5, 0.68, 0.9 and 0.99 with b0 41 to 1001, whole and fractional. At x0 0.999 Excel
  answers instead: probes/t4238-r3-absorb-whole-shape A1 `BETA.DIST(0.999,6E+17,41,TRUE)` = 0, A2 (b 41.5) 0, A6
  `(0.999,7E+17,57,TRUE)` 0, A10 `(0.001,41,6E+17,TRUE)` 1 (swapped); probes/t4238-r2-absorb-orientation A12
  `(0.999,7E+17,57.5,TRUE)` 0 and A13 `(0.001,57.5,7E+17,TRUE)` 1. Ours `#NUM!` at all six (known failures). Main
  matched all six before T-4238: they are T-4238's disclosed loss (decision 0082's precedent).
  A9 `(0.99,6E+17,41,TRUE)` is `#NUM!` in Excel, so the cut lies in x0 (0.99, 0.999).
- Suspects: TOMS bratio's lambda test (`lambda = (a0+b0)*y0 - b0`, bfrac below 0.03*a0) or Excel's bfrac prefactor
  ("Incomplete beta L140 and bfrac: the prefactor is Excel's own density times x·y") returning before the failing
  step. statistics.md "NEGBINOM.DIST's cdf is #NUM! at a huge number_s".
- Also there, Tier B: probes/t4238-r2-absorb-orientation A14, B14 (`BETA.DIST(1E-19,41,6E+17,..)`), digits close
  but not exact.

## Acceptance criteria
- [ ] A probe with predictions committed before the fill brackets the x0 cut and tests a stated mechanism; the six
      cells above pass again (restoring what T-4238 lost), and no t4238 cell regresses.
- [ ] statistics.md records the mechanism.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other callers of the incomplete beta (BINOM.DIST's family, F, T, BETA.INV).
