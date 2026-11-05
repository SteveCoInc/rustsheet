---
id: T-4401
title: "PIVOTBY subtotals on intransitive runs: row subtotals by reached class, and a class no record reached holds every record"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
PIVOTBY's row and column subtotals on an intransitive run hold what Excel's do, confirmed on a fresh hold-out:
a subtotal holds the line's records that reached its class, on either axis, and every record of the line when no
record reached its class.

## Context
- From T-4382 (probes/t4382-pivotby-intransitive-paths, excel-graph, 2026-11-05; docs/excel-semantics/coercion.md
  "PIVOTBY's other paths on intransitive runs (T-4382)"). Texts A = ab, B = ab&U+00AD, C = U+00AD&ab.
- Row subtotals: `Grouping::line_parts` (grouping.rs, used by `grid_rows` in pivotby.rs) gives a subtotal row the
  union of its groups' records; Excel's holds the records that reached the row class, those that reached no leaf
  under it included: over Bx Cx Ax By Cy (q p p q q) the `ab` subtotal row's q cell is `5` (row 5, Cy, reached `ab`
  and no y leaf), ours `""`. docs/research/t4382/score.py gives every row-subtotal cell of the probe (rows 16-30, B
  and C, ascending and `row_sort_order` -1) with that rule.
- Level-2 column subtotals (three column fields, `col_total_depth` 3): a subtotal whose class no record reached
  holds every record of the line, as a leaf no record reaches does, where ours is `""`: over BxC BxB CyA AxB AxB
  (p q p q p) the (`ab`&U+00AD,y) subtotal is p `1, 3, 5`, q `2, 4`. `Grid::cell_rows`'s `Slot::Subtotal` arm.
- The rule was chosen on these cells, so it owes a hold-out probe (predictions committed before the fill), as
  T-4378 did; docs/research/t4382/gen_probe.py and predict_engine.py are the pattern.
- Cells: probes/t4382-pivotby-intransitive-paths Calc!B16, B17, B21, B22, B23, B25, B29, B30, C17, C20, C22, C23,
  C25, C26, C28, C29 (row subtotals) and B31, B32, B33, B35, B39, B41, C31, C33, C39 (level-2 subtotals).

## Acceptance criteria
- [ ] Every probes/t4382-pivotby-intransitive-paths cell named in Context passes (or the cells the hold-out
      refutes are recorded in coercion.md and owned).
- [ ] A hold-out probe of fresh seeded runs (row subtotals both orders, three column fields), written with the
      engine's predictions committed before the fill, passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
