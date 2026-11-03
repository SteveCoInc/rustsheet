---
id: T-3839
title: "GROUPBY: a scalar filter_array beside other options is #VALUE! (probe and fix)"
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
GROUPBY returns `#VALUE!` where Excel does for a scalar `filter_array` given beside other options, and keeps every row where Excel does.

## Context
- Found by T-3778 (gen-m7 arg-pairs batch for GROUPBY, excel-graph oracle, 2026-11-02). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
- Evidence so far: with every other option omitted a scalar filter keeps or drops all rows (probes/groupby-pivotby-followup3 A85 TRUE,
  A25 FALSE, A49 0; groupby-sort-filter-args B55: 2 keeps all), which `Model::filter_rows` implements. In the batch a scalar filter
  beside other options is `#VALUE!` in Excel: `GROUPBY(Data!A1:C12,Data!B1:C12,_xleta.MAX,1,-1,1,-1)`,
  `GROUPBY(Data!A1:A12,Data!D1:E12,LAMBDA(z,z),0,-1,2,0.5)`, `GROUPBY(Data!B1:D12,Data!D1:D12,LAMBDA(y,"ABC"),0,-2,2,TRUE)`,
  `GROUPBY(Data!A1:C12,Data!E1:E12,LAMBDA(v,Data!E8),0,2,-1,TRUE)`, while `GROUPBY(Data!C1:C12,Data!C1:C12,LAMBDA(x,x),2,1,2,TRUE)` and
  `GROUPBY(Data!E1:E12,Data!D1:D12,LAMBDA(acc,"Hello World"),2,1,2,2,Data!H1)` keep every row. Ours keeps every row in all of them.
  The deciding argument is unclear (field_headers 2 against 0/1/3, the filter value, -1 or 0.5 against TRUE or 2): probe it
  (`excel-probe`) with one option varied at a time, before changing `filter_rows`.
- 06c84cc76de04bae B3783:B3784 (`GROUPBY(Data!B1:C12,Data!B1:E12,LAMBDA(w,Data!E$6),Data!A1:B2,1,-2,TRUE)`, an array field_headers
  beside a scalar TRUE filter) are `#VALUE!` for the blank and FALSE header elements, though a blank lifted field_headers element is
  valid elsewhere (arrays.md A185): include this call in the probe.
- Several owned rows also pass FALSE as field_headers or total_depth (3db804a002b664b1 A12140, 3a4c7294ff5c4af3 A11652, A11774); the probe
  should cover FALSE there too.
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/groupby.rs` (`groupby_inputs`, `evaluate_options`, `groupby_args`).
- Owned cells (anchors and the tables we spill where Excel has one `#VALUE!`):
- gen-m7/06c84cc76de04bae.case.json Calc!A245:E255, Calc!A2563:G2575, Calc!B3783:B3784
- gen-m7/07c0c130d894cfe0.case.json Calc!A3661:F3672, Calc!A3722:G3742, Calc!A3905:E3916, Calc!A3966:G3979, Calc!A4027:C4036, Calc!A6223:G6235, Calc!A6467:E6478, Calc!A6589:D6610, Calc!A7077:D7098
- gen-m7/2e564b1fac9bc839.case.json Calc!A1831:F1842, Calc!A2502
- gen-m7/3a4c7294ff5c4af3.case.json Calc!A3722:F3734, Calc!A4088:E4099, Calc!A5125:E5136, Calc!A5735:E5745, Calc!A11591:E11599, Calc!A11652:F11663, Calc!A11774:G11784
- gen-m7/3db804a002b664b1.case.json Calc!A4393:D4405, Calc!A4515:G4526, Calc!A5003:D5025, Calc!A12140:G12164
- gen-m7/53473f95ff58029d.case.json Calc!A5308:D5316, Calc!A5979:F5990, Calc!A6284:G6296, Calc!A8968:E8989

## Acceptance criteria
- [x] An excel-graph probe states which scalar filters keep all rows beside which options; the rule is in docs/excel-semantics/arrays.md.
- [x] The owned cells match Excel; spill-extent failures of these anchors are 0.
- [x] Visible parity: every owned cell has Excel's type, error and value.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY (its arg-pairs batch is T-3841) and the other GROUPBY rules of the batch, owned by T-3837..T-3840.

## Outcome
- Probe probes/t3839-groupby-scalar-filter (879 cells, excel-graph, 2026-11-03): a scalar or 1x1 `filter_array` that is not as
  tall as the fields keeps or drops every row only beside one key column, one value column and no header row (field_headers 0
  or 2, or detected none); otherwise GROUPBY is `#VALUE!` whatever the filter holds. total_depth and sort_order do not matter
  (a 5x6x4 field_headers x total_depth x sort_order grid over two bases, with and without a TRUE filter), nor does the filter
  value (TRUE, 1, 2, 0.5, -1, 7, `{TRUE}`). The rule is in docs/excel-semantics/arrays.md ("GROUPBY scalar inputs, option order
  and scalar filters in the gen-m7 arg-pairs batch", and a pointer from the groupby-sort-filter-args paragraph).
- `Model::filter_rows` takes `one_keeps_all`; `Model::groupby_args` sets it from the key and value widths and the header row.
  PIVOTBY passes `true` (unchanged; its rule is T-3841's probe). A filter as tall as the fields wins over the one-value rule, so
  a one-row GROUPBY keeps its old behavior.
- Unit test `one_filter_value_keeps_all_rows_only_beside_one_key_one_value_and_no_header`.
- Eval delta: gen-m7 known failures 2119 -> 4 (2115 fixed, every owned anchor and its spilled table; the 4 left are T-3840's
  and others'), 0 regressions; the new probe case passes all its cells.
