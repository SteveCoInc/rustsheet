---
id: T-3890
title: "PIVOTBY: a scalar filter_array beside multi-column fields or a header row is #VALUE!"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
PIVOTBY returns `#VALUE!` where Excel does for a scalar `filter_array`, instead of spilling the table.

## Context
- Found by T-3841 (gen-m7 arg-pairs batch for PIVOTBY, excel-graph oracle, 2026-11-03). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
  PIVOTBY's arguments: row_fields, col_fields, values, function, field_headers, row_total_depth, row_sort_order,
  col_total_depth, col_sort_order, filter_array, relative_to.
- GROUPBY's rule (T-3839, arrays.md: a scalar `filter_array` keeps or drops every row only when `row_fields` and `values` are each
  one column wide and there is no header row; otherwise `#VALUE!`) looks to carry over: every owned cell has a scalar filter
  (0.5, -1, 1, 2, TRUE, `Data!G4`) beside a multi-column field or values, or field_headers 1 or 3, and Excel is `#VALUE!`
  where ours spills (the anchor reads `""`, ours the table's empty corner). Probe which of `row_fields`, `col_fields` and `values`
  must be one column wide (excel-probe skill; one key column each with a scalar TRUE filter) and record it in arrays.md.
- Examples:
  - `=_xlfn.PIVOTBY(Data!D1:D12,Data!A1:D12,Data!C1:E12,LAMBDA(x,123456789.123),2,0,-2,0,1,0.5,Data!A1:B2)` is `#VALUE!`, ours `""` (547d6192794dbf57 A11469)
  - `=_xlfn.PIVOTBY(Data!A1:D12,Data!D1:E12,Data!A1:A12,LAMBDA(z,z),2,-2,(-1),1,-2,TRUE,Data!A1:B2)` is `#VALUE!`, ours `""` (5883a41f98113db2 A4637)
  - `=_xlfn.PIVOTBY(Data!C1:E12,Data!C1:C12,Data!B1:E12,LAMBDA(x,x),3,1,-2,0,1,TRUE)` is `#VALUE!`, ours `""` (75038e7ef3522165 A3051)
  - `=_xlfn.PIVOTBY(Data!C1:D12,Data!A1:D12,Data!C1:D12,LAMBDA(v,v),0,-2,2,-1,2,-1)` is `#VALUE!`, ours `""` (75038e7ef3522165 A3661)
  - `=_xlfn.PIVOTBY(Data!D1:E12,Data!A1:D12,Data!A1:A12,LAMBDA(w,Data!E7),0,-2,1,2,-1,2)` is `#VALUE!`, ours `""` (75038e7ef3522165 A977)
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/pivotby.rs` (`fn_pivotby`), sharing `Model::filter_rows`
  and the GROUPBY helpers in `groupby.rs`.
- Owned cells (14 calls, 10060 failing cells; each span runs from the call's anchor over
  the failing cells of its spill or lift):
- gen-m7/547d6192794dbf57.case.json Calc!A10188:B10189, Calc!A11469
- gen-m7/5883a41f98113db2.case.json Calc!A4637
- gen-m7/75038e7ef3522165.case.json Calc!A977:X1002, Calc!A3051:AQ3065, Calc!A3661:AB3687
- gen-m7/9077d156f68bf703.case.json Calc!A1343:BW1370
- gen-m7/9c128bd4a5d1d3b9.case.json Calc!A3783:AI3797, Calc!A6101:AM6127, Calc!A6345:BR6361, Calc!A6467:CH6483
- gen-m7/a11a79d85df364b0.case.json Calc!A4393:AC4405
- gen-m7/e8a89caaafba1c89.case.json Calc!A8480:W8492
- gen-m7/f84158fac01b5cb5.case.json Calc!A7931:AO7955

## Acceptance criteria
- [x] The owned cells match Excel, with 0 spill-extent failures for their anchors; unit tests cover a scalar filter beside a two-column `row_fields`, a two-column `col_fields`, two `values` columns and a header row.
- [x] docs/excel-semantics/arrays.md records the PIVOTBY rule (its `Ours:` line) when it is implemented.
- [x] Visible parity: every owned cell has Excel's type and error.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY's other batch rules (T-3888, T-3889); GROUPBY.

## Outcome
- Probe probes/t3890-pivotby-scalar-filter (56 questions, excel-graph, 2026-11-03): GROUPBY's T-3839 rule with `col_fields`
  added. One filter value keeps or drops every row only beside one row field, one column field, one values column and no
  header row, given or detected. Otherwise it is `#VALUE!`, TRUE and 2 included, and error values still come first.
- `Model::pivotby_args` passes `one_keeps_all` to `Model::filter_rows`. A filter refusal fills the shape of an array
  `relative_to`, as Excel does over `Data!A1:B2`, through `Model::error_over`, which was factored out of GROUPBY's
  `Model::lifted_error` (no copy).
- Unit test `a_scalar_filter_needs_one_column_each_and_no_header` covers a two-column `row_fields`, a two-column
  `col_fields`, two values columns, a header row, a later error, and the filled relative_to shape. arrays.md's PIVOTBY
  batch bullet records the rule, the probe and the `Ours:` note.
- Eval delta: gen-m7 known failures 10607 -> 544 (10063 fixed: the 10060 owned cells plus T-3892's 410128dc65d03200
  A7931:B7932, a filter that keeps no row over a 2x2 relative_to; 0 regressions); probes +1 case (59 cells, all pass).
- Follow-ups filed with this PR: T-3910 (T-3889's probe citations), T-3911 (the eval harness's typed-entry order, from
  T-3898's review).
