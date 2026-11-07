---
id: T-4563
title: "XLOOKUP under a lifted lookup_value keeps a range if_not_found as a reference element that INDEX hands back"
milestone: M6
lane: functions-lookup
status: done
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
- [ ] probes/t4543-xlookup-not-found-address-sheet-lift W45 and W46 pass. (Moved to T-4570: see the Outcome.)
- [x] The consumers above are probed (excel-graph, predictions committed first) and every cell of that probe passes or
      has an owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- An array (not a range) if_not_found under a lifted lookup_value: it takes its first element, and that matches Excel
  (t4543 A33:A41).

## Outcome
- Probe probes/t4563-xlookup-not-found-ref-element: 29 questions, 245 cells, excel-graph, canary-verified,
  2026-11-06, tagged M6. The engine's values were committed first (docs/research/t4563/predictions.md). 139 cells
  pass.
- Finding (lookup.md, added to "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array"): the missed element
  is a real reference, not a nested value.
  - ISREF, TYPE 64 or 1, AREAS, CELL("address") and SUM all treat INDEX of it as a reference.
  - SUM over the whole result walks it.
  - A lifted operator reads it as `#VALUE!`, even for one cell.
  - So SPEC-0018's nested cell, the candidate carrier in Context, would fail ISREF, TYPE, AREAS, CELL and the
    aggregates. SPEC-0014's `RefGrid` holds no values. Matching Excel needs an array that mixes values and
    references: an architectural change, which needs a spec first (CLAUDE.md directive 4).
- Split:
  - T-4569 writes the spec (a SPEC-0014 amendment).
  - T-4570, which depends on T-4569, implements it. It owns the case's 106 visibly-off cells and t4543 W45:W46,
    which move there from this task.
  - The first acceptance criterion moves with them.
- Eval delta: +1 case, 245 cells: 139 pass, 106 new-coverage known failures (owner T-4570). Fixed 0, regressions 0.
