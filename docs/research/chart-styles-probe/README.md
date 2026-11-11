# Probe request: what Excel writes for each Chart Styles preset

*From the charts work, 2026-11-10. For the Excel oracle owner. Not urgent: the Styles half of Chart Design >
Chart Styles. The Colour half is done.*

## Why

Our Chart Styles shows Excel's Colour gallery today, but none of Excel's style presets ("Style 1" to "Style N"
for each chart type). To build them, we need what Excel writes into the package for each preset. Guessing them is
not acceptable.

What we have on disk is each type's default preset only. These are the `cs:chartStyle id` values in the Excel-saved
goldens of `crates/rustsheet-charts/tests/corpus` on poc/charts:

| Default id | Chart type |
|---|---|
| 201 | clustered column |
| 216 | bar |
| 227 | line |
| 240 | scatter |
| 251 | pie and doughnut |
| 276 | area |

The goldens also hold 262, 366, 381 and 395. No file holds a second preset of any type.

## How (either way)

- **Excel desktop or Excel for the web, by hand:**
  1. In a fresh workbook, give each chart type its own sheet with `cases.js`'s data, and insert that type's chart from
     A1:D5.
  2. Copy the chart once per preset. On copy *k*, pick Chart Design > Chart Styles > Style *k*.
  3. Save the workbook as `chart-styles.golden.xlsx`.
- **Office.js:** the same, if the oracle's API set can apply a preset to a chart. We have not confirmed that it can.
  Please say which API was used.

## What we read from the saved file (we parse it ourselves)

For each chart, all of these:
- its chart part's `c14:style` and `c:style`;
- the `cs:chartStyle` part it relates to (its `id` and the whole part);
- its `cs:colorStyle` part;
- the chart part's own `c:spPr`, `c:txPr`, series and data-label formatting, which a preset writes there as well.

Please name each sheet by its type, and each chart `<type> style <k>` (Selection Pane name), so we can tell them
apart without guessing.

## Order, if the full set is too much at once

1. Clustered column.
2. Line with markers.
3. Pie.
4. Then bar, area, doughnut, scatter, and the combo (Clustered Column - Line on Secondary Axis).
