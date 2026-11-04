---
id: T-4028
title: "GROUPBY lifted options beside a spent filter: probe blank and empty-string elements before an error"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY's spent-filter rule for a lifted field_headers and total depth position is evidenced for a blank-cell element and
an empty-string element before an error, and the arrays.md entry claims no more than its probes show.

## Context
- Follow-up of T-4019 (PR #3086; excel-fidelity and design MINORs). T-3985 and T-4019 showed that, after an earlier
  element spent the filter, numeric text ("1", "5", "0.5"), the number 5, TRUE and FALSE before `#N/A` are `#N/A`, and text
  "x" is `#VALUE!` (probes/t3985-groupby-two-lifted-spent-filter, probes/t4019-groupby-lifted-spent-filter-kinds). Our
  `settled` closure in `Model::lift_options` (crates/rustsheet-engine/src/functions/lookup_and_reference/lifted_options.rs)
  also lets a blank element (read as 0) pass through, and treats `""` as `Class::OtherText`, which stops the error. Neither
  case is probed.
- Probe as T-4019 does: 1x2 S!A<n>:B<n> field_headers and S!D<n>:E<n> total depth beside `(Data!C1:C12<0.2)`, with
  position 1 {0, 1} and position 2 holding a blank cell, or a `=""` cell, before `=NA()`, in both argument orders, plus
  no-filter controls.
- docs/excel-semantics/arrays.md "A lifted GROUPBY's filter serves one element": the T-4019 bullet's "Nor does any other
  element that is no error and no non-numeric text", and the "Ours:" paragraph's "every other non-error element", go
  further than the probes show. Reword both to name the probed kinds, or extend them once this probe answers.
- Design MINOR: the `lift_by_class` doc comment line "element and whether the filter is spent) is that value and spends
  nothing: an element whose call returns before it filters," is about 130 characters. Rewrap it to the file's width.

## Acceptance criteria
- [ ] An excel-graph probe answers each combination, and ours matches every cell (change the `settled` closure if not).
- [ ] arrays.md cites the cells and claims only probed kinds; the `lift_by_class` doc line is rewrapped.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's lift beside a filter column (T-3892).
