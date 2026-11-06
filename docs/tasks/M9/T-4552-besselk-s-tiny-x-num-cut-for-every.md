---
id: T-4552
title: "BESSELK at x = 2^-1021 exactly, and whether BESSELJ, BESSELI and BESSELY share the x/2-subnormal #NUM! (T-4459 follow-up)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's BESSELK at x = 2^-1021 (twice the smallest normal, reachable only as a computed value) is known, and so is
whether BESSELJ, BESSELI and BESSELY are also `#NUM!` once `x/2` is subnormal. The engine matches both.

## Context
- T-4459 found that BESSELK of every order is `#NUM!` from 4.4501477170144E-308 down and answers from
  4.45014771701441E-308 up (probes/t4459-besselk-tiny-x-ulps; Excel reads literals at 15 significant digits, so its
  ulp-spaced literals collapse onto those two values). 2^-1021 = 4.4501477170144028E-308 lies between, so the cells fit
  "`x/2` subnormal is `#NUM!`" (engineering.md "BESSELK at a tiny x"). The engine cuts at `x < 2 * f64::MIN_POSITIVE`
  (crates/rustsheet-engine/src/functions/engineering/transcendental/bessel_k.rs).
- x = 2^-1021 itself decides between `x/2 < DBL_MIN` and `x/2 <= DBL_MIN`-style checks. It must be built by a formula,
  for example `2*2.2250738585072E-308+...`; check first that such an x lands on 2^-1021 exactly (`(x)/2^-1021`).
- BESSELY's T-4458 probe stops at 3E-308 for order 1 (it answers); BESSELJ/BESSELI at these x are unprobed.

## Acceptance criteria
- [x] A probe under eval/suites/probes (excel-graph) asks BESSELK orders 0 and 1 at x = 2^-1021 exactly, and BESSELJ,
      BESSELI and BESSELY orders 0 and 1 at 4.4501477170144E-308 and 4.45014771701441E-308.
- [x] engineering.md records the answers; the engine matches every cell; no new known-failure lines.
- [x] `make check` green; reviewers approve.

## Out of scope
- BESSELK's recurrence cuts for orders 2 to 7 (T-4459, done).

## Outcome
- probes/t4552-bessel-tiny-x-cut (33 cells, excel-graph 2026-11-06; engine predictions in docs/research/t4552
  before the fill):
  - x = 2^-1021, built as `2^-1021` and `2^-1000/2^21`, lands exactly (A1:A5).
  - BESSELK orders 0 and 1 answer there (707.8192028722641, 2.247116418577895E+307), so the engine's `x < 2^-1021`
    cut was already right.
  - BESSELJ, BESSELI and BESSELY orders 0 and 1 are never `#NUM!` at 4.4501477170144E-308, 4.45014771701441E-308,
    2^-1021 or 2^-1022.
  - BESSELJ and BESSELI return 0 for a subnormal result: `BESSELJ(2^-1022,1)`, `BESSELI(4.4501477170144E-308,1)` and
    `BESSELI(2^-1022,1)` (C8, D2, D8).
- `fn_besselj` and `fn_besseli` now flush a subnormal result with `flush_subnormal`, as BESSELK and ERFC already did.
  engineering.md updates "BESSELK at a tiny x" and adds "BESSELJ, BESSELI and BESSELY at the bottom of the normal
  range". New unit test `bessel_results_at_the_bottom_of_the_normal_range`.
- Eval delta: 1 new case with all 33 cells passing (the engine predicted 30 of them), no new known-failure lines, and
  0 regressions in any suite.
- Also filed T-4648 (the T-4594 review MINORs).
