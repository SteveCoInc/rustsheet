---
id: T-3880
title: "GROUPBY scalar filter: probe a header row with one record and a detected header row; narrow arrays.md"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY's scalar-filter rule (T-3839) is evidenced at its two unprobed edges, and arrays.md states only what the cells show.

## Context
- Follow-up of T-3839 (PR #2909, excel-fidelity review MINORs). T-3839's rule: a scalar or 1x1 `filter_array` that is not as
  tall as the fields keeps every row only beside one key column, one value column and no header row; otherwise `#VALUE!`
  (probes/t3839-groupby-scalar-filter). `Model::filter_rows` (groupby.rs) applies it via `one_keeps_all`.
- Edge 1: a header row with exactly one record (`GROUPBY(A1:A2,B1:B2,_xleta.SUM,1,,,{TRUE})` and `..,1,,,TRUE)`). The 1x1
  filter is as tall as the records there, which the aligned-with-records rule (groupby-sort-filter-args B56-B59) would accept.
  Since T-3839 ours is `#VALUE!` (the one-value arm comes before the records-height arm); no oracle cell covers it.
- Edge 2: a detected header row (field_headers omitted, values whose first row is text over numbers) beside a scalar TRUE
  filter. T-3839's probe has no omitted-field_headers call that spills without the filter (G!B2:E25, F!B13:K14 are `#VALUE!`
  either way), so the "detected none keeps, detected header refuses" reading is inferred. The detected-none side rests on
  groupby-pivotby-followup3 G!A85.
- arrays.md, entry "GROUPBY scalar inputs, option order and scalar filters in the gen-m7 arg-pairs batch": the T-3839 bullet
  says the lifted field_headers' "blank elements give 0.3 (L!D1:E2)". L!E1 is the blank element and L!E2 the FALSE element
  (Data!B2), and both are 0.3, so a FALSE element of a lifted field_headers is evidenced. Fix the sentence and narrow the
  Status line that still lists that as UNVERIFIED.

## Acceptance criteria
- [ ] An excel-graph probe answers both edges; `Model::filter_rows` matches it (reorder the arms if the records-height rule wins).
- [ ] The same probe covers an error element of a lifted GROUPBY option beside a filter column, before and after the
      filter is spent (`GROUPBY(Data!C1:C12,Data!D1:D12,_xleta.SUM,0,1,,(Data!C1:C12<0.2),{0,1;NA(),1})` and `{NA(),0;0,1}`;
      T-3840's arrays.md entry "A lifted GROUPBY's filter serves one element"; ours: `#VALUE!` once spent, its own error
      before, and it spends nothing; #2913 review MINOR).
- [ ] arrays.md cites the new cells, fixes the L!D1:E2 sentence and its Status line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY's scalar filter (T-3841).
