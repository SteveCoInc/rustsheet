---
id: T-4742
title: "COUNTIF and SUMIF over an XLOOKUP with a text if_not_found spill a 2x1 result (probes/t4704 B64:C64, A65, B67:C67, A68, B70:C70, A71, B73:C73, A74)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4704]
---

## Goal
`COUNTIF(XLOOKUP({2;9},A1:A3,B1:B3,"x"),">0")` and the SUMIF forms, with a miss or with every lookup a hit, spill a 2x1 result of `#VALUE!` elements (the range argument is a value array, not a range), not a scalar `#VALUE!`.

## Context
- Evidence: probes/t4704-mixed-xlookup-grid-readers (excel-graph, 2026-11-07, T-4704): probes 22-25 (Calc A64:C65 COUNTIF with a miss, A67:C68 COUNTIF all hits, A70:C71 SUMIF with a miss, A73:C74 SUMIF all hits): ROWS 2, COLUMNS 1, both elements `#VALUE!`. Ours gives a scalar `#VALUE!` (B64:C64, B67:C67, B70:C70, B73:C73; A65, A68, A71, A74 empty).
- This is the per-element-caller bullet of T-4704; t4569 A26 (`COUNTIF` over an XLOOKUP with a reference if_not_found) already spills.

## Acceptance criteria
- [x] Calc B64:C64, A65, B67:C67, A68, B70:C70, A71, B73:C73 and A74 of probes/t4704 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other readers of a mixed XLOOKUP grid (T-4704).

## Outcome
A lifted XLOOKUP in a per-element caller's reference argument (COUNTIF, SUMIF, COUNTIFS, AVERAGEIF and the rest of `Function::per_reference_element`) is now always a grid of its elements, whatever its `if_not_found`: `Model::xlookup_reference_elements` (xlookup.rs, the `XlookupGrid::Elements` read) keeps the all-hits grid and wraps a value-`if_not_found` result in `RefGrid::of_values`; a value element is `#VALUE!` in the call, an error element its error. `lifts_into_reference_elements` and `criteria_and_grid_static_result` take any XLOOKUP, not only one with a reference `if_not_found`. New probe probes/t4742-countif-array-range-arg (excel-graph, 2026-11-07) covers the three-argument form, all misses, a 1x1 and a 1x2 lookup, a scalar lookup, SUMIF's sum_range, COUNTIFS, AVERAGEIF and an array criterion; COUNTIF over SEQUENCE, arithmetic and SORT arrays made the workbook fail to open on Excel for the web, so those probes were dropped (recorded in lookup.md). Unit test added; lookup.md and SPEC-0014 updated.

Eval delta: probes known failures 7603 -> 7594 (fixed 12: probes/t4704 Calc B64:C64, A65, B67:C67, A68, B70:C70, A71, B73:C73, A74); the new case passes 38 of 41 cells, regressions 0. Follow-up: T-4826 owns probes/t4742 A40, C40, A41 (an array criterion beside the grid broadcasts to 2x2).
