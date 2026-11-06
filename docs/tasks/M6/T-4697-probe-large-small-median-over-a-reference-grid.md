---
id: T-4697
title: "Probe LARGE/SMALL/MEDIAN over a reference grid with references on both the formula's sheet and another; tidy SPEC-0014 mixed-grid wording"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4569]
---

## Goal
Evidence decides what LARGE, SMALL and MEDIAN return over an array of references when some references are on the
formula's own sheet and some on another sheet. SPEC-0014's "Mixed grids" amendment (T-4569) gives this case
`#N/A` as an unprobed fallback.

## Context
- T-4569 (PR #3679, probes/t4569-xlookup-ref-element-max-min) showed the referenced sheet decides: `#VALUE!` when every
  reference is on the formula's own sheet (Local!E1:E3, F1:F5, F7:F8), `#N/A` when they are on another sheet
  (Local!E4:E5, F6; Calc A31:A32, A40:A44). Mixed sheets are unprobed (spec rule 5, "Order statistics" reader).
- Probe on sheet Local (data as t4569's Local and Data), predictions first:
  `=LARGE(INDIRECT({"A1";"Data!G1"}),1)`, `=LARGE(INDIRECT({"Data!G1";"A1"}),1)`, `=SMALL(..)` and `=MEDIAN(..)`
  of the same, and a mixed XLOOKUP grid whose hit and miss reference different sheets if one can be built.
- Review round 3 of T-4569 also left wording MINORs for the same spec section:
  - rule 3 says AVERAGEA, MINA and MAXA read the top-left cell "as a range cell"; they are `TopLeftElements` readers
    (an array element). Say "as the reader's `GridReading` reads a top-left cell".
  - rule 5 cites "round 4": cite the cells (Local!F1:F9) instead.
  - lookup.md's "MIN and MINA are 1 ... PRODUCT is 20" line runs past the wrap width.

## Acceptance criteria
- [ ] The probe is filled (excel-graph) and recorded in lookup.md and SPEC-0014 rule 5; the "Order statistics" reader
      states the evidenced rule instead of the fallback (amend T-4570 if the rule changes).
- [ ] The three wording fixes above are made.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The implementation (T-4570).
