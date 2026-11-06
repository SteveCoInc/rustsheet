---
id: T-4464
title: "GROUPBY's spent filter beside a lift: probe the three modeled cases (a filter keeping no row, a table relationship with subtotals, a nested lift) and match Excel"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The three shapes of GROUPBY's "a lifted filter serves one element" rule that the engine models without evidence are
probed on excel-graph, recorded in arrays.md as VERIFIED, and the engine matches every probed cell: a `filter_array`
that keeps no row, a table `field_relationship` with subtotals, and a lift nested inside another lift.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 07a6b88a, all suites fetched). No visibly-off cell is unowned; this
  is a decision 0031 evidence gap in an M7 function: docs/excel-semantics/arrays.md "A lifted GROUPBY's filter serves
  one element (T-3840)" ends "UNVERIFIED, and modeled from the order above: a filter that keeps no row (ours: not
  spent, the call is `#VALUE!` before it filters anything), a table relationship with subtotals (ours: spent, that
  check comes after the filter), and a lift nested inside another lift (ours: the inner lift spends nothing outside
  it)". No open task names them (`rg -i 'keeps no row|nested inside another lift' docs/tasks`).
- What is evidenced: when GROUPBY lifts over an array `field_relationship`, `field_headers` or total depth beside a
  `filter_array` column that drops at least one row, only the first element whose call filters gets its value, and
  every later one is `#VALUE!`; an element refused before the filter spends nothing, one refused after it spends it
  (probes/t3840-groupby-lifted-beside-filter, t3840-groupby-lifted-filter-consumed, t3880-groupby-scalar-filter-edges,
  t3964-groupby-filter-lift-errors, t3985-groupby-two-lifted-spent-filter, t4019-groupby-lifted-spent-filter-kinds,
  t4028-groupby-lifted-spent-filter-blank; all excel-graph).
- The three modeled answers each pick a side of that order (before or after the filter), so each is a real question,
  not a detail: a no-row filter may be refused before filtering (spends nothing) or after (spends it); the table
  relationship's subtotal refusal is placed after the filter only by analogy with the total depth of 5.
- Code: `Model::groupby_args` (where the filter, field_relationship and total depth are read and checked, and
  `Model::groupby_filter_spent` is set), `Model::lift_by_class` (later elements `#VALUE!`), and the save/restore of
  the flag around a cell read on demand in evaluation.rs (`filter_spent`, T-3840).
- The engine's prediction for every probe cell is committed before the fill (as docs/research/t4382/predict_engine.py
  does), so the probe also tests the model rather than only fitting it.
- PIVOTBY's own lift beside a filter is T-3934's (sonnet probe); do not duplicate its cells.

## Acceptance criteria
- [ ] One probe case (excel-probe skill, excel-graph, tag M7), with the engine's predictions committed before the
      fill, covers, each with a no-filter control and with the filter as the first and as a later element:
      (a) a `filter_array` column that keeps no row (`Data!C1:C12>99`) beside a 2x2 lifted `field_relationship`,
      `field_headers` and total depth; (b) a lifted `field_relationship` holding 1 (table) with total depth 1, 2 and
      -1 beside a dropping filter; (c) a GROUPBY whose lifted argument is itself the result of an inner lifted
      GROUPBY or MAP call, beside a dropping filter in the outer call, the inner call, and both.
- [ ] arrays.md records the result per shape, citing cells, and drops the "UNVERIFIED, and modeled" sentence for
      each shape the probe settles.
- [ ] Every cell of the new probe passes; no other eval cell regresses (`make eval-baseline` refuses none).
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's lift beside a filter column (T-3934).
- GROUPBY's subtotal and grouping rules on intransitive runs (T-4359, T-4393).
