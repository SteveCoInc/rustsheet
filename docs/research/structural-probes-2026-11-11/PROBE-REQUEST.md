# Probe request: the structural-edit unknowns (T-5123, T-5358, T-5265)

From the M4 lane (poc-m4-cc), 2026-11-11. Branch `m4/3-structural-probes`, commit `bbcc244ab` (cut from main
`4f6f5e19d`). Row and column inserts and deletes: what Excel does to references, tables and `#This Row` where the
engine follows a hand rule today. No engine change is on the branch.

## What is asked

Excel-graph values for **53 cases, 599 checked cells** (`rustsheet-case/v1`, `"expected": {}`, no sidecars).

- **48 cases, 561 cells, in the probes suite** on the branch (`eval/suites/probes/cases/table-edits/t5123-*`,
  `t5358-*`, `t5265-*`; copies are in this directory):

  ```sh
  cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes
  ```

  (`--case probes/table-edits/<name>.case.json` fills one.) Wanted back: the 48 `*.oracle.json` sidecars, committed
  on the branch or left here. No math sidecar is needed (`rustsheet-accuracy math --check --suite probes` is
  current with them).
- **5 cases, 38 cells, in `not-in-suite/`** (on the branch under
  `docs/research/structural-probes-2026-11-11/engine-refused/`). The engine refuses their edit today, so they
  cannot sit in the gate suite while pending (a case that does not run is a whole-case failure the baseline would
  have to record). Run each one by path:

  ```sh
  cd tools && uv run rustsheet-oracle recalc -b excel-graph ../docs/research/structural-probes-2026-11-11/engine-refused/<name>.case.json
  ```

  Wanted back: the JSON it prints, or the refusal (below).

## A refusal is an answer

The case format cannot record a refused edit: the fill fails at the edit and the case stays pending
(references.md, "Structured references under row deletes": 409 `InsertDeleteConflict` for a table's header row, no
case). For the cases below a refusal is the expected kind of answer. Please send back, for each refused case: the
case name, which op of `sheet_ops` was refused, and the HTTP status, Graph error code and message.

| Case | Edit | Why Excel may refuse |
|---|---|---|
| `not-in-suite/t5123-insert-row-last-row-nonblank` | insert row 1; S!A1048576 holds 5 | pushes a nonblank cell off the sheet (engine: refused) |
| `not-in-suite/t5123-insert-column-last-column-nonblank` | insert column A; S!XFD1 holds 5 | the column twin (engine: refused) |
| `not-in-suite/t5358-table-delete-rows-whole-table` | delete rows 3-7, exactly the table B3:C7 | takes the header row (engine: refused) |
| `not-in-suite/t5358-table-delete-rows-around-table` | delete rows 2-8 around the table | takes the header row (engine: refused) |
| `not-in-suite/t5358-table-delete-rows-header-and-data` | delete rows 3-5, header and two data rows | takes the header row (engine: refused) |
| `t5123-table-delete-rows-all-data-stacked` | delete rows 4-7; a second table's header is in row 4 | takes that table's header row (engine: accepted) |
| `t5358-no-header-delete-rows-whole-table` | delete rows 3-6, exactly a headerless table | unknown (engine: accepted) |
| `t5358-no-header-delete-rows-around-table` | delete rows 2-7 around a headerless table | unknown (engine: accepted) |
| `t5358-no-header-delete-rows-top-straddle` | delete rows 2-4, into a headerless table from above | unknown (engine: accepted) |

A refusal of any other case is a finding too; the same three things are wanted.

## The cases

| Task | Question | Cases (cells) |
|---|---|---|
| T-5123, U1 (with T-2851's whole-line shapes) | a whole-column range (`S!B:D`, `S!$B:$D`, `S!C:C`, and `=SUM(B:D)` on S itself) and a whole-row one (`S!4:7`, `S!$4:$7`, `S!5:5`, `=SUM(4:7)`) that loses its first line, last line, an inside line, every line, or lines across its first or last edge | `t5123-whole-column-delete-*`, `t5123-whole-row-delete-*`: 12 (96) |
| T-5123 (T-2851) | plain-range shrink with no table: a column delete at the first and last column of `S!C4:F4`, columns B:D across its edge, rows 6-8 across the last edge of `S!C4:C7` | `t5123-plain-range-delete-column*`, `-delete-rows-across-last-edge`: 4 (30) |
| T-5123, U2 | references pushed past row 1048576 or column XFD by an insert (a cell, a range end, a range wholly on the last line, whole-line ranges), and an insert that would push a plain nonblank cell off the sheet | `t5123-insert-row-past-last-row`, `t5123-insert-column-past-last-column`: 2 (28); 2 not in suite (8) |
| T-5123, U3 | an insert inside, at the first line of and just past a plain range with no table near, rows and columns | `t5123-plain-range-insert-*`: 6 (48) |
| T-5123, U6 | how a deleted single cell prints, from another sheet and on its own sheet, in a cell that moves and one that stays | `t5123-deleted-cell-delete-row`, `-column`: 2 (28) |
| T-5123, SPEC-0039 E6 | one delete of every data row of a table: with a totals row; with a second table stacked (its header in the first's first data row); with a second table side by side; with a calculated column in the kept row; and whether the column's formula survives (a row insert after it) | `t5123-table-delete-rows-all-data-*`: 5 (66) |
| T-5123 (T-3739) | `#This Row` through a row delete above and inside its table, a column insert left of and inside it, a column delete left of it: cells on another sheet and beside the table, and a column of `[@Qty]*2` inside it (no inside-the-table case for the insert left of it: `t4274-calc-column-structured-insert-column-a` answers that) | `t5123-this-row-*`: 9 (181) |
| T-5358 | a row delete of every row of a table: exactly, around it, header with data (header table, 3 not in suite); exactly, around it, into it from above (headerless table); and on the column axis, columns around a table and every column of a headerless one | `t5358-*`: 5 (52); 3 not in suite (30) |
| T-5265 | a deleted sheet's reference (`#REF!C4:C7`) through a row insert above its cell, a row insert on another sheet, a column delete left of its cell | `t5265-missing-sheet-*`: 3 (32) |

Each case's `description` states its question and layout cell by cell.

## Predictions

`predictions.tsv` (`case`, `cell`, `engine value today`; also on the branch,
`docs/research/structural-probes-2026-11-11/predictions.tsv`) holds the engine's answer at `bbcc244ab` for every
checked cell, recorded before any fill. Values are printed as `rustsheet-eval show` prints them (`14.0`, `"text"`,
`TRUE`, `#REF!`); the cells of the five cases the engine refuses carry the refusal message instead.
