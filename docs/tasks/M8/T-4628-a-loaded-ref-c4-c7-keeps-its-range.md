---
id: T-4628
title: "A loaded #REF!C4:C7 keeps its range: FORMULATEXT after an xlsx round trip of a deleted-sheet reference (probe first)"
milestone: M8
lane: engine
status: in-progress
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
- [ ] A probe (excel-graph) of a loaded `=SUM(#REF!C4:C7)`, `=#REF!$A$1` and `=ISREF(#REF!A1)`: value and
      FORMULATEXT.
- [ ] If Excel keeps the range, the lexer reads `#REF!<range>` as a wrong reference to `DELETED_SHEET`, so load,
      display and export agree; otherwise the semantics entry records that Excel drops it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- References to a sheet that never existed (`Nope!A1`).
