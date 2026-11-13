# Probe request: which style Excel for the web gives a new pivot

*From poc-pivot-cc, 2026-11-12. For the Excel oracle owner, routed by the coordinator. Excel for the web only
(Office.js and a download): it needs no desktop Excel. It is the evidence for item 4 of decision 0459 (in review as
CR-322034534) and a criterion of T-5569.*

## Why

A workbook's styles part names a default pivot style (`tableStyles@defaultPivotStyle`), which Microsoft's Open XML
SDK reference describes as the style "to apply to new PivotTables". The engine is about to follow it: a new pivot
whose definition names no style takes the workbook's default, and PivotStyleLight16 only when the workbook names
none. That is the schema's reading. No evidence says Excel does it: the five Excel-saved pivots in the repository
are in workbooks whose default is PivotStyleLight16, and the one pivot we hold in a workbook whose default is
PivotStyleMedium9 names PivotStyleLight16, with nothing to say how it was made.

## How

For each of the three workbooks:

1. Open it in Excel for the web. It has a sheet Data (six records) and an empty sheet New.
2. Run `cases.js` (`runAll()`) through Office.js (Script Lab or the oracle's runner) and save the JSON it returns as
   `pivot-default-style-<m9|none|own>.answers.json` next to this file.
3. Download the workbook as Excel for the web saves it after the script ran (File > Save As > Download a Copy) and
   put it next to this file as `pivot-default-style-<m9|none|own>.saved.xlsx`. The pivot's own style name is in that
   file and nowhere in Office.js, so this step is the answer; step 2 is its cross-check.

If Excel shows a message on open (the third workbook defines a pivot style of its own, and its definition is
hand-written), or the script fails, note it in a line here: that is an answer too, and the other two still count.

## The workbooks

Built by `make_request.py` from `pivot-default-style.spec.json` with the repository's case builder, then given a
different `tableStyles` element each (SHA-256 of each file in `SHA256SUMS`). They differ in nothing else.

| Workbook | `tableStyles` in xl/styles.xml |
|---|---|
| pivot-default-style-m9.xlsx | `defaultPivotStyle="PivotStyleMedium9"`, a built-in style |
| pivot-default-style-none.xlsx | no `defaultPivotStyle` attribute |
| pivot-default-style-own.xlsx | `defaultPivotStyle="ProbePivot"`, a pivot style the workbook defines: one element, `headerRow`, an orange fill (`FFFFC000`) |

## What we expect, written before the answers

Nothing firm, which is why this is asked. If Excel follows the workbook's default, the saved pivot part of the first
workbook has `<pivotTableStyleInfo name="PivotStyleMedium9" .../>` and its header row is displayed with that
style's fill, and the third has `name="ProbePivot"` and an orange header row. If Excel for the web has one fixed
style for a new pivot, all three name the same style (PivotStyleLight16 is the likely one). The second workbook says
what an absent default becomes, and whether Excel writes the attribute back.

Also read from each saved workbook: the five switches of the new pivot's `pivotTableStyleInfo` (expected: row and
column headers on, no banding, last column on), and what `tableStyles@defaultPivotStyle` is after the save.

## What the answers change

- Decision 0459 item 4 and SPEC-0032 "Pivot styles": confirmed, or corrected to what Excel does.
- T-5568 (the facade's default for a pivot created without a style) and the collaborative sheet's UI, which writes
  the default's name into a new pivot's definition.
