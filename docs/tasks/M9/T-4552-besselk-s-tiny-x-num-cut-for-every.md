---
id: T-4552
title: "BESSELK at x = 2^-1021 exactly, and whether BESSELJ, BESSELI and BESSELY share the x/2-subnormal #NUM! (T-4459 follow-up)"
milestone: M9
lane: functions-engineering
status: todo
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
- [ ] A probe under eval/suites/probes (excel-graph) asks BESSELK orders 0 and 1 at x = 2^-1021 exactly, and BESSELJ,
      BESSELI and BESSELY orders 0 and 1 at 4.4501477170144E-308 and 4.45014771701441E-308.
- [ ] engineering.md records the answers; the engine matches every cell; no new known-failure lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BESSELK's recurrence cuts for orders 2 to 7 (T-4459, done).
