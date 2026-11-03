---
id: T-3964
title: "GROUPBY: probe a 1x1 filter beside a header row, one record and two value columns, and non-#N/A error relationship elements"
milestone: M7
lane: functions-lookup
status: done
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
- [x] An excel-graph probe answers the three questions; ours matches every cell (change `filter_rows` or the `settled`
      closures if not).
- [x] arrays.md cites the new cells in the T-3778 scalar-filter entry and the T-3840 lifted-filter entry.
- [x] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's scalar filter and lifts (T-3841, T-3892).

## Outcome
- New excel-graph probe `probes/t3964-groupby-filter-lift-errors` (144 cells, 2026-11-03) answers the three questions.
- Q1: a 1x1 filter beside a header row and one record keeps the record with two value columns too (field_headers 1
  and 3; TRUE, `{TRUE}`, 2, a TRUE cell); `{FALSE}`, two records and field_headers 0 are `#VALUE!` (G!A1:C42). Ours
  already matched.
- Q2: `#DIV/0!` (`=1/0` cell) and `#REF!` field_relationship elements stay their own error before and after the filter
  is spent and spend nothing (G!A47:B69). Ours already matched.
- Q3: an error element of a lifted field_headers or total depth stays its error after the filter is spent (G!B71, B83,
  B89, B101; ours was `#VALUE!`) and spends nothing before it (G!A77, A95). Fixed: `Model::lift_options` passes a
  `settled` closure to `Model::lift_by_class` (which now gets whether the filter is spent) that answers a spent
  position's first error when no text element precedes it. Unit test
  `an_error_option_element_is_its_own_error_after_the_filter_is_spent`.
- arrays.md: the T-3778 scalar-filter entry and the T-3840 lifted-filter entry cite the new cells; a position with an
  error and a text element, or two errors, across two lifted options stays unprobed (noted there).
- Eval delta: new probe case 144/144 (4 cells fixed by the change); no other case changed (see PR).
