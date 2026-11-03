---
id: T-3930
title: "IMSECH, IMSINH, IMCOSH and IMCSCH at |x| <= 0.01: the 8 last-digit texts the three-term series leaves (t3721, t3720)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3721]
campaign: visible
---

## Goal
The 8 IM* texts at 0 < |x| <= 0.01 that T-3721's three-term sinh series leaves one 15-digit tie off match Excel, or the
Outcome ranks the candidates that leave them and records them as UNVERIFIED. The kernel is chosen from ranked candidates on a
sweep, not on these 8 cells alone.

## Context
- Filed by T-3721 (review follow-up of its fresh sweep, decision 0245). At each of these x the three-term and the
  four-term series give the same double, so the series is not the lead: the candidates are cosh x (the `1/u` form
  with Excel's EXP against the alternatives T-3721 ranked: libm, the worksheet COSH, `sqrt(1 + sinh²x)`), the
  IMSECH denominator (`mul_add` of sinh²x into cos²y) and the IMCSCH quotient. engineering.md "IMSECH and IMCSCH of a
  small real number" lists what T-3721 ranked; T-3720's sweep carries Excel's own SIN(y), COS(y), SINH(x), COSH(x)
  and EXP(-x) for its rows.
- Fit lab: `im-small-x-complex` (T-3720's sweep) and `im-small-x-complex-fresh` (T-3721's).
- Code: `inverse_exp_sinh_cosh`, `sech_parts`, `imcsch` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.
- Cells owned (Excel, then ours):
  - probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep.case.json (4): S!C311
    `IMSECH("-8.74352867749883E-05+1.16398498777952i")` 2.52727602503685+0.000512882556578914i, ours ...684+...;
    S!C386 `IMSECH("-0.00529114829570232-0.0605498694819928i")` 1.0018218211334-0.00032135183420584i, ours
    ...-0.000321351834205841i; S!C390 `IMSECH("-0.00802240437783925")` 0.99996782137691, ours 0.999967821376909;
    S!D169 `IMSINH("8.76018807937593E-05-0.32913753686474i")` 0.0000828995404338025-0.323226982572063i, ours ...064i.
  - probes-numeric/t3720-im-reciprocal-small-x-complex.case.json (4): S!A3 `IMCOSH("3.58722248184392E-06+0.256378854055322i")`
    real part 0.967314566321166 (ours ...165); S!D7 `IMSINH("-1.46870335270024E-05+0.976331101116213i")`; S!B25
    `IMCSCH("2.5814545068154E-05-0.118602492728453i")` imaginary part 8.45132516126195 (ours ...194); S!B148
    `IMCSCH("2.0102264195349E-05+0.245849374343589i")` imaginary part -4.10879685561039 (ours ...038).

## Acceptance criteria
- [ ] The 8 owned cells match Excel's text, or the Outcome names each one left over with the candidate scores.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188) for any kernel change: hold-out on the fit lab's split,
      a fresh sweep after the freeze, Gate A envelope, `FITTED` tag or kind (c), no keep-matched branch.
- [ ] Every other IM* hyperbolic oracle cell keeps passing (the fit lab's `im-hyperbolic*`, `im-small-x-complex*`,
      `im-small-imag` sets, gen-m9).
- [ ] engineering.md's "IMSECH and IMCSCH of a small real number" entry records the result.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when the 8 owned text cells equal Excel's text; each leftover is recorded as
UNVERIFIED with the ranked candidates.

## Out of scope
- The sinh series (T-3721). |x| above 0.01.

## Outcome
6 of the 8 owned texts now match Excel (C311, D169, A3, D7, B25, B148). The other 2 are
recorded UNVERIFIED with their candidate scores (engineering.md "IMSINH, IMCOSH, IMSECH and IMCSCH take libm's cosh
below |x| 1E-4").
- **Change**: at |x| < `LIBM_COSH_BELOW` (1E-4) `inverse_exp_sinh_cosh` takes libm's cosh instead of the `1/u` form's.
  This is a fitted cut-point (decision 0188 R4 kind (a)), tagged `FITTED(T-3930, 39 cells, |x| 3.6E-06..0.01; cut
  bracketed 8.8E-05..1.09E-04)` and applied only inside `INVERSE_EXP_LOW`.
- **Ranked candidates**, by engine build over the 130 cases with IMSINH, IMCOSH, IMSECH or IMCSCH:
  - libm cosh everywhere at |x| <= 0.01 fixes the 6 and breaks 33. All 33 are at |x| >= 1.09E-4, and all 6
    fixed ones at |x| <= 8.8E-5.
  - The cut fixes the same 6 and breaks none.
  - A real/complex split does not separate them: the broken texts are complex too.
- **Left over, UNVERIFIED**:
  - C386 (IMSECH, every cosh candidate prints ...841).
  - C390: the real IMSECH fits the form's `1/cosh` but not T-3609's libm `1/cosh`, which won 75 to 72 on T-3609's
    sweep. Changing it would refit T-3609.
- **Hold-out (R4.1)**: fit lab split (T-3360, margin 5 points), exact / within 4 ulp / within 15 digits:

  | fit | candidate | train | test | verdict |
  |---|---|---|---|---|
  | im-small-x-complex | cut | 1131 / 1131 / 1131 | 485 / 485 / 485 | accept |
  | im-small-x-complex | form only | 1128 / 1128 / 1128 | 484 / 484 / 484 | accept |
  | im-small-x-complex | libm only | 1117 / 1117 / 1117 | 482 / 482 / 482 | accept |
  | im-small-x-complex | correctly rounded | 1052 / 1052 / 1052 | 448 / 448 / 448 | reference |
  | im-small-x-complex-fresh | cut | 1100 / 1100 / 1100 | 498 / 498 / 498 | accept |
  | im-small-x-complex-fresh | correctly rounded | 1036 / 1036 / 1036 | 469 / 469 / 469 | reference |

  The cut was chosen by looking at these cells before the split was scored, so the fresh sweep is the clean guard.
- **Fresh sweep (R4.2)**: probes-numeric/t3930-r4-im-small-x-cosh (seed 3930, `docs/research/t3930/gensweep.py`).
  Generated and filled from excel-graph after the cut froze at edb75c46a: 480 texts, half of them at |x|
  3E-5..3E-4. Scores: cut 480, form only 475, libm only 476.
- **Gate A envelope**: all 480 sweep texts are within 15 digits of Excel, so none is outside. No IM* Gate A table
  exists for an `r4-T-3930` batch, as in T-3721 and T-3795.
- **Reproducibility floor**: each fixed cell is a whole printed digit of a text result. **No keep-matched branch**:
  the cut is one |x| threshold, bracketed by the evidence, not a cell list.

Eval delta: probes-numeric t3720-im-reciprocal-small-x-complex 4 texts and t3721-im-hyperbolic-small-x-fresh-sweep 2
newly pass (not baselined suites); gated baselines unchanged, REGRESSIONS 0; new case t3930-r4-im-small-x-cosh 480/480.
