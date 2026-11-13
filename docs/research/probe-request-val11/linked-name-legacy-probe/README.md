# linked-name-legacy-probe (poc-engine-c-cc, 2026-11-12, Taskei P559880417)

One workbook, excel-graph (the same backend that filled `fixtures/external-link-cached-ranges`). 108 checked
cells, all legacy formulas (plain `<f>`, no `cm`), placeholder cached 0, on sheet `Refs`.

**Question.** A linked workbook's workbook-scoped name over several cells (`[1]!NR_Row` =
`'Uses'!$A$1:$C$1`), read where a legacy formula takes one value: does Excel intersect the name's area with
the formula's row or column, as it does the same range written in its place (`[1]Uses!$A$1:$C$1`, already
evidenced: `fixtures/external-link-cached-ranges` Refs!B44 is `#VALUE!` off the area), or does it take the
area's first cell, as the engine does now? And the same as an `IF`, `IFS` or `CHOOSE` arm, under `INDEX(…,0)`,
and as `SUM(name+0)`. Every name cell has the written range beside it (same row for the column name, the next
row for the row name), so each answer comes with its own control. The engine's change waits for this
(review of CR-322149590, both reviewers: no Excel evidence for the new behaviour).

**Input.** `linked-name-legacy.xlsx`, sha256 `eb6ac088e0594de281cba7b94d6c5d8afc62ee413002a9c9eb4d9a7927e795a1`,
built by `make_probe.py` from the repository's `eval/suites/fixtures/cases/external-link-cached-ranges.xlsx`
(Apache POI ref-56737.xlsx, Apache-2.0, Excel-written; that fixture's README entry lists its cache). Only two
parts change: the external link part gains workbook names `NR_Col` (`'Uses'!$A$1:$A$3`), `NR_One`
(`'Uses'!$B$1`) and `NR_Row` (`'Uses'!$A$1:$C$1`), and sheet `Refs` is replaced. The cache: `Uses` A1
"Hello!", B1 10, C1 `#N/A`, A2 5, B2 the text "7", C2 1, A3 TRUE.

**Layout.**
- Rows 2, 3 and 9, columns B to M: the column name (on its rows in 2 and 3, A2 5 and A3 TRUE; off in 9), one
  shape a column, name then written range: bare, `&""`, `SUM(x+0)`, `IF(TRUE,x)`, `CHOOSE(1,x)`, `INDEX(x,0)`.
- Rows 12 to 35, columns B, C and E: the row name (on its columns in B and C, B1 10 and C1 `#N/A`; off in E),
  one shape a row (label in column A), name row then written row: bare, `&""`, `SUM(x+0)`, `IF(TRUE,x)`,
  `IF(FALSE,0,x)`, `IFS(TRUE,x)`, `CHOOSE(1,x)`, `INDEX(x,0)`, `SUM(IF(TRUE,x))`, `SINGLE(x)`, a one-cell name
  (`NR_One`) against `[1]Uses!$B$1`, `ERROR.TYPE(x)`.

**What a refusal means.** If Excel for the web refuses the workbook, the refusal is the answer to record; the
source fixture opens there.

**Engine's answers** (`predictions.tsv`, frozen, the engine at main 2b05e4e58 with one unrelated change): every
name cell reads the area's first cell ("Hello!" for both names) wherever it is, where each written range is
intersected (B13 10, C13 `#N/A`, E13 `#VALUE!`; C2 5, C3 TRUE, C9 `#VALUE!`), except under `CHOOSE` and
`INDEX(…,0)`, where both read the first cell.
