---
id: T-3883
title: "IMSECH past its denominator overflow: 12 cells with a last-digit part off (T-3881's band probe)"
milestone: M9
lane: functions-engineering
status: done
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
- [x] The 12 cells (and 12 of the 13 IMCSCH cells above; A3 to T-3918) match Excel's text, and no other IMSECH, IMSEC or IMCSC cell (the users of `overflowed_reciprocal`)
      regresses; their known-failure lines leave eval/baseline/probes.txt.
- [x] R4 guards recorded in the Outcome if candidates are ranked.
- [x] `make check` green; reviewers approve.

## Out of scope
- IMCSCH's band and sign (T-3881).

## Outcome
- `overflowed_parts` (complex/trig.rs), the overflowed form of IMSEC, IMCSC, IMSECH and IMCSCH's band, now divides the
  trigonometric parts by M = cosh |v| (libm's) while M is finite, where it multiplied them by 2e^-|v|. Past
  |v| ≈ 710.48 cosh overflows and the old scale stays. engineering.md "Tiny results" (the T-2335 entry) states it.
- Cells: the 12 IMSECH cells and 12 of the 13 IMCSCH twins pass, and so do T-3851's four 705+705i cells (grid D161,
  E161, I161, J161; T-3851 closed with this PR). flip-start-negative A3 is in IMSECH's finite-t form, not this one,
  and stays: T-3918 (phase-1b).
- R4 (kind (c), structural: an operation order, so no `FITTED` tag). Candidates ranked on all 4,140 oracle texts of
  the four functions: 2e^-|v| times the parts (libm or Excel's EXP, either order) 4,030; parts over e^|v|/2, over
  libm cosh or over `inverse_exp_sinh_cosh`'s cosh 4,058 (tied); Smith's complex division of 1/cosh z 4,026.
  - Hold-out (R4.1): the fit lab's split (`fitlab.holdout`, T-3360) over the 1,474 cells that take this path; margin
    5 points (the default), fixed before scoring.

    | candidate | side | exact | 4 ulp | 15 digits | total |
    |---|---|---|---|---|---|
    | chosen (parts / cosh) | train | 1017 | 1017 | 1017 | 1017 |
    | chosen (parts / cosh) | hold-out | 457 | 457 | 457 | 457 |
    | correctly rounded | train | 288 | 332 | 446 | 1017 |
    | correctly rounded | hold-out | 113 | 137 | 192 | 457 |
    | old (2e^-v product) | train | 994 | 994 | 994 | 1017 |
    | old (2e^-v product) | hold-out | 452 | 452 | 452 | 457 |

    The correctly rounded column is low because Excel's IMCSCH is negated in its band (T-3881) and Excel flushes
    subnormal parts.
  - Fresh sweep (R4.2): probes-numeric/t3883-overflow-reciprocal-sweep, 200 cells (IMSECH, IMCSCH of h+ti; IMSEC,
    IMCSC of t+hi; |h| log-uniform in [355, 711], |t| in [1e-3, 1e4]; docs/research/t3883/gen_sweep.py, seed 3883),
    generated and filled from excel-graph after the choice was frozen: 200 of 200 pass (main's product 192).
  - Gate A envelope (R2): all 200 within 15 digits of Excel (each text equal), none outside.
  - Reproducibility floor (R4.4): each change is a whole 15th digit of the printed text, above the 4 ulp floor.
  - No keep-matched branch: the cosh form applies to the whole path; the 2e^-|v| fallback is where cosh overflows,
    not where the old form matched.
- Unit test `overflowed_reciprocals_divide_by_cosh`.
- Eval delta: probes known failures 4995 -> 4967 (fixed 28: 24 here, 4 of T-3851; regressions 0, no new line).
