---
id: T-4241
title: "GROUPBY and PIVOTBY on a multi-field intransitive run: rows Excel puts in two groups (probes/t4195-groupby-intransitive)"
milestone: M7
lane: engine
status: todo
priority: P3
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
- Owned cells: probes/t4195-groupby-intransitive (24): Calc!C8, C9, C11, C15, C16, C17, C20, C21, C23, C24,
  E46, E48, E49, E50, F8, F9, F11, F15, F16, F17, F20, F21, F23, F24; probes/t4088-groupby-multifield
  (2): Calc!B11, B33.

## Acceptance criteria
- [ ] The owned cells pass and leave eval/baseline/probes.txt with 0 regressions, or the task records, with probe
      evidence, which of them stay known failures and why.
- [ ] coercion.md states the rule (VERIFIED) and "Implemented in".
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT's order on intransitive cycles (T-3418).
