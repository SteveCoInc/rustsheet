---
id: T-5044
title: "xlsx import keeps the content of covered cells of a merged range (SPEC-0036 S1)"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An xlsx file's covered cells of a merged range load with their stored values and formulas, calculate, and survive a round trip, as Excel keeps them.

## Context
Spec: docs/specs/SPEC-0036-merged-covered-content.md, rows I1, W1, U3, U4, U7 (rollout S1). Evidence:
docs/excel-semantics/xlsx-storage.md "A merged range keeps the values of its covered cells" (fixtures/merged-covered-values,
fixtures/merged-covered-formula, excel-graph). Code: `clear_covered_cells` (crates/rustsheet-xlsx/src/import/merged_cells.rs),
called from `load_sheet` (import/worksheets.rs); keep `load_merge_cells` and `KeptRanges`. Evaluation and export need no change
(SPEC-0036 E1, W1). Update `test_import_sanitizes_merged_cells` (tests/test.rs) and
`many_whole_column_merges_import_quickly_and_clear_covered_cells` (tests/malformed_import.rs) per the spec's test plan, and the
xlsx-storage.md entry's "Ours" and Status lines.

## Acceptance criteria
- [x] The importer keeps covered content; `clear_covered_cells` and its unit tests are gone.
- [x] This task owns these visibly-off cells, and they pass:
      fixtures/merged-covered-values.xlsx A4:N4
      fixtures/merged-covered-formula.xlsx B1 A3:D3
- [x] Tests: import keeps covered values and styles; a covered formula calculates and round-trips; unmerge shows the covered
      value; a range clear of the merged range clears it; deleting the anchor row makes the covered cell the anchor with its
      value (each engine test with undo/redo).
- [x] `make check` green; reviewers approve.

## Out of scope
- Paste (T-5045) and autofill (T-5047) over covered content.

## Outcome
The importer keeps covered cells as stored; `clear_covered_cells` and its unit tests are gone (`KeptRanges` stays).
fixtures/merged-covered-values A4:N4 and merged-covered-formula B1 A3:D3 now pass: 15 known-failure lines left
eval/baseline/fixtures.txt. The roundtrip-fixtures baseline counts two more checked cells for merged-covered-formula
(117 to 119), as kept covered cells are now compared. Tests: xlsx import keeps covered content (bad_merge_cells,
whole-column merge bench), covered content round-trips, and engine tests for covered values in formulas, unmerge,
range clear and delete of the anchor row, each with undo/redo. No new Excel quirk beyond xlsx-storage.md's entry,
whose Ours/Status lines are updated.
