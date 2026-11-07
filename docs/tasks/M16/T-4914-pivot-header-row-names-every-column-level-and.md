---
id: T-4914
title: "Pivot header row names every column level and the Values field in outline and tabular form"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
In outline and tabular pivots the engine writes Excel's header labels: the Values field's label (`Values`, or `dataCaption`) where the Values field has its own column or row header, and each column field's name in the header row, one per column level from the first data column.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

In every non-compact gen-pivot pivot with the Values field on an axis or two or more column fields, Excel's header row (the location's first row) names every column level in turn: e.g. gen-pivot/01865529b5a09c0b P02 has `Values` at L7 (Values first on rows, tabular) where the engine shows `Channel`, and `When` at Q5 (the second column field) where the engine leaves the cell blank; gen-pivot/2251b0f5e5a256e1 P12 has `Values`, `Rep`, `Channel` in M5:O5 where the engine shows only `Rep`. The engine writes the first column field's name only, and the row field's name in the Values field's row-header cell. Related probe: T-4650 (Values field positions) and T-4660 (the tabular column header).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/render.rs (header cells). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4915, T-4916, T-4917, T-4918, T-4919, T-4920, T-4921, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 90 values and texts on 56 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/01865529b5a09c0b P01!N5
- gen-pivot/01865529b5a09c0b P02!L5:Q7
- gen-pivot/01865529b5a09c0b P03!N5
- gen-pivot/01865529b5a09c0b P04!L6
- gen-pivot/01865529b5a09c0b P05!L5:N7
- gen-pivot/01865529b5a09c0b P08!N5
- gen-pivot/01865529b5a09c0b P09!L5
- gen-pivot/01865529b5a09c0b P10!L5:O7
- gen-pivot/01865529b5a09c0b P11!N5
- gen-pivot/01865529b5a09c0b P12!L5:O7
- gen-pivot/2251b0f5e5a256e1 P02!L5:P7
- gen-pivot/2251b0f5e5a256e1 P03!O5
- gen-pivot/2251b0f5e5a256e1 P04!N5:P7
- gen-pivot/2251b0f5e5a256e1 P05!L5:P7
- gen-pivot/2251b0f5e5a256e1 P06!O6
- gen-pivot/2251b0f5e5a256e1 P07!M5:O5
- gen-pivot/2251b0f5e5a256e1 P09!P5
- gen-pivot/2251b0f5e5a256e1 P10!P5
- gen-pivot/2251b0f5e5a256e1 P11!M5:O5
- gen-pivot/2251b0f5e5a256e1 P12!M5:O5
- gen-pivot/36996978427905b2 P01!L5
- gen-pivot/36996978427905b2 P02!P5:Q5
- gen-pivot/36996978427905b2 P04!N6
- gen-pivot/36996978427905b2 P06!L5:P7
- gen-pivot/462f10551d480f9d P04!L5:Q7
- gen-pivot/462f10551d480f9d P06!L6
- gen-pivot/462f10551d480f9d P09!O5:P5
- gen-pivot/462f10551d480f9d P10!P5
- gen-pivot/462f10551d480f9d P12!O6
- gen-pivot/640780d7b6f4ad53 P05!O5
- gen-pivot/640780d7b6f4ad53 P06!L5
- gen-pivot/640780d7b6f4ad53 P08!O6
- gen-pivot/640780d7b6f4ad53 P09!L6
- gen-pivot/640780d7b6f4ad53 P11!M5:O5
- gen-pivot/a6d74990f3ae755e P01!P5
- gen-pivot/a6d74990f3ae755e P04!N5:P5
- gen-pivot/a6d74990f3ae755e P07!O5:P5
- gen-pivot/a6d74990f3ae755e P10!M5:O5
- gen-pivot/a6d74990f3ae755e P11!L5
- gen-pivot/ca5c8fa99c13d747 P01!O5:Q5
- gen-pivot/ca5c8fa99c13d747 P06!N5:O5
- gen-pivot/ca5c8fa99c13d747 P07!M5:O7
- gen-pivot/ca5c8fa99c13d747 P09!L6
- gen-pivot/ec473260bb2065b3 P02!N5:O5
- gen-pivot/ec473260bb2065b3 P03!N5:O5
- gen-pivot/ec473260bb2065b3 P06!O5
- gen-pivot/ec473260bb2065b3 P07!O5
- gen-pivot/ec473260bb2065b3 P09!N5:P5
- gen-pivot/ec473260bb2065b3 P11!L5:Q7
- gen-pivot/ec473260bb2065b3 P12!M5:O7
- gen-pivot/f9e7edbb5b74011a P02!O5
- gen-pivot/f9e7edbb5b74011a P03!L7
- gen-pivot/f9e7edbb5b74011a P04!P5
- gen-pivot/f9e7edbb5b74011a P06!O5:P5
- gen-pivot/f9e7edbb5b74011a P10!L5
- gen-pivot/f9e7edbb5b74011a P11!L5:N7

Also owner of these pivot-corpus render cells (T-4906, Excel-saved files: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)"), 8 render cells of 8 fresh pivots:
- pivot-corpus-closedxml/TryToLoad/TemplateWithTableSourcePivotTables.xlsx StressedAssetsPivot!D4
- pivot-corpus-lo/xlsx/pivot/Pivot3_Column_Grand_Subtotals.xlsx Sheet2!C3
- pivot-corpus-lo/xlsx/pivot/Pivot4_Column_Grand_Subtotals_SortDescending.xlsx Sheet2!C3
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_1_DataFieldInRow_RowLabelColor.xlsx Sheet1!G5
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_2_DataFieldInRow_ColumnLabelColor.xlsx Sheet1!G5
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_7_TwoRowTwoColumnFields_DataColor.xlsx Sheet1!J4
- pivot-corpus-lo/xlsx/pivot-table/PivotTableCellFormatsTest_8_DataFieldInRow_DataColor.xlsx Sheet1!G5
- pivot-corpus-lo/xlsx/pivot-table/PivotTableFormatsSubTotals.xlsx Sheet2!E3

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
