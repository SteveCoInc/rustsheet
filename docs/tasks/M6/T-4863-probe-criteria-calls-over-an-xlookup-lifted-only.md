---
id: T-4863
title: "Probe criteria calls over an XLOOKUP lifted only by match_mode or search_mode, and settle the COUNTIF-over-a-computed-array wording"
milestone: M6
lane: functions-lookup
status: done
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

## Outcome
Probe `probes/t4863-xlookup-mode-lift-criteria` (excel-graph, 2026-11-07; 129 cells): a match_mode or search_mode array over a scalar lookup_value lifts XLOOKUP to a grid (ROWS 2, COLUMNS 1 for `{0;1}`; 1x2 for `{0,1}`), and COUNTIF and SUMIF (range or sum_range) over it give an array of `#VALUE!`, exactly as for a lookup_value lift; a 1x1 `{0}` mode is a scalar `#VALUE!`; a range if_not_found does not lift (COUNTIF 2, SUMIF 110). Ours matched all but the spill shape of the mode-only lifts (12 cells, a scalar statically instead of a grid): filed as T-5037. lookup.md's COUNTIF sentence is reworded to the evidence (the web-failed workbook was not a probe of COUNTIF's range argument). Eval: new probe case, 117 of 129 cells pass; no baseline change.
