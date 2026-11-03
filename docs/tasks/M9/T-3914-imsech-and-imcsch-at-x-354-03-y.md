---
id: T-3914
title: "IMSECH and IMCSCH at x = ±354.03, y = 1: Excel's last imaginary digit depends on the sign of x (T-3883's sweep A101)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMSECH's finite-t form (t = sinh²x + cos²y does not overflow, |x| between about 354 and 355.6) prints Excel's
imaginary digit at x = -354.03 and +354.03, which differ in the 15th digit although the true values are mirror images.

## Context
- Evidence (excel-graph, 2026-11-03, T-3883): probes/t3883-im-overflow-reciprocal-sweep A101
  `IMSECH(COMPLEX(-354.03,1))` 1.90717870311855E-154+2.97025484453415E-154i (ours ...453414E-154i), A102
  `IMSECH(COMPLEX(354.03,1))` 1.90717870311855E-154-2.97025484453414E-154i and A103 `IMCSCH(COMPLEX(354.03,1))`
  the same text (both pass). probes/t3881-imcsch-flip-start-negative A3 `IMCSCH(COMPLEX(-354.03,1))` is the IMCSCH twin of
  A101 (-IMSECH there, T-3881) with the same off digit.
- Code: `sech_parts` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs, finite-t branch
  `(x_cosh * y_cos / t, -x_sinh * y_sin / t)` with `t = x_sinh.mul_add(x_sinh, y_cos²)`; past |x| = 20 sinh and cosh are
  libm's (`inverse_exp_sinh_cosh`).
- T-3883 tried, on the two cells only: libm, the `1/u` form `inverse_exp_form`, Excel's EXP(|x|)/2 and libm exp(|x|)/2 as
  sinh/cosh, each with the fused and the unfused t, and the division `cos y / M`. Every symmetric form gives the same digit
  at ±354.03 (414 with t, 415 with the division); `inverse_exp_form` is lopsided the wrong way (414 at -354.03, 415 at
  +354.03). So Excel's form is lopsided as if u = EXP(x) rather than EXP(-x), or it switches form between the two
  points; a sweep of x in ±(340..355.6) with a few y is the next probe. Ranking forms is a structural choice: the R4
  guards of reviewer-excel-fidelity check 7 apply.

## Acceptance criteria
- [ ] probes/t3883-im-overflow-reciprocal-sweep A101 and probes/t3881-imcsch-flip-start-negative A3 match Excel's text,
      with no other IMSECH or IMCSCH cell regressing; their known-failure lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The overflowed form past sinh²x's overflow (T-3883).
