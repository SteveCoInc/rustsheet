---
id: T-4388
title: "PIVOTBY's subtotals of a class no record reached and its row-axis parents on an intransitive run"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4359, T-4378]
---

## Goal
PIVOTBY's subtotals match Excel on an intransitive run where GROUPBY's rule is known but PIVOTBY's is unprobed: a
column subtotal of a class no record of the row group reached, and the row axis's subtotals and value-sort parents.

## Context
- T-4359 (probes/t4359-groupby-value-sort-parents, probes/t4359-groupby-value-sort-holdout): a GROUPBY parent
  holds the rows that reached its class, and every row when none did (a repeated class whose leaves no row
  reaches), for its subtotal line and its value-sort order (`Grouping::class_rows`, grouping.rs). PIVOTBY's row
  axis shares that code (`Grouping::line_parts`, `sort_by_aggregate`), unprobed on intransitive runs.
- T-4378: a PIVOTBY column subtotal holds the row group's records whose classes start with its prefix (the
  `Slot::Subtotal` arm of `Grid::cell_rows`, pivotby.rs), so a class no record reached gets none, where GROUPBY's
  gets every row and a PIVOTBY column leaf no record reaches gets every record of the row group (T-4358).
- GROUPBY's subtotals under a descending key sort take the same rows (`line_parts`), probed only ascending
  (probes/t4359-groupby-value-sort-holdout Calc!F).
- Probe them (seeded runs as docs/research/t4359/gen_probe.py, with A = ab, B = ab&U+00AD, C = U+00AD&ab), with
  predictions committed before the fill (decision 0188 R4), then implement what differs.

## Acceptance criteria
- [x] A probe of PIVOTBY row-axis subtotals and value sorts, column subtotals over a class no record reached, and
      GROUPBY subtotals under a descending key sort, on intransitive runs, filled; the engine matches it or the cells are owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY's ascending and value-sort subtotals (T-4359, probed).

## Outcome
- Probe probes/t4388-pivotby-intransitive-subtotals (excel-graph, 2026-11-05; 30 blocks, 180 cells). It was generated
  by docs/research/t4388/gen_probe.py from fresh seeded runs (seed "T-4388"): 15 three-field runs with a class above
  the leaves that no row reaches, and 15 two-field runs with a leaf no row reaches or a row in no leaf. The engine's
  values were committed as predictions (docs/research/t4388/predictions.tsv, commit 8bf50c35) before the fill.
  `docs/research/t4344/predict_engine.py` now takes the case and output path as arguments, so the script is not
  copied.
- Out of sample, the current model matched all 120 cells of B (PIVOTBY row subtotals), C (a sort by the first value
  column with them), F (GROUPBY subtotals, `sort_order` -1) and G (`sort_order` -2). T-4359's parent rule therefore
  holds on PIVOTBY's row axis and under descending key sorts.
- Finding: a PIVOTBY column subtotal of a class that no record reached shows every record of its row line, as that
  class's leaves do (Calc!D2: over CyB BxC CxC AyB BxB of p q q q q, the (`ab`&U+00AD, `y`) subtotal of q is
  `2, 3, 4, 5`). Before this PR, the engine gave such a subtotal no records (T-4378's prefix filter). This caused all
  15 of the 60 D and E misses.
  - Implemented in the `Slot::Subtotal` arm of `Grid::cell_rows` (pivotby.rs), using a new
    `Grouping::class_reached` (grouping.rs).
  - Recorded in docs/excel-semantics/coercion.md, "GROUPBY on an intransitive run".
  - This is a structural rule read off the cells, not a fit. Every one of the 15 cells it fixes has the same shape.
- Eval delta: new case, 705/705 cells pass (180 Excel formula cells), no new known failures. Other cases: no change;
  the t4358, t4378 and t4379 PIVOTBY cases score as before.
- Follow-up filed with this PR: T-4404, a probe of a color filter exported without `dxfId` (a minor finding of the
  T-4004 review, PR #3425).

## Landing notes
- PR #3430. Head before these notes: 9758c440c, a `make catch-up` merge of origin/main (decision 0207) after the
  queue dropped the PR for a conflict in eval/baseline/probes.txt with #3426 (T-4382, also PIVOTBY). The catch-up's
  eval run showed no regressions. CI was pending on that head when the session's landing window closed at 23:13 UTC,
  and this notes commit restarts it.
- Reviewers: security, design and excel-fidelity all APPROVE on 5f80c34, the reviewed code. Since then the branch has
  only merged main (two catch-ups), which needs no new review.
- Next step: once CI is green on the latest head, call `enable_pr_auto_merge` (SQUASH). If the queue drops it again for
  a baseline-only conflict, run `make catch-up PR=3430` and re-enqueue. If #3426's PIVOTBY change ever conflicts
  textually with `Grid::cell_rows`'s `Slot::Subtotal` arm, re-run probes/t4388-pivotby-intransitive-subtotals (705
  cells) and the excel-fidelity reviewer.
