# cse-row-column-area-probe (poc-engine-c-cc, 2026-11-12, Taskei P561218625)

One workbook, excel-graph. Legacy (CSE) array formulas over areas (`<f t="array" ref=..>`, no `cm`), placeholder
cached 0, on Sheet1; 36 checked cells.

**Question.** In a CSE array formula entered over several cells, is `ROW()` (no argument) the rows of the formula's
whole area and `COLUMN()` its columns, as desktop Excel 16.x's caches in two corpus files show (a k-th-match formula
`{=IFERROR(INDEX(..,SMALL(IF(key=..,ROW(..)-W64,""),ROW()-W64)),"")}` over 30 cells caches the first match, then ""),
or the anchor's row and column? And how does such a formula's FORMULATEXT read?

**Input.** `cse-row-column-area.xlsx`, sha256 `13635a9720ed379cd82c143739e8a6b944f6f75eabdfc40d73c581c931d1031b`,
built by `make_probe.py` from the repository's `eval/suites/fixtures/cases/valueless-cell-forms.xlsx` (hand-written
raw XML; only its shared strings and Sheet1 change). A1:A4 keys k, m, k, k; B1:B4 items a, b, c, d; D1 the key k.

**Layout.** F1:F4 the k-th-match formula; G1:G3 `{=ROW()}`; H5:J5 `{=COLUMN()}`; K2:L3 `{=ROW()*10+COLUMN()}`; A7 a
one-cell `{=ROW()}`; C7:C9 `{=ROWS(ROW())}`; D7:F7 `{=COLUMNS(COLUMN())}`; G7:G9 `{=SUM(ROW())}`; H7:H9 `{=ROW()-6}`;
N1:N9 FORMULATEXT of each anchor (plain formulas).

**Engine's answers** (`predictions.tsv`, frozen): main (anchor row and column for the whole area: F1:F4 "a" four
times, G1:G3 1, 1, 1) and the fix's branch (the area's rows and columns: F1:F4 a, c, d, ""; G1:G3 1, 2, 3; K2:L3 31,
32, 41, 42; C7:C9 3; G7:G9 24).
