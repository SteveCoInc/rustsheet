---
id: T-4401
title: "PIVOTBY column subtotals on intransitive runs: a class no record reached holds every record of the line"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PIVOTBY's column subtotals on an intransitive run hold what Excel's do, confirmed on a fresh hold-out: a column
subtotal whose class no record reached holds every record of the line (level 2 and deeper; T-4378's rule already
gives the reached classes).

## Context
- From T-4382 (probes/t4382-pivotby-intransitive-paths, excel-graph, 2026-11-05; docs/excel-semantics/coercion.md
  "PIVOTBY's other paths on intransitive runs (T-4382)"). Texts A = ab, B = ab&U+00AD, C = U+00AD&ab.
- Row subtotals already follow the rule since T-4359 (#3409, GROUPBY's subtotal lines): every row-subtotal cell
  of the probe passes on main (Calc!B16:C30).
- Level-2 column subtotals (three column fields, `col_total_depth` 3): a subtotal whose class no record reached
  holds every record of the line, as a leaf no record reaches does, where ours is `""`: over BxC BxB CyA AxB AxB
  (p q p q p) the (`ab`&U+00AD,y) subtotal is p `1, 3, 5`, q `2, 4`. `Grid::cell_rows`'s `Slot::Subtotal` arm.
- The rule was chosen on these cells, so it owes a hold-out probe (predictions committed before the fill), as
  T-4378 did; docs/research/t4382/gen_probe.py and predict_engine.py are the pattern.
- Cells: probes/t4382-pivotby-intransitive-paths Calc!B31, B32, B33, B35, B39, B41, C31, C33, C39 (level-2
  subtotals).

## Acceptance criteria
- [ ] Every probes/t4382-pivotby-intransitive-paths cell named in Context passes (or the cells the hold-out
      refutes are recorded in coercion.md and owned).
- [ ] A hold-out probe of fresh seeded runs (three and four column fields, both orders), written with the
      engine's predictions committed before the fill, passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Rule: a PIVOTBY column subtotal whose class no record reached holds every record of the row line, as a column
  leaf no record reaches does. `Grouping::subtotal_reached` (grouping.rs) reads it from `reached_by`, the same
  per-class record of the descent that `Grouping::class_rows` uses for GROUPBY's subtotal lines (T-4359), through a
  shared `subtotal_class` that `subtotal_rows` now uses too; `Grid::cell_rows`' `Slot::Subtotal` arm returns the
  line's records for such a subtotal and keeps T-4378's reached-class filter otherwise. The error gate
  (`Grid::has_records`) is unchanged: the leaves under such a subtotal are unreached too and hold the records.
- Chosen-on cells: probes/t4382-pivotby-intransitive-paths Calc!B31, B32, B33, B35, B39, B41, C31, C33, C39 now
  pass (777 -> 786 of 819; the rest are T-4402's and T-4403's).
- Hold-out (decision 0188 R4, a structural choice): probes/t4401-pivotby-level-subtotal-holdout (new, seed
  "T-4401", 16 runs with three column fields and 14 with four, `col_total_depth` 3 and 4, ascending and
  `col_sort_order` -1; 60 formulas). The engine's predictions were committed before the fill (docs/research/t4401,
  commit 89c6e5d); 21 of the 60 differ from main's engine. Filled from excel-graph: 60 of 60 predictions match
  (730 of 730 cells).
- coercion.md: the T-4382 entry's level-2 bullet records the hold-out.
- Eval delta: probes fixed 9, known failures 4914 -> 4905, 0 regressions; the new case passes all its cells.
