---
id: T-4698
title: "FORMULATEXT of a cell reference a row delete removed is #REF!, not #ERROR!"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=FORMULATEXT(S!C2)` on another sheet, after a row delete removes S!C2, evaluates to `#REF!` as in Excel; ours gives
`#ERROR!`.

## Context
- Evidence: probes/table-edits/t4274-calc-column-delete-row-2 F!A1 (excel-graph, T-4274): F!A1 `=FORMULATEXT(S!C2)`,
  delete_row 2 on S; Excel `#REF!`, ours `#ERROR!`. S!C2 was the first data row of a table, so check first whether a
  plain cell (no table) gives the same, and whether only FORMULATEXT or every function over the displaced `S!#REF!`
  reference is affected (`#ERROR!` suggests a parse error of the displaced text).
- Code: the displaced reference text of a deleted cell (stringify.rs, `wrong_reference.rs`, "A wholly deleted plain
  range prints one sheet-qualified `S!#REF!`", references.md, T-2855) and how it reparses.

## Acceptance criteria
- [ ] t4274-calc-column-delete-row-2 F!A1 passes; its F line leaves eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The table's stored column formula (T-4274).
