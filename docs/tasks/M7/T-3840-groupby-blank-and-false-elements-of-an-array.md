---
id: T-3840
title: "GROUPBY: a FALSE element of an array field_relationship is #VALUE!, and an early error lifts over the array"
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
GROUPBY lifted over an array `field_relationship` gives Excel's `#VALUE!` for a FALSE element (a blank element stays valid), and an
error in `row_fields` or `values` fills the lifted shape as Excel does.

## Context
- Found by T-3778 (gen-m7 arg-pairs batch for GROUPBY, excel-graph oracle, 2026-11-02). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank. Data!A1:B2 is {"a*",<blank>;"?b",FALSE}; Data!C3:D4 is {"1",-2.5;0.1,-0.0001}.
- `Model::lift_options` (lifted_options.rs) and `Model::lifted_groupby` / `lifted_error` (groupby.rs) lift these options per element.
  The batch shows:
  - A FALSE element of a lifted field_relationship `Data!A1:B2` is `#VALUE!` (06c84cc76de04bae B5736, 7ab51243a366f57b B8054); ours
    evaluates it as 0. A blank element is valid (B5735 0.3, B8053 TRUE pass) and must stay so. A scalar
    FALSE field_relationship is 0 (probes/groupby-pivotby-followup G!A941), so this is lift-specific.
  - An error `row_fields` or `values` (`GROUPBY(#NULL!,..,Data!A1:B2)`, `GROUPBY(Data!D1:D12,#REF!,..,Data!A1:B2)`,
    `GROUPBY(Data!A1:C12,Data!G9,..,Data!C3:D4)`) fills the 2x2 lifted shape with that error in Excel; ours returns one cell.
- The early-error lift (45c197dc0b22cf2b A7565:B7566, eb01a9867300331b A2563:B2564, A2624:B2625) was done by T-3837, which routes
  the `row_fields`/`values` errors through `Model::lifted_error`; what is left is the FALSE element.
- Owned cells:
- gen-m7/06c84cc76de04bae.case.json Calc!B5736
- gen-m7/45c197dc0b22cf2b.case.json Calc!A7565:B7566
- gen-m7/53473f95ff58029d.case.json Calc!A9090
- gen-m7/7ab51243a366f57b.case.json Calc!B8054
- gen-m7/eb01a9867300331b.case.json Calc!A2563:B2564, Calc!A2624:B2625
- gen-m7/f310ddb6bc8ec33a.case.json Calc!B4150

## Acceptance criteria
- [x] The owned cells match Excel, spill extent included; unit tests cover a FALSE element (`#VALUE!`), a blank element (valid) and an early error under an array field_relationship.
- [x] Record the rule in docs/excel-semantics/arrays.md ("GROUPBY and PIVOTBY lift over an array field_headers or total depth").
- [x] Visible parity: every owned cell has Excel's type and error.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY (its arg-pairs batch is T-3841) and the other GROUPBY rules of the batch, owned by T-3837..T-3840.

## Outcome
- The FALSE element was never the cause. Three excel-graph probes (2026-11-03) show it: t3840-groupby-lifted-false-element,
  -lifted-beside-filter and -lifted-filter-consumed. When GROUPBY lifts over an array field_relationship, field_headers or
  total depth beside a filter column that drops rows, only the first element whose call filters gets a value. Every later
  element is `#VALUE!`, whatever it holds (0, 1, TRUE, FALSE, a blank). An element refused before the filter (text, a
  relationship of 2) leaves the filter to the next element. An out-of-range depth spends it. A filter that keeps every row,
  a scalar filter and no filter change nothing. Without a filter a FALSE element is 0. The rule is in
  docs/excel-semantics/arrays.md, "A lifted GROUPBY's filter serves one element (T-3840)", and the gen-m7 entry now points
  to it.
- `Model::groupby_filter_spent` is set by `Model::groupby_args` when its filter dropped rows. A GROUPBY inside the function
  doesn't count (`Model::groupby_with` restores the flag). `Model::lift_by_class` makes every element after it `#VALUE!` and
  restores the outer flag. groupby_args now reads field_relationship before the filter and checks the total depth's range
  after it, which is Excel's order here (both are `#VALUE!`, so no other cell changes). PIVOTBY's lift is unprobed
  (T-3841) and doesn't set the flag.
- The early-error lift (A7565:B7566, A2563:B2564, A2624:B2625) and 53473f95ff58029d A9090 already passed on main (T-3837).
- Unit test `a_lifted_groupby_filter_serves_one_element`.
- Eval delta: gen-m7 known failures 4 -> 1 (B5736, B8054, B4150 fixed; the one left, 3f368d034700f19e A2990, is T-3765's
  SCAN cell). 0 regressions. The three new probe cases pass in full.
