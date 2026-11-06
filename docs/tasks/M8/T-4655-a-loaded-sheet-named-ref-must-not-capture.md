---
id: T-4655
title: "A loaded sheet named *#REF! must not capture deleted-sheet references; confirm the saved <f> of a lost reference"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2612]
---

## Goal
The internal sentinel that marks a reference to a deleted sheet cannot collide with a sheet name a workbook brings in.
The `<f>` text the engine saves for such a reference is the one Excel writes, backed by an Excel-saved file.

## Context
- Review MINORs on #3653 (T-2612, round 3: security and excel-fidelity). They change no evidenced behaviour.
- T-2612 rewrites each reference to a deleted sheet into a wrong reference to `DELETED_SHEET = "*#REF!"`
  (crates/rustsheet-engine/src/expressions/parser/stringify/wrong_reference.rs). `is_missing_sheet` prints that
  reference as `#REF!`. Only `*` keeps the name apart from real sheets: `is_valid_sheet_name` refuses it on add and
  rename. Neither reviewer saw the xlsx loader enforce it.
  - A crafted workbook with a sheet named `*#REF!` could let lost references bind to that sheet.
  - A loaded `'*#REF!'!A1` would display and export as `#REF!A1`.
  - No panic either way, and Excel cannot make such a file.
- The export test `crates/rustsheet-xlsx/tests/deleted_sheet_reference_export.rs` checks
  `<f>SUM(#REF!C4:C7)+#REF!$A$1</f>`. That text is inferred from Excel's FORMULATEXT
  (probes/table-edits/t2612-delete-sheet-then-add), not read from a file Excel saved (references.md
  "A deleted table or sheet").
- T-4628 covers loading a `#REF!C4:C7` back. This task covers the save side and the name collision.

## Acceptance criteria
- [ ] Loading a workbook with a sheet named `*#REF!` (or any name `is_valid_sheet_name` refuses) cannot make
      references to a deleted sheet resolve to it. Choose one fix and test it: refuse or rename the sheet on load
      (record which, with a `DEVIATION` entry if Excel's behaviour on such a file is unknown), or use a sentinel the
      R1C1 lexer cannot produce from a sheet name.
- [ ] The saved `<f>` of a reference lost by delete-then-add is confirmed against a workbook Excel saved, using the
      `excel-probe` skill and an oracle-saved file. Otherwise the inferred note in references.md stays and the
      Outcome says why.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Loading `#REF!C4:C7` back (T-4628).
