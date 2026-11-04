---
id: T-4021
title: "Verify that the excel-graph replay of convert_table, delete_table and add_table took effect"
milestone: M8
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The excel-graph backend fails a case whose `convert_table`, `delete_table` or `add_table` replay Excel did not
carry out, instead of recording values of a workbook the op never changed.

## Context
- T-3629 (excel-graph 2026-11-04): with a `#This Row` reference to table Sales on another sheet, Graph's
  `POST tables('Sales')/convertToRange` answered success and left the table in place (the saved workbook still
  defines Sales on B3:C7); an `add_table` of Sales on K8:L11 after it also answered success, and its rename
  read-back saw "Sales", yet the saved file holds only the old table. Those cases were dropped from T-3629.
- Replay: `_replay_sheet_ops` in tools/src/rustsheet_tools/oracle/backends.py; the move op already reads its
  result back (T-1608).

## Acceptance criteria
- [ ] After each table op the backend lists the sheet's tables (`GET worksheets('S')/tables`) and raises
  OracleError unless the op's effect is there (gone after convert/delete; present with its name and range after add).
- [ ] Tool tests with a fake Graph that ignores the op.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other sheet ops.
