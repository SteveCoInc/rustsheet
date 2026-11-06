---
id: T-4148
title: "xlsx export keeps empty-string cells (t=s to an empty string) instead of writing them blank"
milestone: M11
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A cell holding the empty string (a `t="s"` cell whose shared string is empty, or an inline empty string) stays a
text cell holding "" after load and save, not a blank cell.

## Context
Found by the 2026-11-04 round-trip census (docs/research/xlsx-round-trip-2026-11-04.md §2.2; method docs/research/xlsx-round-trip/README.md): `load_from_xlsx` then
`save_to_xlsx` with no edit and no recalculation, then `rustsheet-eval snapshot` of both files.
poi/bug66675.xlsx loses 3,604 such cells (`WGT-D01 By Production Day` B9 and others). poi 52348, 60512.xlsm and
xssf-enum.xltx, calipers pv23_056 and corpus-bench 124-46 lose a few. The difference is visible in Excel: ISBLANK is FALSE
and COUNTA counts the cell. Find whether import or export drops it. Check docs/excel-semantics/ for the empty-string
rules (M1 literals).

## Acceptance criteria
- [ ] After load and save, every cell of poi/bug66675.xlsx that held "" still holds "" (test), and ISBLANK of such a
      cell is FALSE after a reload.
- [ ] The census cases above have no missing cell left (`rustsheet-eval snapshot` before and after).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Style-only blank cells.

## Handoff
Sonnet run, 2026-11-06. Not a design I should decide, so escalated (work-loop 5b).
- The premise is wrong for plain cells. Tests in crates/rustsheet-xlsx/tests/empty_string_cell.rs (committed) show `t="s"` to an
  empty shared string, `t="str"` with no `<v>` or an empty `<v>`, and empty inline strings all load and save as `t="s"` text
  cells, and ISBLANK of A1 is FALSE after a reload.
- Real cause, reproduced on the real file (fetched from the poi suite's pinned rev; load, evaluate, save, then
  `rustsheet-eval snapshot` and `docs/research/xlsx-round-trip/semdiff.py`: missing 3604): the lost cells are the non-anchor cells of
  merged ranges. `WGT-D01 By Production Day` B9, C9, D9 are covered by `<mergeCell ref="A9:D9">`; the file stores them as `t="str"`
  empty cells, and E9 (the anchor of E9:G9) is kept as `t="s"` (si 7, ""). `clear_covered_cells`
  (crates/rustsheet-xlsx/src/import/merged_cells.rs) empties every covered cell on purpose: "covered cells must not hold content
  (an engine invariant), and Excel leaves them empty while third-party producers sometimes leave values behind".
- Open question for opus: does Excel keep a stored non-anchor value (here "") under a merge, so that COUNTA/ISBLANK see it, or
  empty it on load? If it keeps it, the invariant and `clear_covered_cells` change (an engine design change, probably a spec and a
  probe: a workbook with `<c r="B1" t="s">` "x" under B1:C1 plus `=ISBLANK(C1)`, `=COUNTA(A1:C1)`). If Excel empties it, the
  census "loss" is not a loss and this task should be closed with that evidence in docs/excel-semantics/.
- The other census cases (poi 52348, 60512.xlsm, xssf-enum.xltx, calipers pv23_056, corpus-bench 124-46) were not checked; they are
  probably merged-range cover too.
