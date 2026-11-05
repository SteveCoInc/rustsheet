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
- Two excel-graph probes (2026-11-05): probes/t4311-pivot-showall-column (20 cells) and t4311-pivot-showall-grouped
  (129 cells). web.md "GETPIVOTDATA on a recordless item of a pivot field" records them.
  - Column field: the row-axis rule holds. An unused column item is `#REF!` under the column field's `showAll="0"`
    (alone, with a row item in either order, in lower case) and 0 under `showAll="1"`. The engine already matched.
  - Grouped field with `showAll="1"`: Excel lays out every group, empty ones with a blank value (`<start` and `>end`
    too: 14 month items over 3 dates), and GETPIVOTDATA on an empty group is 0 by label or number. The engine
    ignored `showAll` here (`#REF!`, groups hidden).
- Engine: `shown_group` applies `showAll` to grouped fields as to ungrouped ones; `row_layout` takes the row field's
  `showAll` and lays out a recordless item with `LayoutCell::Empty` (`Txn::add_pivot_table` leaves its value cell
  empty); the eval input-set extent covers every group of a `showAll` grouped pivot (spill_extent.rs `group_rows`).
- Builder and adapter: `column_unused_items` and `column_show_all` (tools pivot.py, case.rs, spec_pivots.rs,
  `PivotTableInput`), with builder tests; a builder test no longer mutates the shared `_PIVOT` fixture.
- Eval delta: probes known failures 5713 -> 5713, 2 new cases (149 cells) all pass, 0 regressions in every suite.
