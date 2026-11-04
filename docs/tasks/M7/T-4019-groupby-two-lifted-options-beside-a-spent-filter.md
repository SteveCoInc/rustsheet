---
id: T-4019
title: "GROUPBY two lifted options beside a spent filter: probe out-of-range numeric text and a boolean before an error"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY's spent-filter rule for a lifted field_headers and total depth position is evidenced for the element kinds
T-3985 left unprobed, and `Model::lift_options`' `settled` closure follows the cells.

## Context
- Follow-up of T-3985 (PR #3073, excel-fidelity MINOR). T-3985 showed that, after an earlier element spent the
  filter, a position whose field_headers and total depth hold numeric text "1" or the number 5 before `#N/A` is `#N/A`,
  while text "x" before it is `#VALUE!`. Since T-3985 the `settled` closure stops only at an error or `Class::OtherText`,
  so every other element passes through to the error, unprobed: numeric text out of range ("5", "0.5"), TRUE and FALSE.
- Probe beside `(Data!C1:C12<0.2)` as T-3985's probe does (1x2 ranges, position 1 {0 field_headers, 1 total depth}
  spends the filter), position 2 holding each of those before `#N/A`, in both argument orders, with no-filter controls.
- Also from the T-3985 review (design MINOR): `lift_by_class`' doc says `settled` is "given whether the filter is
  spent"; it now takes the model and the element as well. Reword it.
- docs/excel-semantics/arrays.md "A lifted GROUPBY's filter serves one element".

## Acceptance criteria
- [ ] An excel-graph probe answers each combination; ours matches every cell (change the `settled` closure if not).
- [ ] arrays.md cites the cells; the `lift_by_class` doc sentence is reworded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's lift beside a filter column (T-3892).
