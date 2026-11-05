---
id: T-4403
title: "PIVOTBY's moved subtotal cells over several value columns name a column group by its first row, which a leaf no record reaches shares"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PIVOTBY's subtotal columns of one function over several value columns show Excel's cells when the column tree
has a leaf no record reaches.

## Context
- From T-4382 (probes/t4382-pivotby-intransitive-paths, excel-graph, 2026-11-05; coercion.md "PIVOTBY's other
  paths on intransitive runs (T-4382)").
- `Placement` (pivot_placement.rs) names each column group across the sorts by its first row (`first_row`). A
  leaf no record reaches holds every record, so its first row is the first record's, which a reached group also
  has: the `now` map then probably sends the reached group's lines to the wrong index, and the moved subtotal cell
  (`Shown::Moved`) is looked up on the wrong line. Over Ay Ay Bx Cx Cy (q p p q q), two value columns, the `ab`
  subtotal column shows p `12`, q `11, 14, 15` in Excel (the reached-class records' second value column), ours
  `""` (Calc!B2).
- The subtotal error gate itself matched Excel on all 15 gate blocks (Calc!B1:B15 apart from these three): a
  subtotal whose records reached its class while every leaf under it is empty for the row group is not computed
  (no `#VALUE!`), as `Grid::has_records` decides.
- Cells: probes/t4382-pivotby-intransitive-paths Calc!B2, B5, B14.

## Acceptance criteria
- [x] probes/t4382-pivotby-intransitive-paths Calc!B2, B5 and B14 pass; a unit test covers a column tree with
      a leaf no record reaches under one function over two value columns.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `Placement` (pivot_placement.rs) names each column group across the sorts by `Group::leaf_id` (grouping.rs),
  the id of its leaf class in the grouping tree, instead of its first row, which a leaf no record reaches (it
  holds every record) shares with a reached group. The moved subtotal cells of one function over several value
  columns are then looked up on their own line.
- Unit test `moved_subtotal_cell_beside_a_leaf_no_record_reaches` (test_fn_pivotby.rs) is Calc!B2's run.
- coercion.md's T-4382 gate bullet lists the gate cells and records the fix.
- Eval delta: probes known failures 4964 -> 4961 (fixed probes/t4382-pivotby-intransitive-paths Calc!B2, B5,
  B14), 0 regressions.
