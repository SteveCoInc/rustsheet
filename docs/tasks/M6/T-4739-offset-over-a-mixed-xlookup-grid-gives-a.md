---
id: T-4739
title: "OFFSET over a mixed XLOOKUP grid gives a 2x1 result (probes/t4704 B4:C4, A5, B7:C7, A8)"
milestone: M6
lane: functions-lookup
status: in-progress
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
- [ ] Calc B4:C4, A5, B7:C7 and A8 of probes/t4704 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other readers of a mixed XLOOKUP grid (T-4704).
