---
id: T-4505
title: "Probe calculated fields and calculated items"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-4130]
---

## Goal
Evidence for calculated fields (on sums at every level, Count pivots, division by zero and other errors, references to non-value fields, constants, functions allowed and refused) and calculated items (formula over items per record, solve order, restrictions with grouping and several data fields).

## Context
Spike run 5; excel-docs.md quirks; [MS-OI29500] restrictions. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Probe cases with sidecars; pivots.md entries (calculated fields; calculated items cannot be probed: T-4792).
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
Calculated fields probed on excel-graph (2026-11-07) in two cases with sidecars:
probes/t4505-pivot-calc-fields (LV, LC, CNT, DZ, NV, K, FA, NM, CC) and
probes/t4505-pivot-calc-field-functions (FX, FR, FL, FK, FM, FN, FV, ET, EC, ES, EF; Q!A2:A8 the worksheet
controls). pivots.md "Calculated fields as Excel for the web renders them": every level, column totals included,
computes the formula over that cell's sums; Count, Average, Max and Min over a calculated field show the Sum
value; errors show only where the sums give them and do not reach totals; a text field is 0; a text constant is
#VALUE!; functions see each field as its sum (INDEX and ROWS, documented as not allowed, evaluate); quoted names
`'Unit Cost'` and `'Bob''s'`; a calculated field over another, also one listed later, works; a cycle is #NAME?.
Scratch runs found formulas that make the web refuse the file (NOW, TODAY, RAND, INDIRECT, an array constant, a
cell reference, an unknown name), added to "Pivot flags Excel for the web refuses".

Calculated items: no oracle can compute them (the web refuses or ignores the builder's, excel-desktop is n/a and
out of scope by decision 0266, and no Excel-saved file with one was found), so their rules are UNVERIFIED with
probe task T-4792 (filed), which T-4175 now depends on. The new cases' pivot cells are owned by T-4174.
