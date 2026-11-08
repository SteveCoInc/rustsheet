---
id: T-5037
title: "COUNTIF and SUMIF over an XLOOKUP lifted only by match_mode or search_mode (or a 1x2 mode array) are an array of #VALUE!, not a scalar (probes/t4863)"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A criteria call (COUNTIF, SUMIF) over an XLOOKUP whose only per-element lift is match_mode or search_mode (scalar lookup_value) spills the XLOOKUP's grid shape of `#VALUE!` values, like the lookup_value-lifted form: `COUNTIF(XLOOKUP(2,A1:A3,B1:B3,"x",{0;1}),">0")` is a 2x1 of `#VALUE!`, and a 1x2 mode array gives a 1x2.

## Context
- Evidence: probes/t4863-xlookup-mode-lift-criteria (excel-graph, 2026-11-07, T-4863). Off cells: Calc A11, A15, A19, A27, A31, A35, A43, A47, A51 (2x1 `#VALUE!`, second row empty in ours) and B90, B94, B98 (1x2, second column empty in ours). Value, ROWS and COLUMNS of the first cell already match; the mixed lookup_value x mode forms, the 1x1 `{0}` mode and the range if_not_found (a reference, no lift) all pass.
- Code: `criteria_and_grid_static_result` (static_analysis.rs) takes the spill shape from `lookup_value` alone; `lifts` in `xlookup_read` (xlookup.rs) builds the grid whenever any criterion lifts. Make the static shape follow match_mode and search_mode too.

## Acceptance criteria
- [x] probes/t4863-xlookup-mode-lift-criteria Calc A11, A15, A19, A27, A31, A35, A43, A47, A51, B90, B94, B98 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done in the M6 probe-cell PR: `criteria_and_grid_static_result` takes an XLOOKUP grid's shape from the broadcast of lookup_value, match_mode and search_mode. Every owned cell passes; probes/t4863-xlookup-mode-lift-criteria passes whole.
