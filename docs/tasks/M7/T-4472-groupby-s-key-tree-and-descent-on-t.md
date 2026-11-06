---
id: T-4472
title: "GROUPBY's key tree and descent on T-4393's UNIQUE: t4393 hold-out column C 32/60 and t4380 column C 38/45 under docs/research/t4241's tree"
milestone: M7
lane: research
status: dropped
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A model of GROUPBY's key tree and row descent on two-field intransitive runs that, on top of T-4393's UNIQUE model,
gives probes/t4393-unique-merge-holdout's and probes/t4380-unique-keep-head-holdout's GROUPBY cells, confirmed on a
fresh hold-out; or the negatives recorded and the cells parked with their reason.

## Context
- T-4393 found UNIQUE's merge (docs/research/t4393/model.py); its `groupby` runs docs/research/t4241/model.py's tree
  and lower-bound descent on that UNIQUE. It gives t4380 column C 38 of 45 and the t4393 hold-out column C 32 of 60
  (docs/research/t4393/predictions.tsv against the fill), UNIQUE itself being 60 of 60.
- The misses have two shapes. (1) A key that the merge keeps but GROUPBY's tree folds into an earlier class
  (Bx Cy Cx Ay Cx: Excel Bx 1, 3, 5; By 2; Ay 4, where the tree's descent leaves rows 1 and 4 in no leaf, so its Ay
  leaf holds every row). (2) Two subtotal-free classes of `=` texts in the other order (Bx By Cx Ay Cx: Excel
  Cx 1, 3, 5; Cy 2, 4; Cx all; Cy all, the model swaps the two `y` classes). Both are GROUPBY's tree, not UNIQUE.
- Owned cells: probes/t4393-unique-merge-holdout Calc!C1:C60; probes/t4380-unique-keep-head-holdout Calc!C1:C45 (the
  t4380 column C cells T-4471 also lists until it lands). T-4471 hands over the t4379 and t4358 PIVOTBY cells it
  takes from T-4393 that its UNIQUE change leaves off.

## Acceptance criteria
- [ ] A tree/descent model scored on t4380 and t4393 column C and docs/research/t4241/model.py's 234 blocks, then on
      a fresh hold-out (predictions committed before the fill); or the candidates and scores in coercion.md and the
      owned cells parked with that reason.
- [ ] If a model fits, an engine task is filed and takes the owned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- UNIQUE's merge (T-4471).

## Outcome
Dropped (T-4471, 2026-11-06). Once `coalesce_equal_lines` took T-4393's merge, the engine's own GROUPBY tree passed
every owned cell: probes/t4393-unique-merge-holdout 884 of 884 and probes/t4380-unique-keep-head-holdout 622 of
622. The shortfall this task was filed for, 32 of 60 and 38 of 45, came from docs/research/t4241/model.py's Python
descent, not from Excel or the engine. No model is left to find.
