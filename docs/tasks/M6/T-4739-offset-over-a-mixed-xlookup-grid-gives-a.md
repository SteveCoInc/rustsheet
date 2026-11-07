---
id: T-4739
title: "OFFSET over a mixed XLOOKUP grid gives a 2x1 result (probes/t4704 B4:C4, A5, B7:C7, A8)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4704]
---

## Goal
`OFFSET(XLOOKUP({2;9},A1:A3,B1:B3,Data!G1:G2),0,0)` and `(...,1,0)` spill a 2x1 result of `#VALUE!` elements, as Excel does (ROWS 2, COLUMNS 1; both elements `#VALUE!`), not a scalar `#VALUE!`.

## Context
- Evidence: probes/t4704-mixed-xlookup-grid-readers (excel-graph, 2026-11-07, T-4704): probes 2 and 3 (Calc A4:C5, A7:C8). Ours gives a scalar `#VALUE!` (B4:C4, B7:C7 expect 2 and 1; A5, A8 expect `#VALUE!` where ours is empty).
- T-4704's other readers pass: FORMULATEXT (`#VALUE!`, then `#N/A`) and INDIRECT (`#REF!`, `#VALUE!`) already match, so only OFFSET's lift over a mixed grid differs.
- Code: `functions/lookup_and_reference/offset.rs`; SPEC-0014 "As implemented (T-4570)" per-element callers.

## Acceptance criteria
- [x] Calc B4:C4, A5, B7:C7 and A8 of probes/t4704 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other readers of a mixed XLOOKUP grid (T-4704).

## Outcome
- `offset_source` (offset.rs) reads an XLOOKUP argument in its reference form
  (`xlookup_reference`), as the per-element callers and the aggregates already do, so a lifted
  XLOOKUP with a range if_not_found reaches `offset_over_grid` as its mixed grid: the hit (a value)
  gives `#VALUE!` and the miss is OFFSET of the range, a two-cell range read as `#VALUE!`. Any
  other XLOOKUP reads as before (the reference it picks, or its value).
- Unit test `offset_lifts_over_a_mixed_grid`; SPEC-0014 and lookup.md notes say it now matches.
- Eval delta: probes/t4704 Calc B4:C4, A5, B7:C7, A8 now pass (6 known failures removed from
  eval/baseline/probes.txt); 0 regressions in every suite.
