---
id: T-4702
title: "Undo of a row or column delete puts back conditional formats and defined names it rewrote"
milestone: M8
lane: engine
status: todo
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
- [ ] Undo of a column and of a row delete restores a conditional format whose range or formula the delete changed
      past what the re-insert undoes; redo changes it again. Unit tests.
- [ ] Defined names checked: either they are not rewritten by a delete (Excel evidence recorded in
      docs/excel-semantics/), or the undo restores them too.
- [ ] A peer's undo diff with a bad conditional format or name is refused and changes nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cell formulas (T-4668) and tables (T-3634, T-4652).
