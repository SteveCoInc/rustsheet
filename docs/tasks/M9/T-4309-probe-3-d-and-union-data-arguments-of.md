---
id: T-4309
title: "Probe 3-D and union data arguments of TRIMMEAN, Z.TEST and the PERCENTILE family (T-2883 made a 3-D reference #VALUE!, was #REF!)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A 3-D reference (`Calc:Data!A4:A6`) and a union (`(A1:A2,A4:A6)`) as the data argument of TRIMMEAN, Z.TEST and the PERCENTILE/QUARTILE/PERCENTRANK family are probed against Excel, and the engine matches.

## Context
T-2883 moved TRIMMEAN's and Z.TEST's data to `evaluate_picked_reference_arg(..).into_single_sheet_area`; a 3-D reference as their data went from `#REF!` to `#VALUE!` and a union stays `#VALUE!`, neither backed by evidence (found by the T-2883 fidelity review). Excel accepts 3-D references and unions in LARGE and SMALL (probes/3d-ref-functions C11, probes/multiarea-order-stats H1, H7, H13). Code: statistical/trimmean.rs, z_test.rs, percentile.rs. Rule: docs/excel-semantics/statistics.md, the TRIMMEAN and Z.TEST entries (listed unprobed there).

## Acceptance criteria
- [x] A probe case with oracle values for these forms in each function family is gated; the engine matches, or the deviation is fixed.
- [x] The statistics.md entries' unprobed lines are resolved.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/t4309-3d-union-trimmean-percentile` (22 cells, excel-graph) shows Excel accepts a 3-D reference and a union as the data of all eleven functions. Engine now reads them area by area (`RefArg::into_areas_or_single`, `area_numbers`; Z.TEST via `z_test_stats`); 22/22 cells pass (were 0 for the 22 forms). statistics.md updated.
