---
id: T-4794
title: "Probe how desktop Excel stores a reference to a deleted sheet in its file (cell and defined name)"
milestone: M14
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/references.md "A deleted table or sheet" says what desktop Excel writes into its own xlsx for a
reference to a deleted sheet, in a cell formula and in a defined name, and whether desktop Excel loads a file that
stores `#REF!C4:C7`, `#REF!$A$1` or a `<definedName>` of `#REF!$C$4`. The engine's export and load follow it, or
the note says why not.

## Context
- T-4628 (PR #3823) found that Excel for the web refuses a workbook storing any `#REF!<ref>` cell form and loads the
  bare `SUM(#REF!)` (probes/t4628-deleted-sheet-reference-text, hand-built workbooks, excel-graph 2026-11-07). So the
  engine's export writes a reference to a deleted sheet as the bare `#REF!` (`is_exported_deleted_sheet`,
  stringify/wrong_reference.rs), and the range is lost from the file. A loaded `#REF!C4:C7` still reads as `#REF!`
  (`consume_deleted_sheet_reference`, lexer). Only cell `<f>` forms were probed: a defined name over a deleted sheet
  is stored as `#REF!$C$4` and exported as that text (T-4628 review round 3).
- What desktop Excel itself writes (it is the file author) and whether it loads these forms needs excel-desktop, so
  this is M14 (decision 0039). `rustsheet-oracle status` shows excel-desktop n/a today.

## Acceptance criteria
- [ ] A desktop-Excel-saved workbook with `=SUM(S!C4:C7)`, `=S!$A$1` and a name `nm` = `S!$C$4`, after deleting S,
      is inspected (`rustsheet-eval snapshot`): its `<f>` and `<definedName>` texts are recorded in references.md.
- [ ] Desktop Excel's load of hand-built files storing `#REF!C4:C7` (cell) and `#REF!$C$4` (definedName) is recorded.
- [ ] If the engine's export or load differs, a follow-up task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The typed-text refusal (done in T-4628).
- Excel for the web, already probed in T-4628.
