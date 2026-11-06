---
id: T-4715
title: "references.md and T-3739 say which T-4680 column-edit #This Row rules are hand extensions"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/references.md and T-3739's task file say exactly which of T-4680's column-edit `#This Row` rules
Excel evidence backs and which are hand extensions.

## Context
- T-4680 (PR #3768) makes a `#This Row` reference to a table that a column insert or delete left of it moves whole keep
  its text. The only evidence is probes/table-edits/t4274-calc-column-structured-insert-column-a (excel-graph): cells
  inside the table, after an insert.
- The excel-fidelity review of #3768 (head 8a9f793b6) approved with MINOR findings:
  - references.md ("Structured references when the table changes", Implemented in) says the text is kept "the same
    way" without marking two parts as unprobed. One is a reference outside the table (beside it, or on another sheet,
    such as F!A2 `=T[@Col]`), which used to get the fallback rule. The other is the column delete, which it does flag.
  - T-3739's Context still says column inserts and deletes left of the table "keep the old rule". Since T-4680 they
    keep the text, by hand except for the in-table insert. T-3739's planned probe (F!A2:A8 `=Sales[@Qty]`, insert and
    delete a column left of the table) is the one that settles both extensions.
- Docs only. No engine change.

## Acceptance criteria
- [ ] references.md marks the outside-the-table and column-delete parts of the T-4680 rule as hand (unprobed) and
      points at T-3739.
- [ ] T-3739's Context describes the engine after T-4680.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The probe itself (T-3739).
