---
id: T-3734
title: "Structured column range Sales[[A]:[B]] is #REF! when either end column is deleted"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A structured column range `Sales[[Qty]:[Val]]` is `#REF!` (value and FORMULATEXT) as soon as either end column is deleted.

## Context
T-2899 probes (excel-graph 2026-11-01): table-column-range-delete-end (Val deleted) and -delete-start (Qty deleted) give
`SUM(Sales[[Qty]:[Val]])` `#REF!`, `COLUMNS(...)` `#REF!` and FORMULATEXT `=SUM(#REF!)`; the engine prints the displaced plain
area instead. Deleting both ends (one edit or two) is `#REF!` too and already matches in the one-edit case
(table-column-range-delete-both-ends passes in full). The plain `S!C4:D7` after a one-column
delete shrinks to `S!C4:C7` in Excel (probe cells F!A4, F!A5): a column-delete corner of T-2851's.
Code: stringify/structured.rs `deleted_column`.
- Cells (`make eval-show CASE=<case>`):
  - probes/table-edits/table-column-range-delete-both-ends-separate.case.json (5): F!A1, A2, A3, A4, A5
  - probes/table-edits/table-column-range-delete-start.case.json (3): F!A2, A4, A5

## Acceptance criteria
- [x] table-column-range-delete-end, -delete-start and -delete-both-ends-separate: the structured cells (F!A1-A3) pass.
- [x] The plain-range cells F!A4/F!A5 of those probes pass too (`S!C4:D7` shrinks to `S!C4:C7` after a one-column delete), or, if
      that needs T-2851's wider probe first, move to a task that lists them.
- [x] `make check` green; reviewers approve.

## Out of scope
- Plain-range shrink beyond these probes: whole-row ranges and multi-row deletes (T-2851).


## Outcome
`deleted_column` (stringify/structured.rs) now treats a column range that loses either end as
`Whole`, so the reference prints `#REF!` (value and FORMULATEXT), matching the T-2899 probes. The
`RangeEnd` variant is gone. Eval: 7 probe cells newly pass and left `eval/baseline/probes.txt`
(delete-end F!A1-A3, delete-start F!A2, delete-both-ends-separate F!A1-A3). The plain-range cells
F!A4/F!A5 of these probes (`S!C4:D7` shrinking to `S!C4:C7` after a one-column delete) still fail:
they need the plain-range column-delete shrink, filed as T-3978 with the oracle evidence (T-2851 does not list them), so they stay in the
baseline and are not part of this change.
