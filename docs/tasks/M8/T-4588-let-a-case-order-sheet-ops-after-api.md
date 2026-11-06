---
id: T-4588
title: "Let a case order sheet_ops after api_names, then probe a sheet rename and delete after an API name bound to that sheet"
milestone: M8
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A probe case can apply a sheet edit after its API-added names, so we can learn what Excel does to a name bound to a
sheet when that sheet is later renamed or deleted.

## Context
- docs/architecture/eval-harness.md: `api_names` replay after `sheet_ops`, in both the excel-graph backend
  (`tools/src/rustsheet_tools/api_names.py`, the oracle's replay) and the engine adapter
  (crates/rustsheet-eval). No case can rename a sheet after a name was added.
- T-4554 needed this. docs/excel-semantics/syntax.md "An unqualified reference in an API-defined name binds to the
  active sheet" marks the rename and delete of the bound sheet UNVERIFIED. For example: with S2 active, add `W` = `=$B$1`,
  then rename S2 to S9, or delete it, and read `=W`.
- Ours stores the body qualified (`=S2!$B$1`), so a rename rewrites it and a delete makes it `#REF!`.

## Acceptance criteria
- [ ] One documented case key (for example a per-op `"after_names": true`, or an `api_names` entry position) orders a
      sheet op after the names. spec2xlsx, the excel-graph replay and the engine adapter honour it, with tests.
- [ ] A probe (excel-graph) adds `W` = `=$B$1` and `R` = `=$B$1:$B$2` with S2 active, then renames S2. A second
      probe deletes S2. Both cases cover uses on every sheet. syntax.md marks (b) with the result, and the engine
      matches or an owner task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other op orders.
