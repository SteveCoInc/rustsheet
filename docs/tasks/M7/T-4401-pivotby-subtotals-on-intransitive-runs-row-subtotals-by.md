---
id: T-4401
title: "PIVOTBY column subtotals on intransitive runs: a class no record reached holds every record of the line"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4388]
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
- Owned until PR #3430 (T-4388, the rule) lands, which passes them: probes/t4401-pivotby-unreached-subtotal-holdout
  Calc!B2, Calc!B3, Calc!B4, Calc!B5, Calc!B6, Calc!B7, Calc!B8, Calc!B9, Calc!B10, Calc!B12, Calc!B13, Calc!B14, Calc!B17, Calc!B18, Calc!B20, Calc!B21, Calc!B22, Calc!B23, Calc!C1, Calc!C3, Calc!C5, Calc!C9, Calc!C11, Calc!C13, Calc!C15, Calc!C16, Calc!C17, Calc!C19, Calc!C22, Calc!C24.

## Acceptance criteria
- [ ] Every probes/t4382-pivotby-intransitive-paths cell named in Context passes (or the cells the hold-out
      refutes are recorded in coercion.md and owned).
- [ ] A hold-out probe of fresh seeded runs (three and four column fields, both orders), written with the
      engine's predictions committed before the fill, passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
