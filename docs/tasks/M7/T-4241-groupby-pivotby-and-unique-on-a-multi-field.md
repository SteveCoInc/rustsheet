---
id: T-4241
title: "GROUPBY and PIVOTBY on a multi-field intransitive run: rows Excel puts in two groups (probes/t4195-groupby-intransitive)"
milestone: M7
lane: engine
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY and PIVOTBY give Excel's groups on multi-field keys where Excel's `=` is
intransitive in a field and the rows differ in a later one, including the runs where Excel puts one row in two
groups.

## Context
- Evidence: probes/t4195-groupby-intransitive (excel-graph, 2026-11-04, T-4195); coercion.md "GROUPBY on an
  intransitive run (T-4195)". With A = `ab`, B = `ab`&U+00AD, C = U+00AD&`ab` (A = C = B, A < B):
  - rows (A,y), (B,x), (C,y) give (`ab`,y): rows 1, 3; (`ab`&U+00AD,x): 2; (`ab`&U+00AD,y): 1, 2, 3 (Calc!C8):
    row 2 is in two groups, and a third group shows a key tuple no row has.
  - rows (A,y), (B,y), (C,x) give (U+00AD&`ab`,x): 3; (U+00AD&`ab`,y): 1, 2 (C9): A and B, unequal, share a group.
  - UNIQUE of the same rows keeps 3 and 3 (D8:D9), as its merge meets the rows (T-4200's `coalesce_equal_lines`);
    ours matches every UNIQUE cell of the probe.
- Ours (T-4195): each field is classed within its parent by `classify` (functions/lookup_and_reference/
  grouping.rs: first `=` keys, sorted by `<`, a lower-bound search), and a group none of UNIQUE's kept rows is in
  is dropped. That matches every single-field row, every shown-key row (probes/t4195-groupby-shown-key) and 98 of the
  112 GROUPBY cells the simulator scored, not these.
- UNIQUE's order dependence is T-4200's merge order; GROUPBY's double membership may come from the same merge over
  whole rows. Probe first: the same rows with the x row moved, 4-row runs, a descending `sort_order` and a table sort naming
  the second key first (both unprobed on these runs), which also serve as the fresh
  confirmation T-4195's model (selected on its own probe's cells) lacks.
- Review notes from T-4195 (design, MINOR): `Grouping::new` is about 130 lines; when this task changes it, split
  the whole-row filter and the group building into helpers, and share the text-pair `=`/cost closures.
- Owned cells: none since this task's outcome. The 24 t4195 cells, the 2 t4088 cells and the 140 cells of
  probes/t4241-groupby-intransitive-runs moved to T-4342, the engine change.

## Acceptance criteria
- [x] The owned cells pass and leave eval/baseline/probes.txt with 0 regressions, or the task records, with probe
      evidence, which of them stay known failures and why.
- [x] coercion.md states the rule (VERIFIED) and "Implemented in".
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT's order on intransitive cycles (T-3418).

## Outcome
- New probe probes/t4241-groupby-intransitive-runs: 48 blocks, 336 formula cells (excel-graph, 2026-11-05), from
  docs/research/t4241/gen_probe.py. It holds T-4195's 18 three-row blocks again and 30 seeded four-row runs. Each
  block is shown ascending, with `sort_order` -1, 2 and -2, with a SUM aggregate, and through UNIQUE. The fresh
  three-row cells repeat T-4195's Excel values exactly.
- Excel's rule was found and recorded in coercion.md, "GROUPBY on an intransitive run" (VERIFIED for the cells
  cited). UNIQUE's kept rows are sorted, then classed field by field, chaining each key to its class's first key.
  Each row descends that tree with a lower-bound search in the field's sort direction. A leaf no row reaches
  aggregates every row. docs/research/t4241/model.py fits 234 of 234 blocks (36 + 6 of t4195, 4 x 48 of t4241),
  the shown keys and the sort directions included.
- Rejected models are scored in docs/research/t4241/sim.py and brute_ranges.py (run with PYTHONPATH=docs/research/t4241).
  A per-row single membership cannot double a row. Whole-row
  binary-search ranges, over any row order, fit only 33 of t4241's 48 ascending blocks.
- Selection: the model was chosen after looking at the t4241 values, so neither probe is a hold-out. T-4342 owes a
  confirmation probe whose predictions are committed before the fill (decision 0188 R4, a structural model).
- No engine change, which moves to T-4342 (P0, opus). The sort direction has to enter `Grouping::new`.
- Eval delta: the new case checks 636 cells. 496 pass as the engine stands, and 140 are known failures owned by
  T-4342. 0 regressions; every other suite is unchanged.
