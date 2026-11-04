---
id: T-3914
title: "IMSECH and IMCSCH at x = ±354.03, y = 1: Excel's last imaginary digit depends on the sign of x (T-3883's sweep A101)"
milestone: M9
lane: functions-engineering
status: done
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
- probes/t3912-imcsch-band-edges (excel-graph, 2026-11-03, T-3912) adds one more: Calc!B25 `IMSECH(COMPLEX(-354.007,1))`
  1.9515521518382E-154+3.03936239633971E-154i (ours ...633972E-154i) and its IMCSCH twin Calc!A25 (negated band, the
  same text). The other 34 points at x = -354.0001 to -354.009 and +354.5001 to +354.50009 pass.
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
- [ ] probes/t3883-im-overflow-reciprocal-sweep A101 and probes/t3881-imcsch-flip-start-negative A3 (and
      probes/t3912-imcsch-band-edges Calc!A25 and B25) match Excel's text,
      with no other IMSECH or IMCSCH cell regressing; their known-failure lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The overflowed form past sinh²x's overflow (T-3883).

## Outcome
Findings, no engine change: Excel's IMSECH is symmetric in the sign of x across the finite-t form, and the ±354.03
asymmetry is sub-ulp noise at a 15-digit rounding midpoint, below the reproducibility floor (decision 0188 R4.4).
The four cells move to T-4001 (`defer: phase-1b`, decision 0114) with the four sub-ulp cells the probe added, so the
first acceptance box is not met; the task's question is answered instead.
- Probe: probes/t3914-imsech-sign-sweep (excel-graph, 2026-11-04), 600 cells: IMSECH and IMCOSH at x+yi and -x+yi,
  |x| in [340, 355.5] (rows 1-50) and log-uniform in [20, 355.5] (rows 51-150). Excel prints the same digits at ±x in
  every one of its 150 IMSECH pairs and 150 IMCOSH pairs.
- In each off cell the exact value (50-digit mpmath) is 0.16 to 0.44 ulp from the 15-digit midpoint, and Excel is
  not consistent about the side: it prints the correctly rounded digit at -354.03 and -354.007 and the other one at
  +354.03, ±346.218848 (A11:B11) and ±353.877487 (IMCOSH C21:D21), where ours is the correctly rounded one.
- Forms ranked (scratch harness, not committed), split fixed before scoring: train = sweep rows not a multiple of 3
  (200 IMSECH cells) plus the 292 older finite-t IMSECH and band IMCSCH cells, hold-out = rows that are a multiple of
  3 (100 cells), margin 10 points. Ours (libm, unfused t): 198/200 train, 100/100 hold-out, 270/292 older. Excel's EXP
  pair: 200/200, 100/100, 270/292; it gains only the sub-ulp A11:B11 pair, so it was not taken (R4.4). The `1/u` form
  (either u), the complex reciprocal, Smith's division, `(cosh 2x + cos 2y)/2` and its log-form variant all score lower
  and none gives the ±354.03 pair. engineering.md "IMSECH's last digit at 15-digit midpoints, |x| 20 to 355.5".
- R4: no model or constant changed, so the guards are not triggered (hold-out shown above for the record; fresh
  sweep, Gate A envelope and `FITTED` N/A).
- Eval delta: probes gains 1 case (600 cells, 596 pass) with 4 new-coverage known failures (A11:B11, C21:D21, owned by T-4001). Fixed 0, regressions 0.
- Follow-up: T-4001 (deferred) owns probes/t3883-im-overflow-reciprocal-sweep A101,
  t3881-imcsch-flip-start-negative A3, t3912-imcsch-band-edges A25:B25 and t3914-imsech-sign-sweep A11:B11, C21:D21.
