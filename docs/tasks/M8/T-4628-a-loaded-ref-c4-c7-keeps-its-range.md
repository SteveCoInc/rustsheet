---
id: T-4628
title: "A loaded #REF!C4:C7 keeps its range: FORMULATEXT after an xlsx round trip of a deleted-sheet reference (probe first)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A workbook whose file holds `SUM(#REF!C4:C7)` (Excel's form of a reference to a deleted sheet) loads with
FORMULATEXT and the export giving what Excel gives, the range included.

## Context
T-2612 makes a sheet delete store such a reference as a wrong reference to `DELETED_SHEET`, printed `#REF!C4:C7`
(references.md, "A deleted table or sheet", stringify/wrong_reference.rs). On load, though, the A1 lexer
(`consume_deleted_sheet_reference`, lexer/mod.rs) swallows the `C4:C7` after `#REF!` into a bare `#REF!` error, so
the engine's FORMULATEXT after a round trip is `=SUM(#REF!)` and the next save drops the range. Excel's
FORMULATEXT of the loaded file is unprobed: it may keep `=SUM(#REF!C4:C7)`.

## Acceptance criteria
- [x] A probe (excel-graph) of a loaded `=SUM(#REF!C4:C7)`, `=#REF!$A$1` and `=ISREF(#REF!A1)`: value and
      FORMULATEXT.
- [x] If Excel keeps the range, the lexer reads `#REF!<range>` as a wrong reference to `DELETED_SHEET`, so load,
      display and export agree; otherwise the semantics entry records that Excel drops it.
- [x] `make check` green; reviewers approve.

## Out of scope
- References to a sheet that never existed (`Nope!A1`).

## Outcome
- Excel keeps nothing of the range: it refuses every `#REF!<ref>` form. New probe probes/t4628-deleted-sheet-reference-text
  (excel-graph, 9 cells) types `=SUM(#REF!C4:C7)`, `=#REF!$A$1`, `=ISREF(#REF!A1)` and `=SUM(#REF!A:A)`: each is refused
  (blank cell, FORMULATEXT `#N/A`), while the bare `=SUM(#REF!)`, stored or typed, is `#REF!` with FORMULATEXT
  `=SUM(#REF!)`, and `ISREF(#REF!)` is FALSE. Stored in a file (hand-built workbooks, dynamic-array and legacy `<f>`),
  every `#REF!<ref>` form makes Excel for the web refuse the workbook (createSession 501), so they cannot be committed.
- Engine: the lexer no longer swallows the cell or range after `#REF!` (`consume_deleted_sheet_reference` removed), so
  that text is a parse error, refused when typed, as in Excel. The xlsx export writes a reference to a deleted sheet as
  the bare `#REF!` (`is_exported_deleted_sheet`, wrong_reference.rs) instead of `#REF!C4:C7`, which Excel for the web
  refused to load; the display still prints `#REF!C4:C7` as Excel's FORMULATEXT does. Tests updated
  (`a_displayed_deleted_sheet_reference_does_not_parse`, deleted_sheet_reference_export.rs). A defined name over the
  deleted sheet is now stored as the bare `#REF!` (`delete_sheet_in_name_node`), since its A1 text is parsed again.
- references.md "A deleted table or sheet" records it; desktop Excel's own file form is an M14 question.
- Follow-up filed: T-4757 (T-4446 review MINOR: sheet names with a later mark, INDIRECT and Mc).
- Eval delta: +9 passing cells (new case); no regressions.
