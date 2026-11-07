---
id: T-4310
title: "Move chisq.rs, regression.rs and aggregate_args.rs from a hand-rolled Node::ReferenceKind check to evaluate_picked_reference_arg (T-2883 follow-up)"
milestone: M9
lane: functions-stat
status: todo
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

## Handoff
Sonnet run read the three sites; no code changed. Findings, not instructions:
- `chisq.rs:435` (`numbers_from_ranked_arg`): `by_reference` is a node check used as a flag next to `evaluate_reference_arg`/`external_reference_cells`; it feeds the RANK-family "array that is not a reference" rule (`arrays_are_value_errors`), so swapping to `evaluate_picked_reference_arg` can change which IF/CHOOSE-picked cells count as references for RANK/LARGE.
- `regression.rs:52` (`read_known_y_and_x`): `!matches!(y_arg.resolved_at(..), ReferenceKind)` decides whether a blank y is `#VALUE!`; a picked reference (name/INDEX/IF) probably should count as a reference, but this needs a probe row since t2883 covers only CHISQ.TEST/SLOPE/FORECAST data.
- `util/aggregate_args.rs:365`: `is_reference` is passed to `walk_aggregate_read` (ItemSource::ReferenceValue vs Direct); `walk_aggregate_read` takes a `RefArg` already read, so the flag could be derived from the `RefArg` variant if every caller reads with the picked reader. Callers (COUNTA etc.) choose their own reader.
Next step: derive the flag from the `RefArg` variant (`Ranges`/`Grid` = reference) at each site, run t2883-stat-name-index-data (must stay 168/168) and rank/regression/aggregate probes, and probe any cell whose behaviour changes before accepting it.
