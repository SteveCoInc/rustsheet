---
id: T-4310
title: "Move chisq.rs, regression.rs and aggregate_args.rs from a hand-rolled Node::ReferenceKind check to evaluate_picked_reference_arg (T-2883 follow-up)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
No statistical function decides "was this argument a cell reference" with a hand-rolled `Node::ReferenceKind` check on the node; the data of each is read with `evaluate_picked_reference_arg` (one definition).

## Context
T-2883 moved TRIMMEAN and Z.TEST. `rg 'Node::ReferenceKind' crates/rustsheet-engine/src/functions` still lists statistical/chisq.rs, regression.rs and util/aggregate_args.rs. probes/t2883-stat-name-index-data shows CHISQ.TEST, SLOPE and FORECAST already match Excel for name/INDEX/IF-picked cells, so behaviour must not change (that probe stays 168/168).

## Acceptance criteria
- [x] The three files read their data once in reference context; probes/t2883-stat-name-index-data stays 168/168 and no other eval cell changes.
- [x] One definition: no hand-rolled reference check is left in them.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Claimed 2026-11-07 by an opus run with too little session budget left to finish, so it was handed off before any code
change. No code is on this branch. What the three sites do on origin/main (read before changing them):
- statistical/chisq.rs `numbers_from_ranked_arg` (the RANK/LARGE/SMALL data reader, not CHISQ.TEST): `by_reference`
  makes a direct cell holding a scalar `#NUM!` for the RANK family, while the same scalar typed in place is a
  one-element list. Data is read with `evaluate_reference_arg`, not the picked form. Moving it to
  `evaluate_picked_reference_arg` would change name, INDEX and IF-picked cells for RANK, LARGE and SMALL.
  probes/t2883-stat-name-index-data does not show which way Excel goes for these functions, so probe that first
  (`excel-probe`).
- statistical/regression.rs `read_known_y_and_x`: a blank y is `#VALUE!` unless the y node resolves to a
  `ReferenceKind`. A defined name or INDEX that points at a blank cell is unprobed.
- util/aggregate_args.rs `walk_aggregate_arg`: `is_reference` only picks `ItemSource::ReferenceValue` vs `Direct`
  in the `Err(value)` arm (no areas read). Its doc says no consumer sees `ReferenceValue` yet, so this one may move
  without a behavior change. Check `walk_aggregate_read`'s callers (count.rs passes `false`, complex/mod.rs its own
  check).
Next: probe RANK/LARGE/SMALL and LINEST/SLOPE with a name, INDEX and IF picking a scalar cell and a blank cell, then
move each site to `evaluate_picked_reference_arg` as the evidence says.

## Outcome
- Probed first (probes/t4310-rank-regression-picked-cells, 156 cells, excel-graph 2026-11-07): RANK, RANK.EQ,
  RANK.AVG, LARGE and SMALL over a number, boolean, numeric-text or blank cell, and LINEST, LOGEST, TREND and GROWTH
  over a blank `known_y`, are the same through a plain reference, a name, INDEX, IF, CHOOSE and parentheses
  (statistics.md, new T-4310 entry).
- statistical/chisq.rs: the RANK/LARGE/SMALL data reader reads with `evaluate_picked_reference_arg`; the dead
  "cell holding a scalar is `#NUM!`" branch (a cell always arrives as a range) and its node check are gone.
- statistical/regression.rs: a blank y asks `Model::is_reference` (the ISREF definition, asked only for a blank y), so a
  name, INDEX, IF or parenthesized blank waits for x's error like the plain cell: 16 cells fixed.
- util/aggregate_args.rs (and the same check in engineering/complex/mod.rs): the flag picks `ReferenceValue` only for a
  value read out of a written cell reference (a linked workbook's cached cell); it now uses the shared
  `Node::is_cell_reference_at` instead of a hand-rolled match, no behaviour change. Its data is already read with the
  picked reader.
- Eval delta: +16 cells (the new probe 140 → 156 of 156); probes/t2883-stat-name-index-data stays 168/168; no other
  cell changes.
