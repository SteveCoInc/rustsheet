---
id: T-3930
title: "IMSECH, IMSINH, IMCOSH and IMCSCH at |x| <= 0.01: the 8 last-digit texts the three-term series leaves (t3721, t3720)"
milestone: M9
lane: functions-engineering
status: in-progress
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
