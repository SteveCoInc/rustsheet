---
id: T-4702
title: "Undo of a row or column delete puts back conditional formats and defined names it rewrote"
milestone: M8
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Undo of a row or column delete leaves the conditional formats and defined names exactly as they were before it.

## Context
- Found by T-4668 (2026-11-06), which made the undo put back the cell formulas a delete rewrote past what re-inserting
  the lines undoes (`Model::lost_on_reinsert`, `RewrittenCell` in `formula_rewrite.rs`, carried in `Diff::DeleteRows`
  and `Diff::DeleteColumns`). It did not cover the other formulas a delete rewrites:
  - the conditional formats of the edited sheet: `Model::displace_cf_ranges` shrinks or drops a rule's range and
    rewrites its formulas (a rule over D1:D9 alone, or `=$D1>0`, after a delete of column D), and the undo's
    re-insert cannot put either back;
  - defined names: check first whether a row or column delete rewrites them at all (`displace_formulas` does not walk
    `workbook.defined_names` today); if it does not, record what Excel does (excel-probe: `=Sheet1!$D$1` named, delete
    column D, `FORMULATEXT`/Name Manager) before changing anything.
- A peer's undo diff is untrusted (CLAUDE.md directive 5): check what it restores before the undo changes anything,
  as `UserModel::checked_rewritten_cells` does.

## Acceptance criteria
- [x] Undo of a column and of a row delete restores a conditional format whose range or formula the delete changed
      past what the re-insert undoes; redo changes it again. Unit tests.
- [x] Defined names checked: either they are not rewritten by a delete (Excel evidence recorded in
      docs/excel-semantics/), or the undo restores them too.
- [x] A peer's undo diff with a bad conditional format or name is refused and changes nothing.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cell formulas (T-4668) and tables (T-3634, T-4652).

## Outcome
- **Names were not rewritten at all, but Excel rewrites them.** Before this change, a row or column delete or insert
  left every defined name alone. Excel rewrites names as it rewrites cell references. Six excel-graph probes show it,
  with predictions committed before each fill (probes/sheet-ops/t4702-*, 2026-11-07, 37 cells):
  - a name moves with its cell;
  - a range that loses an edge shrinks;
  - a formula name is rewritten too;
  - a name on a deleted cell is `#REF!`;
  - inserts shift names the same way.

  docs/excel-semantics/references.md, "Row and column inserts and deletes rewrite defined names", records the
  results. It also records the `NA1` probe trap: that name is a cell address, so the first probe's A1 and A5 read the
  cell, not the name.
- **The engine now rewrites names.** `Model::displace_defined_names` (formula_rewrite.rs), called from `displace_formulas`,
  rewrites every name for `DisplaceData::Row`/`Column` with the cells' displacement. A name the edit does not change
  keeps its stored text. Moves, relative references in names and other-sheet scopes stay unprobed and unchanged.
  `rewrite_defined_names` is shared with `rewrite_formulas` (sheet rename and delete). It parses names in English through
  `parse_internal_formula`. The two anchor names differently, which is an open question for T-4747:
  - A1 of the edited sheet here;
  - A1 of the first remaining sheet there.
- **Undo restores names and conditional formats.** `UserModel::rewrite_snapshot` and `rewritten_snapshot_diffs`
  (user_model/diff_undo.rs) record one `UpdateConditionalFormatting` per changed rule and one `UpdateDefinedName` per
  changed name. Each diff has `old == new` set to the state before the delete, as `SetMergedCells` does:
  - redo replays them as no-ops before the delete rewrites again;
  - undo writes them back after the lines are inserted again.
- **A peer's bad diff is refused.** `UpdateConditionalFormatting` used to skip an index past the end silently. It
  now goes through `checked_cf_slot` (diff_undo.rs), so an out-of-range index or a range that names no cell is
  refused and changes nothing. An unknown name was already refused by `update_defined_name`.
- **Tests:** crates/rustsheet-engine/src/test/user_model/test_delete_undo_rewrites.rs.
  - Undo and redo of a column delete and of a row delete, covering the formats, the names and a cell that reads them.
  - Refusal of a bad format index, a bad format range and an unknown name, on both undo and redo.
  - `cargo test -p rustsheet_engine`: 5,659 pass.
- **Follow-up:** T-4747 probes relative-reference and sheet-scoped names, moves, and the stored spelling of a rewritten
  name (from the excel-fidelity review).
- **Eval delta:** 6 new probe cases (37 cells), all passing; 0 regressions (`make eval-baseline`).
