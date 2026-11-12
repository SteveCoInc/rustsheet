# sheet-defaults-probe (poc-oracle-cc for poc-ui-b-cc, 2026-11-12; Taskei P561741584, T-5650)

A tier-1 render probe: what Excel for the web makes of a sheet's `<sheetFormatPr>`. Three rules have no
evidence, and the xlsx import hands the stored values on without computing pixels until they do
(`docs/excel-semantics/render.md`, the UNVERIFIED entry "A sheet's default column width and row height from its
`<sheetFormatPr>`"). 257 of the repository's 513 Excel-written sheets take the first branch and 359 the second.

One workbook, `sheet-defaults.xlsx` (7,643 bytes, 12 sheets, 186 cells in the probe ranges), written at the XML
level by `build.py` (xlsxwriter always writes `defaultRowHeight="15"` and never a `baseColWidth` alone). Its
bytes are the pin: sha256 `8431d9c5ff3b0cb98b17825c6343760a8073ee967fd8647581fc2c509b896e48` (`SHA256SUMS`,
`probes.json`). Do not open and re-save it. The default font is Calibri 11, as in the other set-1 probes; column
A of every row says what the row or sheet holds, in OOXML's own words.

## How to record

No change to the repository is needed: the recorder takes a probe directory.

```sh
cd tools
uv run rustsheet-oracle render --probes <this directory> --out <this directory>/refs --probe sheet-defaults
```

It writes `refs/sheet-defaults.render.json` (every column width and row height of each sheet's range, in
points) and `refs/sheet-defaults.pdf` (Excel's print). A refusal to open the file is written as
`refs/sheet-defaults.refused.json` and is an answer too.

## The questions

| Sheet | `<sheetFormatPr>` | What to read | Decides |
|---|---|---|---|
| `Base10` | `baseColWidth="10" defaultRowHeight="15"`, no `<cols>` | `columnWidth` of A to D | (1) the width of a column with no `<col>`: 75, 78 or 80 px (56.25, 58.5 or 60 pt) |
| `Base8`, `Base5`, `Base12`, `Base20` | the same with 8 (the schema's default, written out), 5, 12, 20 | the same | which rule gives it: five points tell `7·n + 5` from a rounding up to a multiple of 8 px and from anything else |
| `DefCol12` | `defaultColWidth="12"`, no `baseColWidth` | the same | the import's other branch, as a control |
| `Base10DefCol12` | both | the same | which of the two wins when a file has both |
| `RowDef14_4` | `defaultRowHeight="14.4"`, no `customHeight` | `rowHeight` of rows 1 to 5 | (2) is 14.4 the height of a row with no `ht`: one with cells (1, 2), one with no `<row>` (3), an empty `<row/>` (4); row 5 is `ht="20" customHeight="1"`, a control |
| `RowDef14_4Custom` | the same with `customHeight="1"` | the same | what the flag changes |
| `RowDef18`, `RowDef18Custom` | 18, without and with the flag | the same | the same at a height that is no rounding neighbour of 15 |
| `RowHt` | `defaultRowHeight="15"` | `rowHeight` of rows 2 to 17 | (3) how a row's own `ht` is drawn: 14.4, 15.5, 14, 14.9, 15.1, 16.2, 12.3 and 30.1, each with `customHeight="1"` and without |

Graph reports points; pixels are points / 0.75. A row with no custom height read 15.0 in the first recording
even where the print showed it taller (`eval/render/README.md`), so the PDF is read beside the numbers.

## What is checked here, and what is not

- Checked: every part parses; each sheet has `sheetViews`, `sheetFormatPr`, `sheetData` in the schema's order
  and the attributes the table names; the repository's reader opens it (`rustsheet-eval snapshot`); the
  recorder's own loader takes the directory (`render.load_probes`, `pinned_bytes`, `range_bounds`); two builds
  give the same bytes.
- Not checked: that Excel opens it. Nothing here has been shown to Excel.
- `../import-load-probe/` (in this change) (poc-cfix-uiperf-cc) asks a neighbouring question: `defaultRowHeight="30"`
  with and without `customHeight`, and whether a wrapped row grows.

## For the gate

`probes-entry.json` is the entry for `eval/render/probes/probes.json` in its own format; the file and the entry
are poc-e2e-cc's to add (the manifest is read by the e2e gates package). The oracle service (`infra/oracle`)
does not carry render recordings: it answers recalculations only.
