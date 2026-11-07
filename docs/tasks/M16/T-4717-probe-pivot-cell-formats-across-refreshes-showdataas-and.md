---
id: T-4717
title: "Probe pivot cell formats across refreshes, showDataAs and field number formats"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4169]
---

## Goal
The engine's pivot cell formats match Excel's where T-4169 left them UNVERIFIED (docs/excel-semantics/pivots.md "Value cells show their data field's number format").

## Context
T-4169 gives a value cell its data field's `numFmtId` when the field has one, and leaves every other cell's format alone (`Model::set_pivot_number_format` in crates/rustsheet-engine/src/pivot/grid.rs). probes/t4169-pivot-number-formats passes, but it does not show what Excel does: (1) when a refresh turns a formatted value cell into a label or empties it; (2) when a data field has no `numFmtId` but the cell already had a format, under `preserveFormatting` on and off; (3) for a showDataAs field (percentOfTotal and others) with no `numFmtId`; (4) when a row, column or page field's `numFmtId`, or a source cell's format, applies to item labels (a date item's label is its serial today). The excel-graph fill now records each pivot's displayed text (`text` in the sidecar's `pivots` entries), so text_expect scores any new probe. Related owners: T-4730 (custom-format ids in the spec adapter) and T-4731 (built-in ids and custom-id numbering).

Owner of the text of date item labels (decision 0245, T-4164): Excel shows a date item's row label in its source
cell's number format (`m/d/yyyy h:mm`, and `4/17/2024` at midnight; pivots.md "useWholeDay"), the engine the serial
(`45398.5`). These cells are off in their text only:
- probes/t4164-pivot-date-times WOe0!A4:A5, WOe1!A4:A7, WOeN!A4:A5

The other date-times label cells are also off in value (the filters, T-4165's), and their text follows once both are
fixed.

## Acceptance criteria
- [ ] A probe filled with the excel-graph oracle covers (1)-(4); its findings are in pivots.md.
- [ ] The engine matches its displayed text, or each off cell has an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional formats and pivot styles (fonts, fills).
