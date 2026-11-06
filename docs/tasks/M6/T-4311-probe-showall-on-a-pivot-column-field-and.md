---
id: T-4311
title: "Probe showAll on a pivot column field and on a grouped field (T-4276 review follow-ups)"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GETPIVOTDATA on a recordless item follows Excel's `showAll` rule on a column field and on a grouped field, with the
behavior backed by probe evidence instead of analogy.

## Context
These are the MINOR findings from the excel-fidelity review of PR #3314 (T-4276), which all reviewers approved.
probes/t4248-getpivotdata-recordless-item covers only an ungrouped **row** field. Two engine behaviors still
have no probe behind them:
- `shown_group` (crates/rustsheet-engine/src/functions/lookup_and_reference/pivot_index.rs) applies a field's `showAll`
  to ungrouped **column** fields too, by analogy with the row axis.
- Grouped fields (numeric range and date groups) ignore `showAll`: an empty group is `#REF!` whatever the setting.
  Only `showAll="0"` is probed (probes/t4228-pivot-range-groups, t4236-pivot-date-groups-no-blank).
Rule today: docs/excel-semantics/web.md "GETPIVOTDATA on a recordless item of an ungrouped pivot field". The spec
builder already writes `unused_items` and `show_all` (tools/src/rustsheet_tools/pivot.py). A column field, and
`show_all` on a grouped pivot, may need builder support.

## Acceptance criteria
- [x] An excel-graph probe covers a recordless item of an ungrouped column field (`showAll` 0 and 1) and an empty group
      of a numeric range or date-grouped row field with `showAll="1"`; the engine matches it, or owner tasks are
      filed for the off cells.
- [x] web.md records both results (column fields; grouped fields under `showAll="1"`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Unused items of a grouped field (`PivotError::GroupedWithUnusedItems` stays).

## Outcome
- Probe probes/t4311-pivot-showall-column-and-groups (excel-graph, 2026-11-05):
  - **Column field:** an unused shared item follows `showAll` exactly as on the row field (T-4248). It is `#REF!` at
    `showAll="0"` and 0 at `showAll="1"`, also with a row item and in lower case (Calc A1:B7). The engine's analogy holds.
  - **Grouped field with `showAll="1"`:** Excel lays out every group, `<0` and `>0.4` included, each empty one with a
    blank value (Calc I1:J8, read back from Excel's own layout), and GETPIVOTDATA on an empty group is 0 (E1, E3,
    E6). With `showAll="0"` the result is as before: the empty groups are hidden and `#REF!` (D1:D7, G1:H5).
- Engine: `shown_group` (pivot_index.rs) now applies `showAll` to grouped fields as well. `row_layout`
  (pivot/layout.rs) takes the row field's `showAll`, shows the empty groups with a new `LayoutCell::Blank`, and
  `max_layout_rows` covers that layout. The `rustsheet` facade (`PivotTableInput`) and the eval adapter take a column
  field's `column_unused_items` and `column_show_all`, which the spec builder (tools pivot.py) now writes.
- web.md: the T-4248 entry records both results, and the numeric-range and date-group entries say an empty group
  follows `showAll`. A date group under `showAll="1"` is taken to follow the numeric range's rule (unprobed).
- Tests: two facade unit tests (a grouped pivot under `showAll`, and a column field's unused items) and the builder's
  pivot tests.
- Eval delta: probes/t4311-pivot-showall-column-and-groups is new, 68 of 68 cells matching Excel (eval/baseline/probes.txt
  records it); no regressions, and the known failures are unchanged.
