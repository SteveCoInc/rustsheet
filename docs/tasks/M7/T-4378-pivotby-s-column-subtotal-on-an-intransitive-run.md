---
id: T-4378
title: "PIVOTBY's column subtotal on an intransitive run holds the records that reached its class"
milestone: M7
lane: engine
status: done
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

## Outcome
- Rule: a PIVOTBY column subtotal cell holds the row line's records that reached its class: `Tree::descend` now
  returns the class ids a row reached (all `fields` of them for a leaf, fewer where a search found a class not `=`
  to it), `Grouping::reached_classes` keeps them per source row, and `Grid::cell_rows`' `Slot::Subtotal` arm takes
  the records whose reached classes start with the subtotal's prefix (`Grouping::prefix`). While `=` is transitive
  every row reaches a leaf, so this is the union of the subtotal's leaves as before. It replaces T-4358's interim
  `dedup` of the merged buckets.
- Chosen-on cells: the 19 owned cells of probes/t4358-pivotby-unreached-column-leaf (D1:D4, D8:D10, D12:D19, D21,
  D22, D24, D25) now pass; the case is 473 of 475 (B6, D6 stay T-4379's).
- Hold-out (decision 0188 R4, a structural choice): probes/t4378-pivotby-subtotal-holdout (new, 25 fresh seeded runs,
  75 formulas), written with the engine's predictions committed before the fill (docs/research/t4378/gen_probe.py,
  predict_engine.py, predictions.tsv; commit before the oracle sidecar). 75 of 75 predictions match Excel.
- Eval delta: probes fixed 19 (t4358 D cells), known failures 4814 -> 4795, 0 regressions; the new case t4378-pivotby-subtotal-holdout passes all its cells.

