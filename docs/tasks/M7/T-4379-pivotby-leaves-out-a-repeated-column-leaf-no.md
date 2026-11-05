---
id: T-4379
title: "PIVOTBY leaves out a repeated column leaf no record reaches on some intransitive runs"
milestone: M7
lane: engine
status: in-progress
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
- Owned cells: probes/t4358-pivotby-unreached-column-leaf (2): Calc!B6, Calc!D6.

## Acceptance criteria
- [ ] Probe GROUPBY and PIVOTBY on B6's rows and a few variants (with and without rows whose key UNIQUE drops);
      the owned cells pass, or stay known failures with the reason recorded in docs/excel-semantics/coercion.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
