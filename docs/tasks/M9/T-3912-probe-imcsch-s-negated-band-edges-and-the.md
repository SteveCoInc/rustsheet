---
id: T-3912
title: "Probe IMCSCH's negated-band edges and the subnormal-y cut finer; tidy T-3881's R4 and citation wording"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMCSCH's negated band (T-3881) has edges and a subnormal-y cut that oracle cells bracket tightly. T-3881's
records say what the evidence shows and no more.

## Context
- T-3881 (PR #2952) implements the band as 2x > 709 or 2x < -708 (`in_csch_negated_band` in
  crates/rustsheet-engine/src/functions/engineering/complex/trig.rs). It reads that cut off two brackets:
  `IMCSCH(COMPLEX(354.5,1))` is true and `(354.501,1)` is negated; `(-354,1)` is true and `(-354.01,1)` is negated
  (probes/t3881-imcsch-flip-start, -flip-start-negative). It also treats y as real below `f64::MIN_POSITIVE`.
  That cut is bracketed only between 1E-308 (counts as real) and 1E-300 (does not; t3881-imcsch-overflow-band).
- Follow-up of T-3881's excel-fidelity review. Its MINORs:
  - Probe the edges finer: x = 354.5001 to 354.5009 and the next double above 354.5, and x = -354.001 to -354.009
    and the next double below -354. Probe y between 1E-308 and 1E-300, for example 2.2250738585072E-308
    (DBL_MIN), 2.3E-308 and 1E-305. Use the excel-probe skill, with IMSECH companions on the same z.
  - engineering.md's T-3881 paragraph justifies 709 and -708 as "the usual overflow and underflow limits of exp".
    Cite instead the same file's evidenced IMCOSH/IMSINH test, "Past x = 709 ... or below x = -708"
    (`hyperbolic_log_form`), and say the subnormal-y cut is inferred within (1E-308, 1E-300) until this probe pins it.
  - T-3881's Outcome calls the cut "kind (c)". No candidates were ranked, so R4 is not triggered: reword it, and mark
    the R4 sub-boxes N/A.
- Design MINOR (same review): `imcsch` checks `past_trig_limit(y) && !reciprocal_flushes(x)` before the band branch,
  and `sech_parts` repeats the check. Both give `#NUM!`, so the early check can move after the band branch.

## Acceptance criteria
- [x] A probes case (excel-graph) covers the points above. Its values are recorded in engineering.md, and the cut
      either holds (the brackets narrow to within an ulp or the case shows where it lies) or moves to the evidence,
      with a decision on what the code takes.
- [x] engineering.md and T-3881's Outcome are reworded as listed under Context.
- [x] No eval regressions; the new case's failing cells, if any, have owners.
- [x] `make check` green; reviewers approve.

## Out of scope
- IMSECH's last digit in the band (T-3883) and 705+705i (T-3851).

## Outcome
- New probe probes/t3912-imcsch-band-edges (excel-graph, 2026-11-03, 174 cells; IMCSCH column A, IMSECH column B).
  - Edges: every point from 354.500000000001 to 354.5009 is negated (A1:A18, A64:A70), and so is every point from
    -354.000000000001 to -354.009 (A19:A36, A71:A77). 354.5 and -354 are the true csch. Each edge is therefore
    pinned to one step of the 15-digit input grid (1E-12 at |x| = 354); column C shows COMPLEX's text keeps the 1E-12
    offsets.
  - Input grid: nothing finer can be typed. Complex text drops digits past the 15th, a 16th digit 5 included, so
    `IMREAL("354.50000000000006+i")-354.5`, `"354.5000000000001+i"` and `"354.5000000000005+i"`, and the same at
    -354, are 0 (C37:C42), and those cells give the edge's own value. The cut 2x > 709 or 2x < -708 holds to the
    resolution Excel accepts; the code keeps it.
  - Subnormal y: the y cut is DBL_MIN, and it comes from the parse. `IMAGINARY(COMPLEX(355,2.2250738585072E-308))`
    is 0 (below DBL_MIN, read back as 0), while 2.2250738585073E-308 and 2.3E-308 are kept (C61:D63). So
    (+-355, 2.2250738585072E-308) keeps the true sign, and 2.2250738585073E-308 up to 1E-301 is negated (A43:A60).
    The code's `y.abs() < f64::MIN_POSITIVE` holds.
- Docs: engineering.md's T-3881 paragraph now cites IMCOSH/IMSINH's log-form cuts (`hyperbolic_log_form`) instead of
  "the usual limits of exp", and records the narrowed brackets and the parse-side y cut. T-3881's Outcome no longer
  says "kind (c)"; its R4 sub-boxes are marked N/A, since no candidates were ranked.
- Code (design MINOR): `imcsch`'s early `past_trig_limit(y) && !reciprocal_flushes(x)` check moved after the band
  branch, where `sech_parts` makes the same check. The behaviour is unchanged. The unit test
  `imcsch_negates_csch_in_the_band` gains (354.50001, 1), (-354.0001, 1) and (355, 2.2250738585073E-308).
- Cells: 172 of 174 pass. A25/B25 (`IMSECH(COMPLEX(-354.007,1))` and its IMCSCH twin) carry IMSECH's sign-dependent
  last digit, which is T-3914's cell kind; T-3914 now owns them.
- Eval delta: probes gains 1 case with 2 new-coverage known failures (owned by T-3914). Fixed 0, regressions 0.
