---
id: T-3964
title: "GROUPBY: probe a 1x1 filter beside a header row, one record and two value columns, and non-#N/A error relationship elements"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY's 1x1-filter and lifted-error rules from T-3880 are evidenced past the shapes T-3880 probed, and arrays.md states only what the cells show.

## Context
- Follow-up of T-3880 (PR #3006, excel-fidelity review MINORs). T-3880's probe (probes/t3880-groupby-scalar-filter-edges)
  showed a 1x1 filter beside a header row and exactly one record is that record's element (G!A1:A41, two key columns
  G!A37), so `Model::filter_rows` (groupby.rs) tries the records-height arm before the one-value arm. That arm now also
  takes two or more value columns there, which no cell covers: probe `GROUPBY(S!A1:A2,S!B1:C2,_xleta.SUM,1,,,{TRUE})`
  and `..,TRUE)`, with field_headers 1 and 3.
- An error `field_relationship` element stays its own error after the filter is spent (`Model::lift_by_class`'s
  `settled`, G!A89:B102), evidenced only for `#N/A`. Probe `#DIV/0!` and `#REF!` elements (cells holding `=1/0` and an
  error value), beside `(Data!C1:C12<0.2)`, in both positions (before and after the spending element).
- An error element of a lifted `field_headers` or total depth beside a filter column is unprobed (arrays.md, "A lifted
  GROUPBY's filter serves one element"); add it to the same probe.
- NA() and other functions cannot sit in an array constant: use ranges holding the errors.

## Acceptance criteria
- [ ] An excel-graph probe answers the three questions; ours matches every cell (change `filter_rows` or the `settled`
      closures if not).
- [ ] arrays.md cites the new cells in the T-3778 scalar-filter entry and the T-3840 lifted-filter entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's scalar filter and lifts (T-3841, T-3892).
