---
id: T-4380
title: "UNIQUE drops a later key equal to an earlier kept one over Cx Ay Ax Bx (keeps Cx, Ay), and GROUPBY and PIVOTBY follow it"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
UNIQUE, and through it GROUPBY's and PIVOTBY's key trees, keep the rows Excel keeps on the intransitive runs of
probes/t4379-pivotby-repeated-unreached-leaf: over Cx Ay Ax Bx (A = ab, B = ab&U+00AD, C = U+00AD&ab; A = C, B = C,
A < B) Excel's UNIQUE is (`­ab`,x), (`ab`,y) and drops Bx, where T-4200's merge (`coalesce_equal_lines`,
docs/research/t4241/model.py `unique_kept`) keeps Cx, Ay, Bx.

## Context
- probes/t4379-pivotby-repeated-unreached-leaf (T-4379, excel-graph, 2026-11-05; docs/excel-semantics/coercion.md,
  "UNIQUE drops Bx after Cx Ay Ax"): UNIQUE (B), GROUPBY (C), PIVOTBY by p/q (D) and by one row group (E) all miss
  on the same five runs, Cx Ay Ax Bx Ax, Cx Ay Ax Bx, Cx Ay Ax Bx Cx, Cx Ay Ax Bx Bx and Cx Ay Ax Ax Bx; on each
  Excel's UNIQUE keeps two rows and GROUPBY/PIVOTBY show no third (C,x) group. Every other run of the probe (10 of
  15, including Cx Ay Bx Ax, Cx Ax Ay Bx Ax, Cx Cx Bx Ay Bx and Cx Ax Cx Ay Bx) and every F cell
  (`col_sort_order` -1) match the engine. The engine's predictions and the model's are committed in
  docs/research/t4379 (predictions.tsv, model_predictions.tsv).
- A Bx after an Ax that followed Cx Ay is dropped; a Bx before the Ax (Cx Ay Bx Ax), or an Ax before the Ay
  (Cx Ax Ay Bx Ax), keeps it. A model change to `unique_kept` must still give every D cell of t4200's probes and
  all 234 blocks docs/research/t4241/model.py scores, and owes a hold-out probe (predictions committed before the
  fill) if it is chosen on these cells.
- The same gap is probes/t4358-pivotby-unreached-column-leaf Calc!B6 (and D6, whose subtotal is also T-4378's).
- Owned cells: probes/t4379-pivotby-repeated-unreached-leaf (20): Calc!B1, Calc!C1, Calc!D1, Calc!E1, Calc!B2, Calc!C2, Calc!D2, Calc!E2, Calc!B8, Calc!C8, Calc!D8, Calc!E8, Calc!B9, Calc!C9, Calc!D9, Calc!E9, Calc!B10, Calc!C10, Calc!D10, Calc!E10; probes/t4358-pivotby-unreached-column-leaf
  (2): Calc!B6, Calc!D6.

## Acceptance criteria
- [ ] The owned cells pass, or stay known failures with the reason recorded in docs/excel-semantics/coercion.md;
      no cell of t4200's, t4241's, t4344's, t4358's or t4379's probes regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
