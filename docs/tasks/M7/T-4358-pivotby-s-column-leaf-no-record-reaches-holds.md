---
id: T-4358
title: "PIVOTBY's column leaf no record reaches holds every record of its row group"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4344]
---

## Goal
PIVOTBY matches Excel on an intransitive run in its column fields: the cells of a column leaf that no record of
the row group reaches, which Excel fills with every record of that row group.

## Context
- probes/t4344-groupby-descending-search Calc!B31:E42 (T-4344; docs/excel-semantics/coercion.md, "GROUPBY on an
  intransitive run"): rows p/q (Data!F), columns the text and x/y (Data!A:B), A = ab, B = ab&U+00AD,
  C = U+00AD&ab. Engine 44 of 48; the four misses:
  - D39 (Cx, Cx, Bx, Ay, Bx; columns (C,x), (C,y), (C,x)): the second (C,x) holds `2` for p and `1, 3, 4, 5`
    for q, every record of each row group; ours is empty (`Grouping::group_of` maps each record to one column).
  - D41 and E41 (Ax, Bx, Bx, Cy, Ax): the (B,y) column holds `1, 2, 3` for p and `4, 5` for q. Row 4 (C,y), the
    only q record with y, reaches (B,y) in the whole-source column tree, yet q's cell is all of q, as if no q record
    reached it.
  - E38 (Ax, Cy, Cy, By, Bx, col_sort_order -1): the (A,y) column holds p `1, 2, 4, 5` (all of p) but q `3`. Rows
    2 and 3 have the same key (C,y), so whether a record reaches a column leaf depends on its row group.
- A model to score first: each cell is a leaf of one tree over the row fields then the column fields (GROUPBY's
  tree, `Grouping::new`, on the concatenated key), with a column leaf no record of its row group reaches holding
  that row group's records. The column headers stay the column grouping's.
- PIVOTBY's relative sets (`relative_to`) on such runs are unprobed.
- Owned cells: probes/t4344-groupby-descending-search (4): Calc!D39, Calc!D41, Calc!E38, Calc!E41.

## Acceptance criteria
- [ ] The model is scored on the four cells and the 44 that pass, with a hold-out probe (predictions committed
      before the fill) if it is chosen on them; the cells pass or stay known failures with the reason recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY's own search (T-4344) and value sorts (T-4359).
