# SPEC-0036 — Covered cells of a merged range keep their content

- **Status:** accepted (2026-11-07, T-4975 item 1)
- **Milestone / lane:** M11 / engine
- **Tasks:** T-4722 (the spec it asked for); landed with T-4975 (docs/research/excel-parity-findings-2026-11-07.md
  item 1). Probe follow-ups: the UNVERIFIED rows below.

## Problem
A workbook can store values and formulas in the covered (non-anchor) cells of a `<mergeCell>`. Excel keeps them:
formulas read them, a covered formula is recalculated and a save writes them back. The importer emptied every covered
cell (`clear_covered_cells`, crates/rustsheet-xlsx/src/import/merged_cells.rs) to keep the engine invariant "covered
cells hold no content". That made fixtures/merged-covered-values A4:N4 and fixtures/merged-covered-formula B1, A3:D3
visibly off (14 of 19 checked cells failing before this spec), kept COUNT, COUNTA, COUNTBLANK, ISBLANK, ISTEXT,
ISFORMULA and LEN just under the Gate B rate, and lost covered values on a round trip (poi/bug66675.xlsx 3,604 cells,
poi/60512.xlsm 12; docs/research/xlsx-round-trip-2026-11-04.md §2.2).

## Goals / non-goals
- Goal: covered content survives import, evaluation, export and the user-model operations below, as in Excel.
- Non-goal: Excel's UI merge of cells that hold content (it keeps only the top-left value; a Format Painter merge keeps
  them all, unprobed). Our merge keeps refusing a range with more than one content cell and clears what it covers.

## Excel behavior to match
Evidence: docs/excel-semantics/xlsx-storage.md "A merged range keeps the values of its covered cells" (excel-graph,
2026-11-06, T-4148; fixtures/merged-covered-values, fixtures/merged-covered-formula).

| Operation | Rule | Evidence |
|---|---|---|
| Import | A covered cell keeps its value, formula and style. | VERIFIED: both fixtures |
| Evaluation | Formulas read a covered cell (`=B1` "x", `=COUNTA(A1:D1)` 4, `=ISBLANK(B1)` FALSE); a covered formula is recalculated and ISFORMULA is TRUE. A value-less `<c t="str"/>` is blank. | VERIFIED: both fixtures |
| Export | A covered cell is written back as it is. | VERIFIED (the file-level form is Excel's own storage); tested by `covered_cells_calculate_and_survive_a_round_trip` |
| Edit of a covered cell | Refused, as before: only the anchor can be selected and edited. | Excel's UI cannot select a covered cell; no probe needed |
| Merge | Refused when more than one cell of the range holds content; otherwise the content moves to the anchor and the covered cells are cleared. Unchanged. | Ours; Excel's UI merge keeps only the top-left value (reported, UNVERIFIED) |
| Unmerge | The covered content shows again, unchanged. | UNVERIFIED: probe candidate |
| Clear (Delete) over the merged range | Clears every cell of the range, covered ones included. | UNVERIFIED: probe candidate |
| Copy-paste, cut-paste of a merged block | The pasted block carries the covered content (a formula's references shifted as for any pasted cell, or moved for a cut); the recreated merge keeps it and clears only cells it swallows outside the pasted block. | UNVERIFIED: probe candidate |
| Autofill of a merged block | A covered source that holds content is copied to its tiled target (a formula shifted), never as a progression term; an empty covered source clears its target. | UNVERIFIED: probe candidate |
| Ctrl+arrow navigation | Unchanged: it moves by cells, and a covered cell snaps to its anchor. Whether Excel stops on a covered value is not known. | UNVERIFIED: probe candidate |

The UNVERIFIED rows are what a desktop probe of Excel's UI would settle (the excel-graph oracle evaluates workbooks; it
does not drive these operations). docs/research/excel-parity-findings-2026-11-07.md "Suggested probes" lists them.

## Design
- **Import** (rustsheet-xlsx `import/worksheets.rs` `load_sheet`): no clearing. `clear_covered_cells` is gone; the
  intersecting-range index (`KeptRanges`, T-3454) is unchanged.
- **Evaluation and export:** no change. They never relied on the invariant.
- **Merge** (`Model::merge_cells`): unchanged; it calls `merge_cells_keep_styles(range, None)`.
- **Paste** (`user_model/clipboard.rs`): copy- and cut-paste share `recreate_pasted_merge(merge, pasted, source_range,
  diffs)`. It calls `covered_cells_clear_diffs(range, Some(pasted))` and `Model::merge_cells_keep_styles(range,
  Some(pasted))`: covered cells inside the pasted block keep what the paste wrote and get no clear diff. Before this,
  both paths cleared them, so a cut-paste dropped covered content silently.
- **Autofill** (`user_model/autofill.rs` `fill_covered_cell`): a non-empty covered source goes through
  `Model::fill_cell` with a `SetCellValue` diff; an empty one still clears the target. Covered cells stay out of the
  progression's terms.
- **Undo/redo:** the existing diff order holds. A paste or fill pushes its merge-removal diff first, then the cell
  diffs, then the merge-creation diff, so a forward replay writes on unmerged cells and an undo unmerges first.
- `MergedCell`'s doc comment (engine `types.rs`) no longer states the invariant.

## Alternatives considered
- Keep clearing on import and record a deviation: rejected; Excel's behavior is VERIFIED and directive 1 has no
  exception for it.
- Clear covered content on paste and autofill (only import keeps it): rejected; the pasted block would lose what the
  copy carried, which no evidence supports either way, and the simpler rule is one invariant fewer.

## Test plan
- xlsx: `covered_cells_keep_their_values_and_styles_on_import`, `covered_cells_calculate_and_survive_a_round_trip`,
  `many_whole_column_merges_import_quickly_and_keep_covered_cells` (the 10 s bound kept),
  `test_import_sanitizes_merged_cells` (covered C2 keeps 43).
- engine (`test/user_model/test_merged_covered_content.rs`): reads and refused edits, unmerge, clear, copy-paste,
  cut-paste, a pasted merge clearing outside its block, autofill with and without covered content; each with undo
  and redo.
- eval: fixtures/merged-covered-values and fixtures/merged-covered-formula pass; the round-trip suites keep covered
  cells.

## Rollout
Lands as one change (T-4975 item 1). Follow-ups: a probe task per UNVERIFIED row when a desktop UI probe exists.
