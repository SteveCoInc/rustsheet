---
id: T-4272
title: "3-D spans after the sheet edits left unprobed: deleting an end sheet, moving the first of a two-sheet span past the last, a backwards span, and row or column inserts inside the span"
milestone: M8
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A 3-D span (`SUM(Jan:Mar!A1)`) in a formula or a defined name gives Excel's value and FORMULATEXT after each sheet
edit that docs/excel-semantics/references.md still lists as unprobed, with excel-graph evidence for each, and the engine
matches it.

## Context
- references.md "3-D references when sheets move or are renamed" ends: "Unprobed: moving the first sheet of a two-sheet
  span past the last (the engine writes the last sheet alone), a span written backwards (`Mar:Jan`, which the engine
  rewrites in tab order when it shrinks), deleting an end sheet and inserting or deleting rows or columns. The
  excel-graph backend cannot delete a sheet (T-1181), and the M14 desktop question is T-1310. The engine makes a span
  that lost an end sheet `#REF!` and does not shift a span for row or column edits."
- That reason is stale: the harness has a `delete_sheet` sheet op that excel-graph fills
  (probes/sheet-qualified-ref-error-text-delete-sheet, probes/table-edits/table-delete-sheet are oracle-filled), and
  T-1310 is dropped (decision 0266). So every point above can now be probed on excel-graph.
- Probe layout (`excel-probe` skill): reuse the probes/sheet-ops/sheet-ops-move-* cases (Calc holding
  `=SUM(Jan:Mar!A1)`, `=COUNT(Jan:Mar!A1:B1)` and FORMULATEXT beside each, Jan/Feb/Mar with A1 = 10, 20, 30), one
  sheet op per case, and the same formula in a defined name (sheet-ops-move-span-end-in-name's pattern). Cases, named
  `t4272-3d-span-*`:
  - move Jan to the end of a Jan:Feb span (the mirror of sheet-ops-move-two-sheet-span-end-first);
  - a backwards span `Mar:Jan` with no edit, and with Mar moved to position 1;
  - delete Jan (start), delete Mar (end), delete Feb (middle);
  - insert_row 1 and delete_row 1 on Jan, insert_column A on Feb, and the same on Calc (the formula's own sheet),
    reading `SUM(Jan:Mar!A1:A3)` with values in rows 1 to 4 so a shift shows.
- Code: `move_sheet_in_node` and the shared walker `for_each_sheet_reference_mut`
  (crates/rustsheet-engine/src/expressions/parser/stringify/sheet_edits.rs), the formula rewrite with undo in
  crates/rustsheet-engine/src/formula_rewrite.rs (shared by moves and sheet deletes since T-1691), `delete_sheet`
  (crates/rustsheet-engine/src/new_empty.rs, user_model/common.rs), and the row and column displacement of
  `insert_rows`/`insert_columns` (actions.rs). Extend these; do not add a second span walker (directive 3).
- Size guard: if the probes call for engine changes in more than two of the edit kinds (move, delete, row/column
  edits), land the probes, the docs and the first fixes here and file the rest with `make new-task`, each owning its
  cells, so this stays one M-size PR.
- No open task owns these edits (`rg 'two-sheet span|Mar:Jan|written backwards' docs/tasks` finds only done tasks).

## Acceptance criteria
- [ ] The `t4272-3d-span-*` cases are filled on excel-graph and every cell passes in `make eval` (or, under the size
      guard, is a known failure listed in a follow-up task filed here), with no other new F line in eval/baseline/probes.txt and every probes/sheet-ops/* cell still passing; a form Graph refuses is recorded with its
      error instead.
- [ ] references.md replaces the "Unprobed" sentence with the probed rules (VERIFIED, cases cited), and the stale
      T-1181/T-1310 remark goes.
- [ ] Each rule has a unit test in the engine's sheet-edit tests, including the undo of the edit restoring the text.
- [ ] `make check` green; reviewers approve.

## Out of scope
- External-workbook 3-D spans (`[1]Jan:Mar!A1`).
- Sheet renames (T-1691 and earlier evidence settled them).
