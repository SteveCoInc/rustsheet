---
id: T-4966
title: "Hide the Values row of a pivot with x14 hideValuesRow and several data fields on columns"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot with `x14:pivotTableDefinition hideValuesRow="1"`, several data fields and the Values field as its only column field renders no `Values` header row, as Excel saves it.

## Context
Found by T-4906, the pivot-corpus render score (eval-harness.md "Pivot freshness"). Excel's saved layout of such a pivot has `firstHeaderRow="0"`: the data fields' captions head their columns on the location's first row, beside the row field's header, and the body starts on the next row. The engine writes a `Values` row first and shifts every cell down one row, so the whole area differs. With one data field the flag changes nothing (pivots.md "Layout and totals flags as Excel for the web renders them", X14).

The rule and its evidence: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)". Code: crates/rustsheet-engine/src/pivot/layout.rs (geometry), crates/rustsheet-xlsx/src/import/pivots/table.rs (reading the x14 flag, which the definition keeps in `PivotTable::opaque`). The engine pivot code is another stream's: this task is that stream's to implement. The rule is UNVERIFIED (Excel-saved corpus cells, not an oracle probe): probe it first with the excel-probe skill (pivots.md's evidence rule). Related: T-4650 (`firstHeaderRow` other than 1).

The non-en-US corpus pivots show it too, unscored (decision 0030): pivot-corpus-lo `xlsx/pivot-table/shared-numgroup.xlsx`, `shared-dategroup.xlsx`, `shared-group-field.xlsx`, `shared-nested-dategroup.xlsx` and `first_header_row_zero.xlsx` (which collapses to `#SPILL!` for want of the extra row).

Split: a corpus pivot can show several causes; each T-4906 owner task (T-4966 to T-4970) and the gen-pivot owners it names own their own cause there.

Owner of these visibly-off cells (decision 0245), 378 render cells of 13 fresh pivots (a span is the bounding box of the pivot's differing cells), two of them in the calipers and poi suites, whose runs score their fresh pivots too:
- calipers/officejs/pivot_two_data/golden.xlsx Pivot!A1:C5
- poi/DataTableCities.xlsx PivotTable!B5:E19
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/ClearPivotTableRenderedRangeWhenLoading/inputfile.xlsx Sheet1!A1:D6
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx DesktopUsage!A1:C5
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx DesktopFunVsProfit!A3:C8
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx ServerUsage!A3:C7
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx ServerDistros!A3:C45
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_3_DataFieldInColumn_ColumnLabelColor.xlsx Sheet1!G4:I10
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_4_DataFieldInColumn_DataColor.xlsx Sheet1!G4:I10
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_5_DataFieldInColumnAndTwoRowFields_DataColor.xlsx Sheet1!G4:J14
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_9_MultipleSelections.xlsx Sheet1!G4:I10
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_14_TwoRowsDataFieldInColumn_LabelColor.xlsx Sheet1!G4:J14
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_15_TwoDataFieldColumns_WholeDataColumnSelected.xlsx Sheet1!F1:H8

## Acceptance criteria
- [ ] A probe case (excel-probe) shows what Excel for the web renders for `hideValuesRow="1"` with two and three data fields on columns, in compact, outline and tabular form, and pivots.md's entry cites it.
- [ ] The engine lays such a pivot out as the probe shows, and the location, `firstHeaderRow` and `rowItems`/`colItems` it emits follow.
- [ ] The listed pivot-corpus pivots render exactly (`rustsheet-eval update-baseline --suite pivot-corpus-lo --suite pivot-corpus-closedxml` drops their failures).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other causes of the same pivots (their own tasks).
