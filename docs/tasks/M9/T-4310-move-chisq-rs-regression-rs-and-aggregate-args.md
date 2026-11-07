---
id: T-4310
title: "Move chisq.rs, regression.rs and aggregate_args.rs from a hand-rolled Node::ReferenceKind check to evaluate_picked_reference_arg (T-2883 follow-up)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
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
