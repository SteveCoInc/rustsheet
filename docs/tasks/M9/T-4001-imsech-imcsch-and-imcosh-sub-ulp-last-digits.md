---
id: T-4001
title: "IMSECH, IMCSCH and IMCOSH sub-ulp last digits at 15-digit rounding midpoints, |x| 340..355 (deferred)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
defer: phase-1b
depends_on: []
---

## Goal
IMSECH, IMCSCH and IMCOSH print Excel's 15th digit at the points below, where the true value lies within 1 ulp
of a 15-digit rounding midpoint, without losing any other IM* cell.

## Context
- Follow-up of T-3914, which found no form for these cells (engineering.md "IMSECH's last digit at 15-digit
  midpoints, |x| 20 to 355.5", T-3914). In each cell the exact value (50-digit mpmath) is 0.16 to 0.44 ulp from
  the 15-digit midpoint, and ours and Excel's doubles print on opposite sides of it. Against the exact value
  rounded to 15 digits, Excel is right at 3 of the 5 points (-354.03, -354.007, ±346.218848) and off at +354.03
  and ±353.877487 (IMCOSH); ours is right only at ±353.877487. At ±346.218848 even the correctly rounded double
  prints our digits, so only an evaluation more accurate than a double result (or Excel's own rounding path)
  reaches Excel's text there. Every symmetric form T-3914 scored (libm, Excel's EXP pair, the
  `1/u` form, with the fused or unfused t, the complex reciprocal, Smith's division, and the `cosh 2x` denominator)
  gives the same digit at ±x. All differences are below 1 ulp, under the oracle's reproducibility floor
  (decision 0188 R4.4: web against desktop up to 4 ulp; docs/research/oracle-calibration.md), so they are
  not fit targets, and the task is deferred (decision 0114). A more accurate kernel could reach Excel's text at
  its 3 exact points, but not at C21:D21, and it would lose +354.03 (probes/t3883-im-overflow-reciprocal-sweep
  A102, which passes now because ours and Excel are both inexact there). So no accuracy fix covers all 8 cells,
  and a model of Excel's formulation falls under decision 0056's time-box.
- Cells (known failures in eval/baseline/probes.txt):
  - probes/t3883-im-overflow-reciprocal-sweep Calc!A101 `IMSECH(COMPLEX(-354.03,1))`
  - probes/t3881-imcsch-flip-start-negative Calc!A3 `IMCSCH(COMPLEX(-354.03,1))`
  - probes/t3912-imcsch-band-edges Calc!A25 and Calc!B25 (x = -354.007, y = 1)
  - probes/t3914-imsech-sign-sweep Calc!A11 and Calc!B11 (IMSECH, x = ±346.218848, y = 2.457207)
  - probes/t3914-imsech-sign-sweep Calc!C21 and Calc!D21 (IMCOSH, x = ±353.877487, y = 1.153473)
- Code: `sech_parts` and `imcosh` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

## Acceptance criteria
- [ ] The cells above match Excel's text, with no other IM* cell regressing, by a form that passes the R4 guards
      of reviewer-excel-fidelity check 7 (hold-out, fresh sweep, with the scoring harness committed so the split
      is reproducible); or the task records why none exists.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSECH past the denominator's overflow (T-3883, done) and IMCSCH's negated band edges (T-3912, done).
