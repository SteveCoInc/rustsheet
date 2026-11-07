---
id: T-4769
title: "Carry #3739's delete_row 2 calculatedColumnFormula probe onto main (T-4274 follow-up)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The probe that only the superseded T-4274 branch (#3739) holds, t4274-calc-column-delete-row-2 (excel-graph, filled),
is on main: the saved `calculatedColumnFormula` after a delete of a table's first data row is evidenced (VERIFIED) and
pinned by the export test, and the follow-ups that branch found are filed under fresh ids.

## Context
- T-4274 merged as #3738; its parallel run #3739 is a duplicate whose landing notes list what main lacks: the
  -delete-row-2 and -insert-row-2 probes and two follow-ups (its T-4679 and T-4698; main's T-4698 id is a pivot task).
- -insert-row-2 duplicates main's t4692-insert-row-first-data (T-4692), so it is not carried.
- Evidence (the branch's xlsx-storage.md text, from the saved `xl/tables/table1.xml` of the cached download): delete_row 2
  on T S!A1:C4 with C2 typed `=A2*2` saves `A2*2` (table `A1:C3`, shared `ref="C2:C3"` `A2*2`), not `#REF!*2`.
  Main's rule (xlsx-storage.md, "a text ... whose first data cell the edit deletes keeps its text (unprobed)") already
  gives `A2*2` here.
- Sheet F scores FORMULATEXT of a deleted cell's reference (F!A1): Excel `#REF!`.

## Acceptance criteria
- [x] probes/table-edits/t4274-calc-column-delete-row-2 on main with its oracle sidecar; baseline records only its
  real failures.
- [x] `a_calculated_column_formula_follows_row_and_column_edits` covers delete_row 2 (`A2*2`).
- [x] xlsx-storage.md quotes the evidence (no longer "unprobed" for this case).
- [x] Follow-ups filed: the `#REF!` delete probe (a read cell deleted) and any F-sheet gap.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other #3739 items main already has (T-4680, T-4692).

## Outcome
- Carried probes/table-edits/t4274-calc-column-delete-row-2 (case + excel-graph sidecar, unchanged from #3739's
  2c0d0541c) onto main: 5 of 6 cells pass on main as is (S!C2:C3 values, FORMULATEXT of the moved cells, the SUM,
  FORMULATEXT/ISFORMULA through INDIRECT); F!A1 (`=FORMULATEXT(S!C2)` after S!C2 is deleted, Excel `#REF!`, ours
  `#ERROR!`) is baselined and filed as T-4770.
- `a_calculated_column_formula_follows_row_and_column_edits` (rustsheet-xlsx test_tables.rs) gains delete_row 2 →
  `A2*2`, which main's rule already gives (a first data cell the edit deletes keeps its stored text).
- xlsx-storage.md quotes the delete_row 2 evidence from #3739 (table A1:C3, shared `ref="C2:C3"` `A2*2`).
- Not carried: -insert-row-2 (duplicates t4692-insert-row-first-data), #3739's T-4695 (duplicates T-4692) and its
  insert_columns fix (main's T-4680). Follow-ups: T-4770 (engine), T-4771 (research; was #3739's T-4679).
- Eval delta: +1 case, +6 checked cells, +5 passing; 1 new known failure (F!A1, T-4770); no regressions.
