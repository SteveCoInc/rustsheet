---
id: T-4717
title: "Probe pivot cell formats across refreshes, showDataAs and field number formats"
milestone: M16
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4169]
---

## Goal
The engine's pivot cell formats match Excel's where T-4169 left them UNVERIFIED (docs/excel-semantics/pivots.md "Value cells show their data field's number format").

## Context
T-4169 gives a value cell its data field's `numFmtId` when the field has one, and leaves every other cell's format alone (`Model::set_pivot_number_format` in crates/rustsheet-engine/src/pivot/grid.rs). probes/t4169-pivot-number-formats passes, but it does not show what Excel does: (1) when a refresh turns a formatted value cell into a label or empties it; (2) when a data field has no `numFmtId` but the cell already had a format, under `preserveFormatting` on and off; (3) for a showDataAs field (percentOfTotal and others) with no `numFmtId`; (4) when a row, column or page field's `numFmtId`, or a source cell's format, applies to item labels (a date item's label is its serial today). The excel-graph fill now records each pivot's displayed text (`text` in the sidecar's `pivots` entries), so text_expect scores any new probe. Related owners: T-4730 (custom-format ids in the spec adapter) and T-4731 (built-in ids and custom-id numbering).

The date-label cells this task owned (decision 0245, moved here from T-4164 and T-4165) were the text of date item
labels: probes/t4164-pivot-date-times (WOe0, WOe1, WOeN, WBt0, WBtN, WEq0, WEq1, WEqN, WNt0, WNt1, WNtN, column A)
and probes/t4130-pivot-date-filters (DLit, DOld, DOldEq, DNe, DNbt, DBt, DEq, DNt, DNte, DWd, M1 to M12, Q1 to Q4,
column A). T-4807 (merged as bcfb7256b, date item labels in `m/d/yyyy`, with ` h:mm` for a time part) is the rule's
real owner; every one of those cells passes on main since, so the list is dropped here (both cases pass in full).

## Acceptance criteria
- [x] A probe filled with the excel-graph oracle covers (1)-(4); its findings are in pivots.md.
- [x] The engine matches its displayed text, or each off cell has an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional formats and pivot styles (fonts, fills).

## Outcome
probes/t4717-pivot-cell-formats (excel-graph, 2026-11-07) covers readings (1)-(4) on thirteen refreshOnLoad pivots,
some over blank cells formatted before the render. Findings (pivots.md "A refresh gives pivot cells their fields'
number formats", VERIFIED): a refresh gives every number cell its field's format, whatever the cell had. (1) Cells
formatted `$#,##0.00` or `0.0%` that become number item labels read General (`7`, `12`), and a value cell its data
field's format. (2) A data field with no `numFmtId`, or `numFmtId="0"`, shows General over a formatted cell, with
`preserveFormatting` on and off alike. (3) showDataAs percentOfTotal and index with no `numFmtId` show General
(`0.755594872`), and a source column's format does not reach its values. (4) A row, column or page field's
`numFmtId` formats its number and date item labels (`7.00`, `12.00`, `15-Jan-24`, `100.00%`) and the item in a
subtotal label (`7.00 Total`), and it beats the source's format. Without one, a number item is General (a `0.00`
source reads `1`), and a date item shows the fixed short date T-4807 implemented, even over a `yyyy-mm-dd` source
(L5 `1/15/2024`).

Engine, test-first (`pivot_cells_take_their_fields_formats_over_the_cells_own` in pivot/grid/tests.rs, which checks
every text of twelve sheets against the sidecar), built on T-4807's `CellFormat`:
- `item_format` in render.rs gives a number or date item its pivot field's `numFmtId`, else General for a number
  (date items unchanged: date group items m/d/yyyy, others T-4807's).
- A value cell with no data field `numFmtId` gets General (0), through the existing `RenderedCell::number_format`
  path and `Model::set_pivot_number_format`.
- The page area formats a selected item as a label.
- The new `item_text_in` formats a subtotal label's item in its field's `numFmtId`. `render` takes the workbook's
  `num_fmts` for it, and the PivotChart categories now share it in place of their own copy.

Eval: the new case passes in full (177 cells, 159 texts); no other case changed and there are 0 regressions. The
roundtrip baselines are unchanged. roundtrip-poi LIBRE_OFFICE-116306-0 timed out twice under machine load (about
28 s alone, both with and without this change, against a 60 s limit) and passed when re-run with less contention.
The date-label cells T-4717 owned all pass since T-4807 merged, so the list is dropped.

Catalog: PV-0362 and PV-0895 are re-pointed to T-5058 with this probe as evidence, and PV-1115 cites L1.

Follow-up: T-5058 (filed) owns what text cannot show or this probe left out:
- emptied cells;
- captions under a format with a text section;
- the pivot's `<formats>`;
- a non-zero `cacheField@numFmtId`;
- a field `numFmtId` on grouped fields;
- a pivotField `numFmtId="0"` on an ungrouped date field (the engine shows General);
- custom field ids;
- caption filters over formatted items.

Date items' seconds, subtotal labels and page items stay T-5049's. The label skip in pivot/filter/tests.rs
(`check_calculated`) was a leftover of this task's ownership; it is removed, and those date label cells are checked.
