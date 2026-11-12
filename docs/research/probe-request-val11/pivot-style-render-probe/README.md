# Probe request: what Excel for the web paints for a pivot table style, and how it formats row labels

*From poc-pivot-cc, 2026-11-12. For the Excel oracle owner, routed by the coordinator. Excel for the web only
(Office.js): it needs no desktop Excel. It is the evidence for the owner's ruling of 2026-11-12 that pivots render
Excel's styles (decision 0459, tasks T-5567 and T-5569), and for T-5556 (row label alignment and indent).*

## Why

The engine will give the collaborative sheet each pivot's style resolved to colours (the built-in definitions come
from ECMA-376's preset styles), and the UI decides which cells each element of the style paints and which element
wins where two meet. Neither half has evidence: the excel-graph oracle reads values and text, not formats. Office.js
has the read we need: `Range.getDisplayedCellProperties` (API set ExcelApi 1.19) returns "properties that are
displayed from indirect sources, such as conditional formatting or styles", and `Range.getImage` (ExcelApi 1.7)
renders a range as a PNG.

The same read answers a second question. Two workbooks Excel saved store every compact row label with an explicit
left alignment and its level as the indent; all their labels are text. What a numeric or a date row label gets, and
what outline and tabular form get, is not known.

## How

1. Open `pivot-style-render.xlsx` in Excel for the web. Its fifteen pivots are `refreshOnLoad`: Excel lays each out on
   open. Do not change anything.
2. Run `cases.js` (`runAll()`) through Office.js (Script Lab or the oracle's runner).
3. Save the JSON it returns as `pivot-style-render.answers.json` next to this file. If Excel shows a message on open
   or a sheet shows `#SPILL!` or nothing, note it in a line here: that is an answer too.

The script writes nothing. For each pivot it returns: the pivot's range and its row-label, column-label and
data-body ranges; the text and values of every cell; the format each cell holds itself (`getCellProperties`); the
format Excel displays (`getDisplayedCellProperties`); the same for the report filter's cells; and a picture. Where
the host lacks ExcelApi 1.19, `displayedCellProperties` is `false` and `displayed` is null: please say which host
and version that was.

## The workbook

Built by `make_workbook.py` from `pivot-style-render.spec.json` with the repository's case builder, then each pivot's
`pivotTableStyleInfo` set (SHA-256 of each file in `SHA256SUMS`). One source, `Data!A1:I23` (Region, Product, Size,
Channel, Kind, Rep, Sales, Units, When; 22 records, some intersections left empty), one pivot a sheet at A3.

Style sheets: compact form, rows Region > Product > Size, columns Channel > Kind, report filter Rep, Sum of Sales,
subtotals and grand totals on, a blank row after each Region.

| Sheet | Style | Row headers | Column headers | Banded rows | Banded columns | Last column |
|---|---|---|---|---|---|---|
| L16 | PivotStyleLight16 | on | on | off | off | on |
| L16B | PivotStyleLight16 | on | on | on | on | on |
| M9 | PivotStyleMedium9 | on | on | off | off | on |
| M9B | PivotStyleMedium9 | on | on | on | on | on |
| D2 | PivotStyleDark2 | on | on | off | off | on |
| D2B | PivotStyleDark2 | on | on | on | on | on |
| L16N | PivotStyleLight16 | off | off | off | off | off |
| L16S | PivotStyleLight16 | on | on | not written | not written | on |
| L16X | PivotStyleLight16 | not written | not written | not written | not written | not written |
| L16C | PivotStyleLight16 | on | on | off | off | off |

L16 has the switches Excel writes for a new pivot. L16N turns every switch off, to show what each one adds.

**Added 2026-11-12 22:15Z (three sheets, before any answer was filed; the workbook, the spec, the script and
`cases.js` changed, and `SHA256SUMS` holds the new hashes).** L16S and L16X leave switches out of the file: nine
pivot parts of the corpus have no stripe switch, and Microsoft's note on ECMA-376 §18.10.1.74 says Excel reads a
missing switch as off. If that holds, L16S looks like L16 and L16X like L16N. L16C is L16 with the last column off
alone: the same note says Excel ignores `showLastColumn`, so L16C should look like L16. These three decide what the
engine tells the UI for a file's missing switch (T-5566, T-5568). If you already ran the first version, its ten
answers stand: run `runAll()` again in this workbook for the new sheets (it reads all fifteen).

**Added 2026-11-12 22:35Z (two more sheets, AM and AI, and the script now also returns the merged ranges inside
each pivot, `merged`).** They answer the pivot UI's request P-5, which the excel-graph fill cannot (it returns
values, not formats). AM is tabular form, rows Region > Product, columns Channel, with "Merge and center cells with
labels" (`mergeItem="1"`): which label cells Excel merges on open and how each label is aligned. AI is compact
form, rows Region > Product > Size, with the pivot's `indent="3"` (the other sheets have 0): the `indentLevel` of
each inner label.

Label sheets: rows Region (text) > Units (numbers) > When (dates), columns Channel, Sum of Sales, PivotStyleLight16.
AC is compact form, AO outline form, AT tabular form.

## What we expect, written before the answers

- **Colours.** PivotStyleLight16's header row is accent 1 at tint 0.8 with bold dark text and a thin accent 1 (tint
  0.4) rule below; its first row stripe is the light colour at tint -0.15. In the default Office theme (accent 1
  `#4472C4`) the plain luminance rule gives about `#D9E1F2` for the header fill; the engine's tint, which works on
  whole percents, gives `#D8E2F3`. If Excel shows `#D9E1F2`, the engine's tint is a level off and gets fixed.
- **Which cells.** Not written down on purpose: the UI lane's mapping (header row, subheadings by level, subtotal
  rows by level, stripes counted over item lines, the grand total row and column, the filter labels and values) is
  frozen in its own tests, and the answers are compared with it.
- **Labels.** In AC every item label and the Grand Total label in column A is left-aligned with its level as the
  indent (0, 1, 2), whatever its type, and the cell holds that alignment itself. In AO and AT: unknown; the engine
  writes no alignment today.

## What the answers change

- T-5567: the engine's resolved colours for the three styles are checked against `displayed` cell by cell.
- The UI lane's painter: which cells each element paints, and the precedence, for the seven style sheets.
- T-5556: the alignment and indent the grid writer gives a row label, for each form and item type.
