---
id: T-4563
title: "XLOOKUP under a lifted lookup_value keeps a range if_not_found as a reference element that INDEX hands back"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P2
size: M
tier: opus
campaign: semantics
depends_on: []
---

## Goal
`XLOOKUP` with an array lookup_value and a range if_not_found gives Excel's elements: a missed element is the range
itself, shown as `#VALUE!` in the grid, and INDEX of that element returns the range's values.

## Context
- T-4543's probe (excel-graph, 2026-11-06; lookup.md "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array"):
  `XLOOKUP({2;9},Data!$A$1:$A$3,Data!$B$1:$B$3,Data!$D$1:$D$2)` (D1:D2 = "x", "y") spills 20, `#VALUE!` (A45:A46) and is
  2x1. Its `INDEX(..,2,1)` (W45) is not `#VALUE!` but the 2x1 range: W45 "x" and W46 "y" spill from it. Ours keeps
  `#VALUE!` for the element (`xlookup.rs` `xlookup_if_not_found`), so W45 is `#VALUE!` and W46 blank.
- probes/xlookup-array-lookup-value-args A30 (`Data!C1:C2`) and A58 (one cell, `Data!C1`) are the grid view: `#VALUE!`.
- Candidate carrier: SPEC-0018's nested cell (`nested_array.rs`): it shows `#VALUE!` and INDEX returns its array
  (`NestedKind::Array`) or its value (`NestedKind::Reference`, one cell). Before using it, probe what else reads the
  element: `ISREF(INDEX(..,2,1))`, `ROWS(INDEX(..,2,1))`, the element under a lift (`XLOOKUP(..)+0`, `&""`), `SUM`,
  `COUNTA` and `ISERROR` of the whole result, a one-cell range, and a range on another sheet. A nested copy also needs a
  charge against the nested budgets (CLAUDE.md directive 5); a reference element would need none.

## Acceptance criteria
- [ ] probes/t4543-xlookup-not-found-address-sheet-lift W45 and W46 pass.
- [ ] The consumers above are probed (excel-graph, predictions committed first) and every cell of that probe passes or
      has an owner.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Owner of the visibly-off cells of probes/t4543-xlookup-not-found-address-sheet-lift (W45, W46): Excel's reference
element read back by INDEX, where ours is `#VALUE!` and a blank.

## Out of scope
- An array (not a range) if_not_found under a lifted lookup_value: it takes its first element, and that matches Excel
  (t4543 A33:A41).
