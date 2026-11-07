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

These date label cells were also off in value until T-4165 applied the filters; they are now off in their text only
(moved from T-4165):
- probes/t4164-pivot-date-times WBt0!A4, WBtN!A4, WEq0!A4, WEq1!A4:A6, WEqN!A4, WNt0!A4:A7, WNt1!A4:A5, WNtN!A4:A7
- probes/t4130-pivot-date-filters DLit!A4:A16, DOld!A4:A6, DOldEq!A4:A7, DNe!A4:A6, DNbt!A4:A6, M1!A4, Q1!A4:A6, DBt!A4:A7, DEq!A4, DNbt!A7:A12, DNe!A7:A15, DNt!A4:A9, DNte!A4:A10, DWd!A4, M1!A5, M10!A4, M11!A4, M12!A4, M2!A4, M3!A4, M4!A4, M5!A4, M6!A4, M7!A4, M8!A4, M9!A4, Q1!A7, Q2!A4:A6, Q3!A4:A6, Q4!A4:A6

## Acceptance criteria
- [ ] A probe filled with the excel-graph oracle covers (1)-(4); its findings are in pivots.md.
- [ ] The engine matches its displayed text, or each off cell has an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional formats and pivot styles (fonts, fills).
