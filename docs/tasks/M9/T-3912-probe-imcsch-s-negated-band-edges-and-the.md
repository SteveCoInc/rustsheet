---
id: T-3912
title: "Probe IMCSCH's negated-band edges and the subnormal-y cut finer; tidy T-3881's R4 and citation wording"
milestone: M9
lane: functions-engineering
status: in-progress
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
- [ ] A probes case (excel-graph) covers the points above. Its values are recorded in engineering.md, and the cut
      either holds (the brackets narrow to within an ulp or the case shows where it lies) or moves to the evidence,
      with a decision on what the code takes.
- [ ] engineering.md and T-3881's Outcome are reworded as listed under Context.
- [ ] No eval regressions; the new case's failing cells, if any, have owners.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSECH's last digit in the band (T-3883) and 705+705i (T-3851).
