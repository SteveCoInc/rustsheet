---
id: T-3946
title: "Point the aggregate_cmp references at group_order.rs and probe non-number elements across array aggregates"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-3893]
---

## Goal
The docs name the right file for `aggregate_cmp`, and Excel evidence says how two array aggregates compare when their elements are text, booleans or errors against numbers.

## Context
- MINOR findings of the design and excel-fidelity reviewers on PR #2986 (T-3893). `aggregate_cmp` lives in
  crates/rustsheet-engine/src/functions/lookup_and_reference/group_order.rs, but three references say grouping.rs:
  docs/excel-semantics/arrays.md ("Ours: implemented (`grouping.rs` `aggregate_cmp`)"), the `NestedArray::elements` doc
  comment in crates/rustsheet-engine/src/nested_array.rs, and the Outcome of docs/tasks/M7/T-3893-pivotby-row-sort-by-an-error-holding-values.md.
- `aggregate_cmp` compares non-blank array elements with `key_cmp` (numbers < text < FALSE < TRUE < errors). Only text
  against TRUE as first elements is probed (probes/t3893-pivotby-error-tie-order A641:A654). Probe, with `excel-probe`,
  GROUPBY value sorts over `LAMBDA(x,IF(..,{"a";1},{1;2}))`-style arrays whose first elements are a number against text,
  a boolean and an error, and a blank element against text, a boolean and an error.

## Acceptance criteria
- [x] The three references name group_order.rs.
- [x] An excel-graph probe covers the element pairs above; arrays.md records the result, and the engine matches every
      probe cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- Key sorts; error aggregates (already probed).

## Outcome
- The three references (arrays.md, `NestedArray::elements`, the T-3893 Outcome) name group_order.rs.
- Probe `t3946-aggregate-array-element-kinds` (excel-graph, 80 cells): a number sorts before text, TRUE and an error,
  and a blank element after all of them, ascending and descending. The engine already matched every cell; no code
  change. arrays.md records it. Eval delta: +80 probe cells passing (new case), no regressions.
