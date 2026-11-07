---
id: T-4705
title: "Tidy T-4570: one order-statistic error helper, re-wrap lift_references doc, tighten the XLOOKUP static shape to a reference if_not_found"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4570]
---

## Goal
The T-4570 code has one definition per concept and a static shape that matches its runtime shape.

## Context
MINOR findings from the PR #3757 reviews (design and Excel fidelity), deferred as follow-ups:
- `functions/statistical/chisq.rs` (`numbers_from_ranked_arg`) and `functions/util/aggregate_args.rs`
  (`walk_aggregate_read`, `GridReading::OrderStatistic`) each build the same
  `CalcResult::new_error(error, cell, "Array of references on another sheet")` from
  `RefGrid::order_statistic_error`. Make one helper (for example `RefGrid::order_statistic_result(sheet, cell)`)
  and call it from both.
- The doc comment of `lift_references` in `functions/lift/driver.rs` has an overlong line: re-wrap it.
- `expressions/parser/static_analysis.rs` `criteria_and_grid_static_result` gives a per-element call over any
  XLOOKUP with an `if_not_found` the lookup_value's shape. Only a reference `if_not_found` builds a grid (the
  engine's `NotFound::Range`). Narrow the shape to an `if_not_found` that is a reference node (`ReferenceKind`,
  `RangeKind`, or a reference-producing call), unless T-4704's probe shows Excel spills anyway.
- Optional: the `Node::FunctionKind { kind: Function::Xlookup, args }` arm that calls `xlookup_reference` appears
  in 4 to 5 readers (`functions/util.rs`, `ref_lift.rs`). Add one accessor if another reader joins.

## Acceptance criteria
- [ ] One helper builds the order-statistic error, and both readers call it.
- [ ] The static shape matches the runtime shape for the forms named above (with a unit test). No eval regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change beyond the static shape.
