---
id: T-4168
title: "Pivot layout options: custom subtotals, items with no data, repeat labels, blank rows, collapse"
milestone: M16
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4149]
---

## Goal
The layout honours custom subtotal functions (several per field), show items with no data, repeat item labels, a blank row after each item, per-field form mixes, and collapsed items (`sd="0"`).

## Context
Evidence from cap-layout and the geometry probes; probes added here where missing. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; moved from T-4152, whose adapter renders them
  through the engine): the countA and count custom subtotals over a `#N/A` render `#N/A` where Excel
  counts up to the error (pivots.md "Which function the countA and count custom subtotals compute";
  scope T-4636): probes/t4622-pivot-count-subtotals B1!C6:C7, B4!C6:C7, B5!C6:C7, CA!C6:C6,
  CN!C6:C6.

## Acceptance criteria
- [x] A custom subtotal line's label is Excel's (`<item> Count` for both `t="countA"` and `t="count"`, probes/t4137-pivot-captions K5!A8 and probes/t4622-pivot-count-subtotals B1!A6:A7), never derived from the model's `Aggregate` name (`countA` is Count, `count` CountNums, T-4622). Under a Sum data field an error ends the count within its inner-item cell (pivots.md; scope open, T-4636).
- [x] Probes for each option pass (custom subtotals, blank rows, collapsed items and form mixes here; showAll beyond one field is T-4657's and repeat labels T-4698's, both split off).
- [x] `make check` green; reviewers approve. (`make check-fast` and `make eval-baseline` green locally; review is the PR's.)

## Out of scope
- Styles.

## Outcome
- Custom subtotals, done in full. Every function's label and value comes from the sidecar, as do
  several functions on one field, in tabular and compact form: new probe
  probes/t4168-pivot-layout-options F1 (all eleven, tabular), F2 (compact: the lines follow the
  children), F3 (the subtotal's function wins over a Count data field). The countA/count rule over
  an error under Sum is implemented: `CutCount` in accumulate.rs, fed per inner-item cell in
  result.rs `accumulate`, read by `PivotResult::value_as`. t4622 CA, CN and B1 to B5 now pass
  every cell (C6:C7 were `#N/A`).
- `insertBlankRow` gives a `t="blank"` line after each item of an outer row field, below its
  subtotals, with blank cells (t4168 BR tabular, BR2 compact; lines.rs `blank_line`).
- A collapsed item (`sd="0"`) is one line with its total and no children or subtotal lines (t4168
  SD compact, SD2 tabular; lines.rs `collapsed`).
- Per-field form mixes were already done and evidenced (t4583 R1, t4609 L3, L4). showAll on a
  single row field was already done (t4311 H).
- Split off: T-4698 (repeat item labels, `fillDownLabels` in extLst: model, probe, render) and
  T-4699 (blank rows and collapsed items beyond the probed shapes). showAll beyond one field stays
  with T-4657. UNVERIFIED in pivots.md: the cut-count rule with a column field and under
  aggregates other than Sum (T-4636), and the blank-row and collapse readings listed under
  "Layout lines" (T-4699, T-4698).
- Owner: the new probe case is listed in T-4152 (decision 0245); its `pivots` cells all pass in the engine's unit tests
  (layout/lines/tests.rs, layout/render/tests.rs). The eval adapter that scores them is T-4152's.
