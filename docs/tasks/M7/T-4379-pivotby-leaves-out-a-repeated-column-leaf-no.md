---
id: T-4379
title: "PIVOTBY leaves out a repeated column leaf no record reaches on some intransitive runs"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4358]
---

## Goal
PIVOTBY's column headers match Excel where the column tree repeats a key in a leaf no record reaches: over
Cx Ay Ax Bx Ax (q q p p q) Excel shows the columns (`­ab`,x), (`­ab`,y) only, where the engine adds a third,
(`­ab`,x) again, holding every record.

## Context
- Found by T-4358's hold-out: probes/t4358-pivotby-unreached-column-leaf Calc!B6 (and D6, its column-subtotal
  form). The column tree (docs/research/t4241/model.py, T-4344's search) is one class C holding the keys Cx, Ay,
  Bx, so leaves (C,x), (C,y), (C,x); every record reaches the first two and the third no record reaches.
- probes/t4344-groupby-descending-search Calc!D39 (Cx Cx Bx Ay Bx) has the same key tree, and there Excel does
  show the third (C,x) leaf, holding every record of each row group (T-4358). The difference is in the data rows:
  B6 has rows Ax, whose key UNIQUE does not keep (Ax = Cx), D39 has none. GROUPBY on B6's rows is unprobed.
- The other 24 blocks of the probe show every leaf the model builds, so a rule should be checked against them,
  against t4344 Calc!B31:E42 and against t4241 Calc!B25 (GROUPBY's repeated (C,x)).
- Owned cells (until T-4379 is done; then T-4380's): probes/t4358-pivotby-unreached-column-leaf (2): Calc!B6, Calc!D6.

## Acceptance criteria
- [x] Probe GROUPBY and PIVOTBY on B6's rows and a few variants (with and without rows whose key UNIQUE drops);
      the owned cells pass, or stay known failures with the reason recorded in docs/excel-semantics/coercion.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe: probes/t4379-pivotby-repeated-unreached-leaf (new; 15 runs of B6's keys and variants, 75 formulas: UNIQUE,
  GROUPBY, PIVOTBY by p/q, by one row group and with `col_sort_order` -1), written by docs/research/t4379/gen_probe.py
  with the engine's predictions (predictions.tsv) and the t4241 model's UNIQUE and GROUPBY predictions
  (model_predictions.tsv) committed before the excel-graph fill.
- Finding: the left-out leaf is UNIQUE's doing, not PIVOTBY's. Over Cx Ay Ax Bx Excel's UNIQUE keeps Cx, Ay and
  drops Bx (which `=` Cx only), where T-4200's merge keeps it; GROUPBY and PIVOTBY build their trees on those two
  keys, so the repeated (C,x) leaf never exists. Five runs (rows 1, 2, 8, 9, 10) miss in B, C, D and E alike; the
  other ten runs and every F cell match the engine, 55 of 75 cells. Recorded in docs/excel-semantics/coercion.md
  ("UNIQUE drops Bx after Cx Ay Ax").
- No engine change: the rule is a revision of UNIQUE's merge model that must keep every earlier UNIQUE/GROUPBY probe
  and owes a hold-out, so it is filed as T-4380 (M7, engine, opus), which owns the 20 new known-failure cells and
  t4358 Calc!B6, D6 (the cells this task owned stay known failures).
- Eval delta: probes +20 known failures in the new case (owned by T-4380), 0 fixed, 0 regressions; `make
  eval-baseline` records them.
