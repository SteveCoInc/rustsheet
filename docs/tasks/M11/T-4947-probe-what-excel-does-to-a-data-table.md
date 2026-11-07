---
id: T-4947
title: "Probe what Excel does to a data table on a row or column move through it, a cut or fill over its results, a one-cell clear and a sheet copy"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for the three data-table edits T-0452 left as our choice, so the engine follows
evidence there too (docs/excel-semantics/recalc.md, "Data tables follow row and column edits around
them and refuse edits that cut through them").

## Context
- T-0452 refuses a row or column move (`move_rows_action`, `move_columns_action`) that shifts any
  line of a table's region, and a cut whose source holds a result cell
  (crates/rustsheet-engine/src/data_table_edits.rs, `refuse_data_table_move`,
  `refuse_data_table_cut`). Neither is probed.
- Copying a sheet keeps its tables. Graph has no sheet copy, so excel-graph cannot answer this; a
  desktop backend (or a desktop-saved file holding a copied sheet with a table) can.
- Graph range `moveTo` may answer the move and the cut, as the T-0452 uploads did for inserts (each
  refused edit is a 500 `GeneralException`). Record each upload's edit, response and sha256.
- An autofill whose target holds a result cell is refused (`refuse_data_table_overlap`), and a
  single-cell clear of a one-cell table is refused by the write guard, although a clear covering
  every result of a larger table deletes it (probed): probe both.
- A paste over every result cell replaces the table, as a clear of them deletes it (probed); a paste
  over the whole results is unprobed too.

## Acceptance criteria
- [ ] Each point is probed (upload or case) or put to M14 Phase 4 if only the desktop can answer;
      recalc.md states each result.
- [ ] Any visibly-off behavior is fixed here if size S, or has an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- UserModel undo of these edits (T-4940).
