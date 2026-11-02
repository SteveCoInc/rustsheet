---
id: T-3838
title: "GROUPBY: a text or blank option is #VALUE! ahead of a later option's error, and a TRUE field_relationship's order"
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
GROUPBY reports `#VALUE!` where Excel does when a text, empty-text or blank option comes before an option holding an error value, and orders a `field_relationship` of TRUE as Excel does.

## Context
- Found by T-3778 (gen-m7 arg-pairs batch for GROUPBY, excel-graph oracle, 2026-11-02). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
- `Model::evaluate_options` returns the first error value among the options, ahead of any invalid one (probes/groupby-sort-filter-args
  B61-B78: `GROUPBY(k,v,SUM,,3,NA())` is `#N/A`). That holds for an invalid *number* (3, 4, a function 5, a height mismatch), but the
  batch shows a text or blank option before the error wins with `#VALUE!`:
  `GROUPBY(Data!A1:A12,Data!D1:E12,_xleta.AVERAGE,Data!G7,Data!G9)` (field_headers `""`), `GROUPBY(Data!A1:B12,Data!A1:B12,LAMBDA(x,x),1,Data!G6,Data!G9)`
  (total_depth "x"), `GROUPBY(Data!B1:D12,Data!B1:C12,LAMBDA(z,z),0,2,Data!B11:B11,..,#REF!)` (sort_order a blank cell): Excel `#VALUE!`,
  ours the later error. Hypothesis: options are coerced in argument order (text or blank fails at once with `#VALUE!`, an error is
  that error) and range checks come after; confirm with an `excel-probe` row per option kind (text, `""`, blank cell, FALSE) before
  a later `NA()`.
- 2e564b1fac9bc839 Calc!A4881:C4891: `GROUPBY(Data!B1:D12,Data!A1:D12,LAMBDA(z,z),0,0,-2,(Data!A1:A12<>0.2),TRUE)` puts the rows in
  another order than ours (we read TRUE as 1, a table; Excel's order looks like the hierarchy's or another sort). Probe
  `field_relationship` TRUE against 1 and 0 with `sort_order` -2.
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/groupby.rs` (`groupby_inputs`, `evaluate_options`, `groupby_args`).
- Owned cells:
- gen-m7/06c84cc76de04bae.case.json Calc!A10676, Calc!A12018
- gen-m7/2e564b1fac9bc839.case.json Calc!A4027, Calc!A4881:C4891, Calc!A9822, Calc!A11164
- gen-m7/3a4c7294ff5c4af3.case.json Calc!A9822, Calc!A11042
- gen-m7/3db804a002b664b1.case.json Calc!A7748, Calc!A10432, Calc!A11774
- gen-m7/53473f95ff58029d.case.json Calc!A4088
- gen-m7/65a4611a6242a59f.case.json Calc!A9578, Calc!A10920
- gen-m7/709913a7469ca178.case.json Calc!A5247, Calc!A5308
- gen-m7/eb01a9867300331b.case.json Calc!A9822, Calc!A11164
- gen-m7/f310ddb6bc8ec33a.case.json Calc!A8968, Calc!A10066

## Acceptance criteria
- [x] An excel-graph probe settles the option-coercion order and TRUE as field_relationship; the rule is in docs/excel-semantics/arrays.md.
- [x] The owned cells match Excel; unit tests cover each option kind before a later error.
- [x] Visible parity: every owned cell has Excel's type, error and value.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY (its arg-pairs batch is T-3841) and the other GROUPBY rules of the batch, owned by T-3837..T-3840.

## Outcome
Probe: probes/t3838-groupby-option-order (232 questions, excel-graph 2026-11-02): 13 option kinds (text, `""`, references
to text, `=""` and a blank cell, FALSE, TRUE, "1", 9, `{1}`) at each option, alone, before a later `NA()` and after an
earlier one, and `field_relationship` 1/TRUE/"1"/0/FALSE against `sort_order` scalars and arrays over two and three keys.
The hypothesis in Context was half right; the rules (arrays.md "GROUPBY and PIVOTBY", after B79-B83) are:
- `field_headers` and `total_depth` refuse text that is no number in place, ahead of a later error, whether it is written
  or comes from a one-cell reference (`""` too); ours already did this for written text only. `Model::lift_options` now
  reads a written reference's value too (lifted_options.rs).
- `sort_order` and `filter_array` refuse a one-cell reference to a blank cell in place (text there does not):
  `Model::evaluate_options` takes the option positions that refuse a blank (`blank_refused`; GROUPBY passes
  `sort_order` and `filter_array`, PIVOTBY and `function_elements` pass none, unprobed).
- A table relationship sorts by the keys `sort_order` names first, in written order, then the other keys ascending in
  column order. A descending key reverses the whole ascending order, so the blank key sorts first. A hierarchy keeps
  column order and the blank last. `SortOrder::Keys` now carries the named fields (`named`), and
  `Grouping::sort_table_by_keys` sorts a table (grouping.rs).
Unit tests: `options_are_refused_in_argument_order`, `a_table_sorts_by_the_named_keys_first` (test_fn_groupby.rs).
Every owned gen-m7 cell now matches Excel, A4881:C4891 included.
