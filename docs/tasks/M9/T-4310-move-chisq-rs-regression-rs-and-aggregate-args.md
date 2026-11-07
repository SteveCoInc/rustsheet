---
id: T-4310
title: "Move chisq.rs, regression.rs and aggregate_args.rs from a hand-rolled Node::ReferenceKind check to evaluate_picked_reference_arg (T-2883 follow-up)"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] The three files read their data once in reference context; probes/t2883-stat-name-index-data stays 168/168 and no other eval cell changes.
- [ ] One definition: no hand-rolled reference check is left in them.
- [ ] `make check` green; reviewers approve.

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
