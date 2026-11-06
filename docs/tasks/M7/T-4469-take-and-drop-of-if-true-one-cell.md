---
id: T-4469
title: "TAKE and DROP of IF(TRUE,<one-cell ref>) keep the reference: TYPE(TAKE(IF(TRUE,Z1),1)) is 1, ours 64"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TAKE and DROP read `IF(TRUE,Z1)` (an IF that returns a one-cell reference) as a reference, the way Excel does. So
`TYPE(TAKE(IF(TRUE,Z1),1))` is 1, as `TYPE(TAKE(Z1,1))` already is.

## Context
- probes/t4448-lone-blank-reshapers Sheet1!B9 (excel-graph, 2026-11-06, T-4448): Excel gives 1 and the engine gives 64.
  Cells A9, C9 and D9 of the same row pass after T-4448: the blank stays blank.
- crates/rustsheet-engine/src/functions/lookup_and_reference/drop_take.rs: a source keeps its reference reading only
  when it is a reference, a defined name or INDEX (`evaluate_node_with_reference`). Every other source, IF included, is
  read through `value_or_grid` or `evaluate_picked_source`, so the result is a 1x1 array.
- Adding `Function::If` to that match did not change B9 (T-4448 tried it), so the value path of IF's picked branch is
  where to look.
- Opus: this touches the reference-or-value reading shared with CHOOSE and the other picked sources (T-3072, T-3085).

## Acceptance criteria
- [ ] probes/t4448-lone-blank-reshapers B9 passes, and its baseline line drops the cell.
- [ ] arrays.md records the rule with the cell.
- [ ] 0 regressions; `make check` green; reviewers approve.

## Out of scope
- Other picked sources (CHOOSE, IFS, SWITCH) unless the probe shows they share the rule.
