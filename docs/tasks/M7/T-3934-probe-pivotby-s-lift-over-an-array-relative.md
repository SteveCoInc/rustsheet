---
id: T-3934
title: "Probe PIVOTBY's lift over an array relative_to: a validation error (bad sort_order, total depth) and the filter beside it"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for two PIVOTBY lifts over an array `relative_to` that T-3892's batch did not cover, recorded in
docs/excel-semantics/arrays.md, so the engine's rule is evidenced rather than extrapolated.

## Context
- Review follow-up of T-3892 (#2974). Since T-3892, `Model::pivotby` lifts every error of `Model::pivotby_args` over an
  array `relative_to` (`Model::lifted_error`), as GROUPBY does. The gen-m7 batch evidences only a text, scalar or
  error-cell `row_fields`, an error option and a filter refusal. The lift of a validation error (an invalid
  `row_sort_order` such as 99, a total depth of 3 with one field, a non-function `function`, blank-cell `values`) is
  extrapolated: arrays.md, the PIVOTBY bullet owned by T-3892.
- T-3892's Context also asked whether PIVOTBY's lift beside a filter spends the filter as GROUPBY's does (arrays.md
  "A lifted GROUPBY's filter serves one element", T-3840); the batch had no failing cell of that shape.
- Use the `excel-probe` skill: one probe case, a 2x2 `relative_to` (`Data!A1:B2`) beside each validation error, and a
  lifted call with a valid filter, with `{0;1}` and a 2x2 relative_to.

## Acceptance criteria
- [ ] A probe case with oracle values (excel-graph) covers each validation error above and the filter-beside-lift shape.
- [ ] arrays.md records what Excel does; any cell the engine gets wrong goes to a new engine task (`make new-task`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task does that if the probe shows a gap).
