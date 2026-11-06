# T-4089: a 255-argument `_xlfn.VAR.S` file does not open in Excel for the web

Scratch recalcs with `rustsheet-oracle recalc -b excel-graph`, 2026-11-06. Each workbook was built by
`rustsheet_tools.spec2xlsx.build_xlsx` from a one-cell case (sheet `S`, cell A1):

| A1 as typed in the case | file form | result |
|---|---|---|
| `=VAR.S(1,2,...,255)` | `_xlfn.VAR.S(...)` | `createSession` 501 `FileCorruptTryRepair` / `unsupportedWorkbook` |
| `=VAR.S(1,2,...,254)` | `_xlfn.VAR.S(...)` | opens and calculates |
| `=VAR(1,2,...,255)` | `VAR(...)` | opens and calculates |

The same 501 failed whole Gate A batches of VAR.S and STDEV.S (`eval/accuracy/tables/{VAR.S,STDEV.S}/s1`) while they
held 255-argument calls. The VAR, VARA, STDEV, STDEVA and DEVSQ batches, which are not `_xlfn.` functions, held 255
arguments and filled. To reproduce, build the three cases above and run the recalc. The 255-argument case fails to open.
