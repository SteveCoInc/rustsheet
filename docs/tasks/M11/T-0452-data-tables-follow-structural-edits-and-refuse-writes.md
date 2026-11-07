---
id: T-0452
title: "Data tables follow structural edits and refuse writes into their result cells"
milestone: M11
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0441]
---

## Goal
A what-if data table keeps describing the same cells after rows, columns or sheets around it
change, as in Excel. A write into one of its result cells is refused, as Excel refuses one ("Cannot
change part of a data table").

## Context
T-0441 added data tables as sheet metadata: `Worksheet::data_tables`, a `DataTable` range plus a
row and a column input cell (crates/rustsheet-engine/src/types.rs, data_tables.rs). Nothing yet
updates that metadata:
- Inserting or deleting rows or columns (crates/rustsheet-engine/src/actions.rs) moves cells and
  rewrites formulas, but not a table's range or input cells. A table then computes the wrong cells.
- Deleting a row or column that holds an input cell should make that input `Deleted` (`#REF!`, as
  `del1`/`del2` say in the file).
- `set_user_input` and the facade's `Txn::set` overwrite a result cell, and the next evaluation
  writes the table's value back over it.
- Copying a sheet clones the metadata. Check what Excel does when the copy is made.
Probe Excel first (excel-probe skill) for:
- an insert above, inside and left of a table;
- deleting an input cell's row;
- writing into a result cell.
Record the answers in docs/excel-semantics/recalc.md.

## Acceptance criteria
- [x] Probe cases cover the edits above and are oracle-filled; recalc.md records the rules.
- [x] Row and column inserts and deletes move or shrink a table's range and its inputs, or mark the
      inputs `Deleted`, as Excel does. Unit tests cover each.
- [x] Writing into a result cell is an error through the engine and the facade, and an undo of
      `add_data_table` still works.
- [x] `make check` green; reviewers approve.

## Out of scope
- Recompute cost (T-0453).

## Outcome
- Probed (excel-graph, 2026-11-07): Excel treats a data table's region (results plus the row above
  and column left) as one block. Inserts strictly inside it and deletes taking part of it are
  refused, as is every write into a result cell (value, formula) and a clear of some of them; a
  clear of every result deletes the table. Edits above or left move the table and its input;
  deleting the input's row or column makes it `#REF!`; deleting the whole region deletes the table.
  probes/t0452-data-table-row-column-edits (60 cells, the accepted edits) plus 34 single-edit
  uploads, each listed in recalc.md with its response.
- Engine: `data_table_edits.rs` refuses the splitting edits before anything changes and moves,
  `#REF!`s or drops tables after the accepted ones (`insert_rows`, `delete_rows`,
  `insert_columns`, `delete_columns`); row and column moves touching a region are refused
  (unprobed, our choice); `prepare_cell_for_user_input` refuses writes into results, so the
  facade's `Txn::set` errors and rolls back. `range_clear_contents` and `range_clear_all` refuse a
  clear of some results and delete a table whose results they cover; a paste over some results, or
  a cut from them (our choice), is refused before it changes anything. Undo of `add_data_table` is
  tested.
- Copying a sheet: Graph has no sheet copy; recorded as unprobed (the copy keeps its tables).
- Eval delta: probes +60 cells, all pass (on main the engine passed 36/60). No existing case moved.
- Follow-ups: T-4940 (UserModel undo of a delete or clear that `#REF!`ed an input or dropped a
  table); T-4947 (probe a move through a table, a cut from it and a sheet copy).
