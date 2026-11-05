---
id: T-4378
title: "PIVOTBY's column subtotal on an intransitive run holds the records that reached its class"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4358]
---

## Goal
PIVOTBY's column subtotals match Excel on an intransitive run in the column fields: a first-field subtotal column
holds the row group's records whose descent reached that class, including records that reached no leaf under it,
and not the extra records a leaf no record reaches holds.

## Context
- Found by T-4358's hold-out, probes/t4358-pivotby-unreached-column-leaf column D (`col_total_depth` 2): every
  leaf cell matches the engine, every subtotal cell its records (the engine unions its leaves' buckets). Over Bx Cy
  Cy Ax By of q q q p p, the `ab` subtotal of q is `2, 3` (rows 2 and 3 reach class A, as C = A, then no y leaf)
  though its one leaf (A,x) holds nothing for q (D1); over Ax Cx Bx Cy Bx the `ab`&U+00AD subtotal of p is `3, 5`
  while its unreached leaf (B,y) holds `1, 3, 5` (D10).
- The candidate (docs/research/t4358/score.py) gives 88 of 88 subtotal cells of the 25 blocks. It was chosen on
  them, so it owes a hold-out probe with predictions committed before the fill (decision 0188 R4).
- The same notion, the records that reached a parent but no leaf, decides a value sort's parent values (T-4359);
  one per-row "deepest class reached" from `Tree::descend` should serve both (directive 3).
- Code: `Grid::cell_rows`' `Slot::Subtotal` arm (functions/lookup_and_reference/pivotby.rs), `Tree::descend`
  (grouping/tree.rs).
- Owned cells: probes/t4358-pivotby-unreached-column-leaf (19): Calc!D1, Calc!D2, Calc!D3, Calc!D4, Calc!D8, Calc!D9, Calc!D10, Calc!D12, Calc!D13, Calc!D14, Calc!D15, Calc!D16, Calc!D17, Calc!D18, Calc!D19, Calc!D21, Calc!D22, Calc!D24, Calc!D25.

## Acceptance criteria
- [ ] A subtotal column's cell holds the records that reached its class; the owned cells pass, with a hold-out
      probe of fresh runs (predictions committed before the fill) scored in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
