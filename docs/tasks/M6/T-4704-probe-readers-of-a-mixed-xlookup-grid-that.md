---
id: T-4704
title: "Probe readers of a mixed XLOOKUP grid that T-4570 left unprobed (FORMULATEXT, OFFSET, RANK, error beside an off-sheet reference, both-sheet grids)"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4570]
---

## Goal
Excel evidence for each reader of a mixed XLOOKUP grid (SPEC-0014 amendment "Mixed grids") that T-4570 implemented
by fallback, so every such choice is either confirmed or filed as a fix.

## Context
- T-4570 (PR #3757) left these choices unprobed, each labelled so in the code and in SPEC-0014 "As implemented
  (T-4570)" (Excel-fidelity review, MINOR):
  - FORMULATEXT over `XLOOKUP({2;9},A1:A3,B1:B3,F1:F2)`: the value element gives `#VALUE!`
    (`functions/lookup_and_reference/formulatext.rs`).
  - OFFSET over such a grid: the value element gives `#VALUE!` (`offset.rs`). Also INDIRECT and TAKE/DROP over one.
  - RANK, RANK.EQ and RANK.AVG over a grid of references on another sheet: they keep the value view, while LARGE and
    SMALL give `#N/A` (`statistical/chisq.rs numbers_from_ranked_arg`).
  - LARGE, SMALL and MEDIAN over a grid holding an error element beside an off-sheet reference: the error comes
    first (`RefGrid::order_statistic_error`).
  - A grid with references on both the formula's sheet and another: `#N/A`.
  - A per-element caller over an XLOOKUP with a non-range if_not_found, or with every lookup a hit
    (`COUNTIF(XLOOKUP({2;9},A1:A3,B1:B3,"x"),">0")`): the static analysis predicts the lookup_value's shape, but
    ours gives one scalar. Probe the cell's spill shape too.
- Use the excel-probe skill. Lay out the data as probes/t4569-xlookup-ref-element-max-min does (a Data sheet read
  from Calc, and a Local sheet read from itself).

## Acceptance criteria
- [ ] A probe case with oracle values covers every bullet above.
- [ ] docs/excel-semantics/lookup.md records the results. Each mismatch gets its own fix task (or is fixed here when
      size S), and each match updates SPEC-0014 "As implemented (T-4570)" from fallback to evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- One-index INDEX of a mixed grid (T-4701).

## Outcome
Probe `probes/t4704-mixed-xlookup-grid-readers` (Data and Calc, 29 probes of three columns each, 103 cells, excel-graph,
2026-11-07). Results are in lookup.md (after "MEDIAN, LARGE and SMALL over a lifted OFFSET") and SPEC-0014 "As
implemented (T-4570)". Matching ours: FORMULATEXT, INDIRECT, TAKE, DROP, the error-first order of LARGE, SMALL, MEDIAN and
MAX, and RANK's shape. Mismatches, each with a fix task: OFFSET's 2x1 result (T-4739), RANK, RANK.EQ and RANK.AVG per
element (T-4740), the first element deciding between an error and an off-sheet reference (T-4741; this replaces the
"error comes first" fallback), COUNTIF and SUMIF over an XLOOKUP with a text if_not_found (T-4742).
A grid with references on both the formula's sheet and another cannot be built from XLOOKUP (every miss is the one
if_not_found reference); t4697 covers it with INDIRECT. The probe lays out one probe per three rows so a spill has room.
Eval delta: 74 of the 103 cells pass; the 29 that do not are owned by T-4739 to T-4742 and sit in the probes baseline.
