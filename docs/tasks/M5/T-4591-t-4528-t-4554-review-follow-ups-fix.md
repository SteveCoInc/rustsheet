---
id: T-4591
title: "T-4528/T-4554 review follow-ups: fix the array_constant_context path in information.md, probe a nested legacy ATP call over {5}, add WorkbookView::new"
milestone: M5
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The MINOR review findings of PR #3604 (T-4528) and PR #3607 (T-4554) are closed: a wrong path is corrected, the
nested legacy ATP form has evidence, and a repeated struct literal gets one constructor.

## Context
- docs/excel-semantics/information.md "ISEVEN/ISODD over a range" and the T-4528 Outcome say `array_constant_context`
  is in static_analysis/legacy.rs. It is in crates/rustsheet-engine/src/expressions/parser/static_analysis/lifted.rs.
- `array_constant_context` gives every `LiftArrays` argument the scalar signature, so a one-element constant in a
  nested legacy ATP call also gets `@`: `=SUM(EDATE({45000},1))` shows `=SUM(EDATE(@{45000},1))`. That follows the
  scalar-argument rule (`=ABS(@{5})`), but probes/iseven-isodd-range-derived-arrays covers only top-level calls (Leg!C3,
  F3, G3:N3). docs/excel-semantics/arrays.md says "at a value position", which reads narrower than what the code does.
- `WorkbookView { sheet, window_width: DEFAULT_WINDOW_WIDTH, window_height: DEFAULT_WINDOW_HEIGHT }` is built in
  crates/rustsheet-engine/src/defined_names.rs (`set_active_sheet`, T-4554) and new_empty.rs.

## Acceptance criteria
- [ ] information.md and the T-4528 Outcome name lifted.rs.
- [ ] A legacy probe case (excel-probe skill, `legacy_formulas`, FORMULATEXT) covers `=SUM(EDATE({45000},1))`,
      `=SUM(EDATE({45000;45031},1))`, `=SUMPRODUCT(ISEVEN({5}))` and `=ABS(QUOTIENT({7},2))`. The engine matches, or
      an owner task is filed, and arrays.md's wording matches the evidence.
- [ ] One `WorkbookView` constructor with the default window size is used where those two literals are, once T-4554
      (PR #3607) has merged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other lifting-table kinds.

## Outcome
- information.md and the T-4528 Outcome now name static_analysis/lifted.rs.
- New probe probes/nested-legacy-atp-array-constant (excel-graph): a nested legacy ATP call over an array
  constant gets no `@` under SUM/SUMPRODUCT (`=SUM(EDATE({45000},1))` = 45031, the two-element form 90092,
  `=SUMPRODUCT(ISEVEN({5}))` = 0) and gets it under ABS (`=ABS(QUOTIENT(@{7},2))` = 3). The engine already
  matches all four; arrays.md records the rule.
- `WorkbookView::new(sheet)` replaces the literals in new_empty.rs and defined_names.rs (T-4554 merged as #3607).
  The xlsx importer keeps its own 800x600 literal and was not touched.
- Eval delta: +8 passing cells in the new probe; no baseline change.
