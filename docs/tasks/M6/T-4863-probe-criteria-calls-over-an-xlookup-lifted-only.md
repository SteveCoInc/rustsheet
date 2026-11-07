---
id: T-4863
title: "Probe criteria calls over an XLOOKUP lifted only by match_mode or search_mode, and settle the COUNTIF-over-a-computed-array wording"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4742]
---

## Goal
Excel evidence decides what `COUNTIF(XLOOKUP(2,A1:A3,B1:B3,"x",{0;1}),">0")` and the search_mode form give (a lifted XLOOKUP whose lookup_value is a scalar), and lookup.md's sentence on COUNTIF over a computed array says only what the evidence shows.

## Context
- From the T-4742 excel-fidelity review (PR #3890). Ours builds the per-element grid whenever any XLOOKUP criterion lifts (`lifts` in `xlookup_read`, xlookup.rs), but `criteria_and_grid_static_result` (static_analysis.rs) takes the spill shape from `lookup_value` alone, so a lift by match_mode or search_mode only is a scalar statically and a 2x1 grid at run time. The same gap held before T-4742 for a range if_not_found. Probe both forms (value, ROWS, COLUMNS) and the mixed form with a lifted lookup_value of another shape, then match the static shape to the evidence or file the fix.
- docs/excel-semantics/lookup.md (the T-4742 paragraph) says "COUNTIF takes no computed array as a range" from a workbook that failed to open on Excel for the web (createSession 501 unsupportedWorkbook; probes/t4742-countif-array-range-arg description). No cell backs it. Reword it as the observation it is, or probe it on the desktop backend if one is up (M14 Phase 4 otherwise).

## Acceptance criteria
- [ ] A probe case with excel-graph oracle values for the match_mode and search_mode lifts under COUNTIF and SUMIF is committed; ours matches it or an owner task is filed.
- [ ] The lookup.md sentence is reworded to the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The array-criterion broadcast (T-4826).
