---
id: T-4311
title: "Probe showAll on a pivot column field and on a grouped field (T-4276 review follow-ups)"
milestone: M6
lane: engine
status: in-progress
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
- [ ] An excel-graph probe covers a recordless item of an ungrouped column field (`showAll` 0 and 1) and an empty group
      of a numeric range or date-grouped row field with `showAll="1"`; the engine matches it, or owner tasks are
      filed for the off cells.
- [ ] web.md records both results (column fields; grouped fields under `showAll="1"`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unused items of a grouped field (`PivotError::GroupedWithUnusedItems` stays).
