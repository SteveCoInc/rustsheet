---
id: T-3629
title: "Probe a detached #This Row after convert to range: a re-created same-name table, and its xlsx save and reload"
milestone: M8
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: [T-2763]
---

## Goal
Excel evidence for two choices T-2763 made without it, for a `#This Row` reference left off the data rows by a
convert to range (FORMULATEXT `=Sales[@Qty]`, value `#VALUE!`; `StructuredReference::detached`):
(a) whether a table later created with the same name (`Sales`, over the same area or another) takes the reference
back (value and FORMULATEXT of the off-row cells afterwards); the engine says it does not;
(b) what Excel writes to the cell's `<f>` when it saves such a workbook, and what the cell reads after a reload; the
engine's xlsx export writes `#VALUE!`.

## Context
- T-2763 code: `StructuredReference::detached` (parser/structured_reference.rs), the export arm of
  `reference_text` (parser/stringify/structured.rs), references.md convert-to-range entry.
- (a) needs a sheet op that adds a table after the convert: the oracle's sheet ops (tools/src/rustsheet_tools/
  sheetop.py) have `convert_table` and `delete_table` but no `add_table`; Graph has `tables/add`. Adding the op
  is part of this task.
- (b) needs a save of a workbook edited through Graph and the downloaded file's `<f>`; if excel-graph cannot give
  it, it is a desktop question (M14, decision 0039).

## Acceptance criteria
- [x] Probe cases with oracle values for (a), and for (b) where the oracle can answer (else an M14 note).
- [x] The engine matches them, or follow-up tasks are filed; the `TODO(T-3629)` markers are resolved;
  findings recorded in docs/excel-semantics/references.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other structural edits of a detached reference.

## Outcome
- Harness: a sheet op `{"op": "add_table", "sheet", "table", "ref"}` (recorded as `<table> <range>`): the
  excel-graph backend adds the table with Graph `tables/add`, renames it and reads the name back; the engine
  adapter uses `Txn::add_table` (`SheetOp::AddTable`). Tool and adapter tests.
- Evidence (excel-graph 2026-11-04, 5 probes/table-edits/t3629-* cases, verified against Excel's saved files):
  after `convertToRange`, an off-row `#This Row` (F1, F9) is `#REF!` with FORMULATEXT `=#REF!`, and Excel's save
  writes `<f>#REF!</f>` (answers (b): a reload reads `#REF!`); a data-row one is lowered to `$C5`. A table re-added
  as Sales over the same area or over K8:L11 does not take F1 or F9 back (answers (a)).
- The engine keeps them detached (`#VALUE!`, `=Sales[@Qty]`, export `#VALUE!`), so 20 cells (F1, F2, F9, F10 of
  each case) fail and are baselined as new-coverage known failures owned by T-4020; the TODO(T-3629) markers now
  cite the evidence and point at T-4020. references.md convert-to-range entry records it all.
- T-2763's evidence (table-convert-names-this-row) is doubtful: with a `#This Row` of Sales on another sheet,
  Graph's `convertToRange` answered success but left the table in place (seen in the saved files; those three
  runs were dropped). Re-running that case crashed Excel's recalculation twice; its sidecar was kept. T-4021
  makes the replay verify table ops.
- Eval delta: probes +5 cases, known failures 4433 -> 4453 (20 new-coverage, owned by T-4020), regressions 0.
- Follow-ups: T-4020 (engine: `#REF!` at convert), T-4021 (harness: verify table ops took effect).
