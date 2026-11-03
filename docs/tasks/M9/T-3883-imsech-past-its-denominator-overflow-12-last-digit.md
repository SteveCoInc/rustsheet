---
id: T-3883
title: "IMSECH past its denominator overflow: 12 cells with a last-digit part off (T-3881's band probe)"
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
cells listed under Context, where one part is one digit off today.

## Context
- Found by T-3881's probe (excel-graph, 2026-11-03). Every IMSECH cell of the case passes except
  probes/t3881-imcsch-overflow-band.case.json Calc!B45, B51, B70, B76, B82, B88, B149, B155, B174, B180, B186 and
  B192: x = ±400 with y = ±20, x = ±600 with y = ±5.3113 and x = ±699 with y = ±1. Examples:
  `IMSECH(COMPLEX(400,20))` Excel 1.56309271549875E-174-3.49688997531599E-174i, ours ...-3.496889975316E-174i;
  `IMSECH(COMPLEX(699,1))` Excel 2.89616943787016E-304-..., ours 2.89616943787015E-304-.... The imaginary part is
  off at x = ±400 and ±600; the real part is off at x = ±699 (B82, B88, B186, B192).
- Code: `overflowed_reciprocal` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs takes
  `2.0 * (-v.abs()).exp()` with libm's exp. Candidates: Excel's own EXP (`x87_exp`), or exp(-|x|) times sin y before
  the factor 2 (and the same for cos y in the real part). Ranking candidates is a structural (kind (c)) choice: the
  R4 guards of reviewer-excel-fidelity check 7 apply (hold-out, fresh sweep, Gate A envelope).
- T-3881 found that inside this band Excel's IMCSCH of a non-real z equals -sgn(x)·IMSECH(z) digit for digit, and
  IMCSCH now reuses IMSECH's parts there (`sech_parts`). So this task also owns the IMCSCH twins of the 12 cells,
  probes/t3881-imcsch-overflow-band.case.json Calc!A45, A51, A70, A76, A82, A88, A149, A155, A174, A180, A186 and A192,
  and probes/t3881-imcsch-flip-start-negative.case.json Calc!A3 (`IMCSCH(COMPLEX(-354.03,1))`, ours
  ...2.97025484453414E-154i against Excel's ...453415E-154i). A3 is in IMSECH's finite-t form (t = sinh²x + cos²y does
  not overflow at x = -354.03), not `overflowed_reciprocal`; its IMSECH twin is unprobed.

## Acceptance criteria
- [ ] The 12 cells (and the 13 IMCSCH cells above) match Excel's text, and no other IMSECH, IMSEC or IMCSC cell (the users of `overflowed_reciprocal`)
      regresses; their known-failure lines leave eval/baseline/probes.txt.
- [ ] R4 guards recorded in the Outcome if candidates are ranked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMCSCH's band and sign (T-3881).
