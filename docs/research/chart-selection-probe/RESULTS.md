# Results: which cells Excel for the web makes series names, categories and values (2026-11-10)

Recorded through Microsoft Graph (Excel for the web), as README.md allows: one workbook, one sheet per case, each
filled with `cases.js`'s values and charted with `charts/add` (`ColumnClustered`, `seriesBy: Auto`) in a session
that persists. Excel saved it as `chart-selection.graph.xlsx`
(sha256 43267a9de0cb951a91311776f0014b93177628bbedb62129082b716698666aa6). `chart-selection.graph.json` holds each
series' `c:tx`, `c:cat` and `c:val` as read from the saved chart parts. Recorder: `record_graph.py` (run from tools/).
Desktop Excel was not run (M14, decision 0039).

**Every case matches the reading README.md gives (poc/charts).**

| Case | Series (name; values) | Categories | Category axis |
|---|---|---|---|
| P1 | B1; B2:B13, C1; C2:C13, D1; D2:D13 | A2:A13, `numRef` | `c:dateAx` |
| P2 | A1; A2:A5 | none | `c:catAx` |
| P3 | A1; B1:D1 | none | `c:catAx` |
| P4 | A1; A2:A6, B1; B2:B6 (Year is a series) | none | `c:catAx` |
| P5 | B1; B2:B4, C1; C2:C4 | A2:A4, `strRef` | `c:catAx` |
| P6 | B1; B2:B4, C1; C2:C4 (date headers name the series) | A2:A4, `strRef` | `c:catAx` |
| P7 | no `c:tx`; B1:B4, no `c:tx`; C1:C4 (row 1 is data) | A1:A4, `strRef` | `c:catAx` |
