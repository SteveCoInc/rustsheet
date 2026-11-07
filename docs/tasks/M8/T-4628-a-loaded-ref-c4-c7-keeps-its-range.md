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
- Engine: in typed text only, the lexer no longer swallows the cell or range after `#REF!`, so a typed `#REF!C4:C7`
  is refused, as in Excel. Every other text still reads it as `#REF!` (`consume_deleted_sheet_reference`, skipped when
  the lexer is `typed`): a fill or paste re-entering the display (review round 1 found autofill giving `#ERROR!`;
  test `fill_and_paste_of_a_deleted_sheet_reference_stay_ref`), `set_user_input`, and a loaded file (review round 2:
  no evidence for changing load; desktop Excel's file form is an M14 question). The xlsx export writes a reference to a deleted sheet as
  the bare `#REF!` (`is_exported_deleted_sheet`, wrong_reference.rs) instead of `#REF!C4:C7`, which Excel for the web
  refused to load; the display still prints `#REF!C4:C7` as Excel's FORMULATEXT does. Tests updated
  (`a_typed_deleted_sheet_reference_does_not_parse`, deleted_sheet_reference_export.rs). The refused workbooks
  were built with spec2xlsx from one-cell specs (`=SUM(#REF!C4:C7)`, `=#REF!$A$1`, `=ISREF(#REF!A1)`, `=SUM(#REF!A:A)`,
  `=SUM(#REF!1:2)`, `=#REF!A1`, each with `=FORMULATEXT(A1)` in A2; and `legacy_formulas: ["A1"]` for the legacy form)
  and `rustsheet-oracle recalc -b excel-graph`. A defined name over the
  deleted sheet keeps main's stored `#REF!$C$4` (untyped text reads it back as `#REF!`; review round 3 dropped an
  unevidenced change to the bare `#REF!`); a stored `<definedName>` of that form is unprobed (T-4794).
- references.md "A deleted table or sheet" records it; desktop Excel's own file form is an M14 question (T-4794).
- Follow-ups filed: T-4757 (T-4446 review MINOR: sheet names with a later mark, INDIRECT and Mc); T-4794 (desktop
  Excel's file form of a deleted-sheet reference, cell and defined name, M14).
- Eval delta: +9 passing cells (new case); no regressions.

## Review
Resumed after the review-round limit (step 5c).
- Rounds 1-3 (heads 2b41b538, c1e38624, 8485758bc): security and design approved each round. Excel fidelity requested
  changes three times:
  - autofill/paste gave `#ERROR!`;
  - loaded-file text was changed without evidence;
  - an unevidenced bare-`#REF!` rewrite of defined names.
  Each was fixed (see Outcome).
- Step 5c re-review on head a7e3eb4af (2026-11-07, full panel): security, design and excel-fidelity APPROVE. The
  round-3 design MINOR is settled on that head: the lexer's `double_star_exponent` and `typed` were one concept and
  are now the single `typed` flag. The remaining MINORs (a design note on one doc line; a fidelity wish to commit
  the one-cell specs of the refused workbooks for re-checking, which T-4794 can carry) are noted on PR #3823.
