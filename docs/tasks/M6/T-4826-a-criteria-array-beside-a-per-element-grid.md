---
id: T-4826
title: "A criteria array beside a per-element grid broadcasts: COUNTIF(XLOOKUP({2;9},..,\"x\"),{\">0\",\">5\"}) is a 2x2 result (probes/t4742 A40, C40, A41)"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4742]
---

## Goal
A per-element criteria call (COUNTIF, SUMIF and the like) whose range argument is a grid and whose criteria is an array spills the broadcast of the grid's shape and the criteria's: `COUNTIF(XLOOKUP({2;9},Data!A1:A3,Data!B1:B3,"x"),{">0",">5"})` is 2x2 (`#SPILL!` where blocked), not a 2x1 cut to each call's top-left value.

## Context
- Evidence: probes/t4742-countif-array-range-arg (excel-graph, 2026-11-07, T-4742) probe 14: A40 `#SPILL!`, B40 2, C40 2 (A41 is blocked, so empty). Ours gives a 2x1 (`#VALUE!` in A40, COLUMNS 1, A41 spilled).
- Code: `Model::call_per_reference_element` (lookup_and_reference/ref_lift.rs) cuts each element's array result to its top-left value; `criteria_and_grid_static_result` (static_analysis.rs) takes the grids' shape alone. The same holds for OFFSET and INDIRECT grids (unprobed, T-2292 lists it).
- The element values of the 2x2 are not visible in this probe (blocked spill): probe them unblocked (e.g. INDEX of each, or room to spill) before implementing.

## Acceptance criteria
- [ ] probes/t4742 Calc A40, C40, A41 pass, with the element values probed and matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Grids beside several criteria arrays of different shapes.
