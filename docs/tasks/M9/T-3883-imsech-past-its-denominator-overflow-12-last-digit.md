---
id: T-3883
title: "IMSECH past its denominator overflow: 12 last-digit imaginary parts (t3881-imcsch-overflow-band)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
IMSECH's overflowed-reciprocal form (`overflowed_reciprocal`, scale 2e^-|x|) prints Excel's 15-digit text on the 12
cells of probes/t3881-imcsch-overflow-band where it is one digit off today.

## Context
- Found by T-3881's probe (excel-graph, 2026-11-03). Every IMSECH cell of the case passes except
  probes/t3881-imcsch-overflow-band.case.json Calc!B45, B51, B70, B76, B82, B88, B149, B155, B174, B180, B186 and
  B192: x = ±400 with y = ±20, x = ±600 with y = ±5.3113 and x = ±699 with y = ±1. Examples:
  `IMSECH(COMPLEX(400,20))` Excel 1.56309271549875E-174-3.49688997531599E-174i, ours ...-3.496889975316E-174i;
  `IMSECH(COMPLEX(699,1))` Excel 2.89616943787016E-304-..., ours 2.89616943787015E-304-....
- Code: `overflowed_reciprocal` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs takes
  `2.0 * (-v.abs()).exp()` with libm's exp. Candidates: Excel's own EXP (`x87_exp`), or exp(-|x|) times sin y before
  the factor 2. Ranking candidates is a structural (kind (c)) choice: the R4 guards of reviewer-excel-fidelity check 7
  apply (hold-out, fresh sweep, Gate A envelope).
- T-3881 found that inside this band Excel's IMCSCH of a non-real z equals -sgn(x)·IMSECH(z) digit for digit, so a
  fix here should carry to T-3881's IMCSCH texts if IMCSCH reuses this form.

## Acceptance criteria
- [ ] The 12 cells match Excel's text, and no other IMSECH, IMSEC or IMCSC cell (the users of `overflowed_reciprocal`)
      regresses; their known-failure lines leave eval/baseline/probes.txt.
- [ ] R4 guards recorded in the Outcome if candidates are ranked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMCSCH's band and sign (T-3881).
