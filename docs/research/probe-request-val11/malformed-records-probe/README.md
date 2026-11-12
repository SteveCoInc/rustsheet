# Probe request `malformed-records`: a repeated row record, overlapping `<col>` records, a reversed hyperlink `ref`

*From poc-design-cc, 2026-11-11, for the Excel oracle owner (routed by the coordinator). For Taskei P559416790.
T-5324 bounded what the xlsx importer keeps of three record forms, and `docs/excel-semantics/xlsx-storage.md`
records each as a resource-limit DEVIATION with "Evidence: none for Excel" ("A repeated row record keeps the
first, and a `<col>` is kept only while it covers a new column"; the hyperlink entry's "A reversed `ref` … links
no cell"). This request asks Excel.*

## How

- `python3 -I generate.py <dir>` writes six workbooks, three cases and a control for each. A control differs
  from its case only in being well formed, and holds what the importer makes of the case.
- For each, with `<dir>` an absolute path (the command runs in `tools/`), and a control first: `cd tools && uv
  run rustsheet-oracle recalc -b excel-graph <dir>/<file> > <dir>/<file>.graph.json 2> <dir>/<file>.graph.err`.
  The tool's canary writer accepts all six (checked here, without the network). Please return, for each file,
  the JSON output, or the `.err` file if the backend refused the workbook (a refusal such as
  `FileCorruptTryRepair` is an answer, beside an empty JSON), and the recalculated workbook, which the tool
  keeps as `eval/.cache/oracle/excel-graph/<sha256>.xlsx` under the file's sha256 below: what Excel wrote back
  for the rows, the columns and the hyperlinks is read from that file. If Excel refuses a control, stop there:
  the package is then the suspect, not the record.
- `python3 -I check.py` runs the generator and reads the workbooks back: each case holds the record form this
  README says, each control the well-formed one, and the sha256 values below are the files'. It needs no Excel
  and says nothing about Excel's values.

## The cases

| File (sha256) | What it holds | Questions |
|---|---|---|
| `row-repeated.xlsx` (sha256 `cde06ff6252f5801682c5206984e0d356ac40790135636cdf697454afff1af78`) | A1 = 1, A3 = 100, and two records for row 2. The first: height 30, cell format 1, A2 = 10. The second: height 60, hidden, cell format 2, A2 = 20 and B2 = 5. D1 `=A2+B2`, E1 `=SUBTOTAL(109,A1:A3)`, F1 `=AGGREGATE(9,5,A1:A3)` (stored with its `_xlfn.` prefix), G1 `=SUM(A1:B3)` | Does Excel open it? Which record's cells does row 2 hold (D1: 25 if the second's, 10 if the first's, 15 if only B2 is added)? Is row 2 hidden to SUBTOTAL and AGGREGATE (E1 and F1: 121 or 111 if not, 101 if hidden)? In the file Excel writes back: one record for row 2 or two, and with which height, format and `hidden`? |
| `row-repeated-control.xlsx` (sha256 `6d42e841ade0e5a48c63cad55f3c061995d6c2904d2be2085fc9282652e55806`) | the same with one record for row 2 that holds what the importer makes of the two: the first record's height, format and visibility, and the cells as the second leaves them (A2 = 20, B2 = 5) | the control: D1 = 25, E1 = F1 = 121, G1 = 126, as the engine gives for both files |
| `col-overlap.xlsx` (sha256 `13613707e17834766576e021cdd9976d6154aa65819e8589db8b62558b4abed9`) | `<col min="1" max="5" width="20">`, then `<col min="2" max="3" width="40" hidden="1">`; A1:C1 = 1, 2, 3; G1 `=SUM(A1:C1)` | Does Excel open it? In the `<cols>` Excel writes back: which width do B and C have, and are they hidden? (No formula can say on Excel for the web, where `CELL("width")` is `#VALUE!`.) |
| `col-overlap-control.xlsx` (sha256 `2a35c01885bd143a83eb4a3813c22722a724eb81a99a15fa18b6d943edefef66`) | the same with the first `<col>` alone, which is what the importer keeps | the control |
| `hyperlink-reversed.xlsx` (sha256 `91a91e7c7e5fb96f329ba163158d9cc081c5209c6a473230150a4c72b4650e42`) | A1:B2 = 1, 2, 3, 4; `<hyperlink ref="B2:A1" location="S!D5">`; G1 `=SUM(A1:B2)` | Does Excel open it? In the file it writes back: is the link there, and over which `ref` (`A1:B2`, `B2:A1`, one cell, none)? |
| `hyperlink-reversed-control.xlsx` (sha256 `619389d5b5b964385a56e7c29220a1038a0824d76edf2063e312cc3ba5f1395b`) | the same with `ref="A1:B2"` | the control |

## What the engine does today

From the two entries of `xlsx-storage.md`, and measured on main 621a8367b by loading each workbook through the
importer and calculating it:

- **A repeated row record.** The importer keeps the first record's height, format and visibility, and "the row's
  cells are not affected: a repeated row's cells replace the earlier ones". So row 2 is not hidden and holds A2
  = 20 and B2 = 5: D1 = 25, E1 = F1 = 121, G1 = 126, for the case and for its control alike.
- **Overlapping `<col>` records.** A `<col>` is kept only while it covers a column no earlier one does, so the
  second record is dropped. G1 = 6 for both files.
- **A reversed `ref`** links no cell. G1 = 10 for both files.

None of the three changes a calculated value, except through what `SUBTOTAL` and `AGGREGATE` ignore as hidden.

## What the answers decide

Each answer goes into `xlsx-storage.md` beside the DEVIATION it concerns. Where Excel opens the file and keeps
something the importer drops (the second record's cells, a hidden column, the link), the entry gains what Excel
shows and an owner task for the importer. Where Excel refuses the file, the deviation is moot for files Excel
wrote, and stays as the bound for files it did not.
