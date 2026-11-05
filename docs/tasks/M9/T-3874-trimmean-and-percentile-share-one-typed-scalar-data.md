---
id: T-3874
title: "TRIMMEAN and PERCENTILE share one typed-scalar data reader; probe a picked cell and typed FALSE"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TRIMMEAN reads its data argument through the same typed-scalar path as PERCENTILE, and a picked or named cell and a typed FALSE are backed by oracle evidence.

## Context
- T-3790 (PR #2894) made `TRIMMEAN("2.5",0)` 2.5 and `TRIMMEAN(TRUE,0)` 1 in the `TRIMMEAN_DATA` reader (trimmean.rs); its reviewers noted:
  - the String/Boolean arms catch every non-literal-reference value, so `TRIMMEAN(INDEX(<"2.5" cell>,1),0)` or a name for a text cell is read as a typed literal; PERCENTILE splits picked cells with `evaluate_picked_reference_arg(..).into_single_sheet_area(..)` (percentile.rs, T-2344). No TRIMMEAN oracle evidence exists for those forms;
  - the arms nearly duplicate percentile.rs's typed boolean/text to one-element vector (CLAUDE.md directive 3);
  - `TRIMMEAN(FALSE,0)` is unprobed (FALSE read as 0 by analogy with probes-numeric/t1748-typed-boolean-data).
- Use the `excel-probe` skill for the three probes.

## Acceptance criteria
- [ ] A probe case covers `TRIMMEAN(INDEX(<text cell>,1),0)`, `TRIMMEAN(<name of a TRUE cell>,0)` and `TRIMMEAN(FALSE,0)` with oracle values.
- [ ] TRIMMEAN and PERCENTILE share one helper for a typed scalar data argument, and TRIMMEAN's picked-cell behavior matches the probe.
- [ ] docs/excel-semantics/statistics.md "TRIMMEAN of a typed scalar (T-3781)" records the results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other sorted_data.rs users unless the probe shows the same rule.
