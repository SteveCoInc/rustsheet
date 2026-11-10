# Probe request: which cells Excel makes series names, categories and values when it charts a selection

*From the charts work, 2026-11-10. For the Excel oracle owner. Not urgent: the charts
lane has implemented its best reading, marked UNVERIFIED in `docs/excel-semantics/charts.md` ("Series from a
selection").*

## Why

Insert (and Office.js `charts.add` / Graph `charts/add`) builds a chart's series from a range by an undocumented
heuristic. Our evidence covers one shape only: Calipers' `A1:B5`, a header row over one text category column. The
e2e lane hit two other shapes in the v4 driver:
- a column of **dates** under a text corner, which we plotted as a series;
- a **single column** under a text header, where we left the header in the values.

Both are fixed in poc/charts to the reading below. These seven cases settle them and the older UNVERIFIED rules.

## How (either way)

- **Office.js** (as Calipers' `officejs-chart_*.script.js` do): run `cases.js`, case by case, in a fresh workbook,
  then save the workbook (`chart-selection.golden.xlsx`).
- **Graph:** write each case's values, then `POST /me/drive/items/{id}/workbook/worksheets/{sheet}/charts/add` with
  `{"type": "ColumnClustered", "sourceData": "<range>", "seriesBy": "Auto"}`, in a session with
  `persistChanges: true`. Then download the file.

From the saved file we read each chart part ourselves. For each series we need `c:tx` (its `c:f`), `c:cat` (its
`c:f`, and `numRef` or `strRef`) and `c:val` (`c:f`), plus whether the category axis is `c:catAx` or `c:dateAx`.
Nothing else is needed. The dates use the number format `mmm-yy`.

## The cases and our current reading

| Case | Sheet range and contents | Our reading (poc/charts) |
|---|---|---|
| P1 | A1:D13: A1 "Month", A2:A13 dates (`mmm-yy`), B1:D1 "Revenue", "Cost", "Margin", B2:D13 numbers | 3 series named B1, C1, D1; categories A2:A13 (numbers, date axis) |
| P2 | A1:A5: A1 "Cost", A2:A5 numbers | 1 series named A1, values A2:A5, no categories |
| P3 | A1:D1: A1 "Cost", B1:D1 numbers | 1 series named A1, values B1:D1, no categories |
| P4 | A1:B6: A1 "Year", A2:A6 2014-2018 (General), B1 "Sales", B2:B6 numbers | 2 series (Year and Sales), no categories |
| P5 | A1:C4: A1 empty, B1:C1 2023, 2024 (General), A2:A4 text, B2:C4 numbers | 2 series named B1, C1; categories A2:A4 |
| P6 | A1:C4: A1 "Region", B1:C1 dates (`mmm-yy`), A2:A4 text, B2:C4 numbers | 2 series named B1, C1; categories A2:A4 |
| P7 | A1:C4: A1 "Region", B1:C1 numbers 10, 20 (General), A2:A4 text, B2:C4 numbers | row 1 is data: 2 series with no names; categories A1:A4 |

`cases.js` has each case's exact values.
