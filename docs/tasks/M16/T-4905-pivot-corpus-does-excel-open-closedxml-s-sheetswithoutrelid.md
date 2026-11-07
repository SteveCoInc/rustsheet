---
id: T-4905
title: "pivot-corpus: does Excel open ClosedXML's SheetsWithoutRelId.xlsx, which the importer refuses (Missing relationship for worksheet)?"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence of whether Excel opens `pivot-corpus-closedxml/TryToLoad/SheetsWithoutRelId.xlsx`, so the
importer's refusal is either confirmed (an accepted malformed-input refusal) or becomes an engine task with
the values Excel shows.

## Context
- The only pivot-corpus case that does not run: `pivot-corpus-closedxml/TryToLoad/SheetsWithoutRelId.xlsx`
  ends in `engine_error` with `import failed: invalid workbook: Missing relationship "" for worksheet`
  (`crates/rustsheet-xlsx/src/import/util.rs`), so none of its cells is scored and its one pivot is not
  classified (T-4131).
- Its docProps say Microsoft Excel. ClosedXML keeps it in `TryToLoad/` as a file it must load: a `<sheet>`
  with an empty `r:id`.
- The same cause as poi/60825.xlsx, which T-0489 (M11) asks the same question for; one excel-graph run can
  answer both.

## Acceptance criteria
- [ ] docs/excel-semantics/xlsx-storage.md says what Excel for the web does with the file (opens, repairs or
      refuses it), with the date and backend; T-0489's entry is shared if it lands first.
- [ ] If Excel opens it, an engine task is filed (milestone M16, lane engine) with the values Excel shows;
      if it refuses it, the Outcome says the case stays an import failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the importer (the filed engine task does that).
- Desktop Excel's repair dialog (M14).
