---
id: T-4800
title: "Probe OFFSET over an all-hit lifted XLOOKUP with a range if_not_found (T-4739 follow-up)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's value and shape (ROWS, COLUMNS) are known for OFFSET over a lifted XLOOKUP with a range if_not_found where
every lookup hits, e.g. `OFFSET(XLOOKUP({2;3},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$G$1:$G$2),0,0)`. Any mismatch gets a fix task.

## Context
- T-4739 (#3866) made OFFSET lift over a mixed XLOOKUP grid (a hit and a miss): a 2x1 of `#VALUE!` (probes/t4704 A4:C8).
  With no miss, `xlookup_reference` gives the hits as a value array, and OFFSET of an array is a scalar `#VALUE!` (ours,
  unprobed; unchanged by T-4739). Excel may still spill a 2x1 here: in probes/t4704, COUNTIF and SUMIF over an
  XLOOKUP with a text if_not_found spill a 2x1 of `#VALUE!` with all hits too (cells owned by T-4742; reviewer-excel-fidelity
  on #3866).
- Probe with the `excel-probe` skill: value, ROWS and COLUMNS for all hits with `,0,0` and `,1,0`, a one-element
  `{2}` lookup, and a scalar lookup as a control; the t4704 Data sheet can be reused.
- Code: functions/lookup_and_reference/offset.rs `offset_source`; SPEC-0014 amendment "Per-element callers".

## Acceptance criteria
- [ ] A probe case filled on excel-graph covers the shapes above, with predictions committed before the fill.
- [ ] The finding is in docs/excel-semantics/lookup.md and SPEC-0014; each mismatch has a fix task naming its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing a mismatch (its own task).
