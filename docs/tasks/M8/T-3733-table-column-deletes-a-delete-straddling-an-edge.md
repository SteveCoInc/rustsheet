---
id: T-3733
title: "Table column deletes: a delete straddling an edge or removing every column match Excel"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine's column deletes that straddle a table edge or remove every column of a table give Excel's values and FORMULATEXT text.

## Context
T-2899 probes (excel-graph 2026-11-01, docs/excel-semantics/references.md "Structured references under row deletes, column edits"):
- table-delete-columns-straddle-left (delete B:C, table on C3:E7): the table survives without the deleted column, `COLUMNS(Sales)` 2,
  `INDEX(Sales[#Headers],1,1)` "Qty", `SUM(Sales[Qty])` kept as `=SUM(Sales[Qty])`, `Sales[Region]` is `#REF!` (`=COUNTA(#REF!)`).
  The engine falls back to a plain range (`#REF!` for `COLUMNS`/`ROWS`).
- table-delete-columns-straddle-right (delete D:E, table on B3:D7): same, the last column is dropped and the table keeps two.
- table-delete-all-columns (delete B:C, the whole table): every reference is `#REF!`, `ISREF(Sales)` FALSE, `COUNTA(Sales[Region])` 1;
  the engine matches in value but FORMULATEXT prints `=SUM(S!#REF!)` where Excel prints `=SUM(#REF!)`.
Code: table_edits.rs `edit_columns`, `column_span`; stringify/structured.rs `deleted_column`.
- Cells (`make eval-show CASE=<case>`):
  - probes/table-edits/table-delete-columns-straddle-left.case.json (6): F!A2, A3, A4, A6, A8, A9
  - probes/table-edits/table-delete-columns-straddle-right.case.json (4): F!A3, A4, A6, A9

## Acceptance criteria
- [x] The three probes above (eval/suites/probes/cases/table-edits/) pass and leave eval/baseline/probes.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
- Column ranges that lose one end (T-3734).


## Outcome
A column delete straddling the left or right edge of a table now shrinks it (`column_span`; the column edit removes only the overlapped columns), and a delete of every column prints the table's structured references as `#REF!` (`Displacement::deleted_tables`) instead of `S!#REF!`. The three probes pass and left eval/baseline/probes.txt (11 cells). Rule recorded in docs/excel-semantics/references.md.
