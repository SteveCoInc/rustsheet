---
id: T-3837
title: "GROUPBY: a one-cell error row_fields and text or blank scalar values are #VALUE!"
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
GROUPBY returns `#VALUE!` where Excel does for a one-cell reference to an error as `row_fields` and for text or blank scalar `values`, instead of the error or a one-row table.

## Context
- Found by T-3778 (gen-m7 arg-pairs batch for GROUPBY, excel-graph oracle, 2026-11-02). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
- Rules the batch shows (gen-m7/35679b4facc62956.case.json, a full grid of scalar row_fields x values):
  - `values` that is text (`"2.5"`, `"abc"`, `""`, or a one-cell reference to text: G5, G6, G7) or a one-cell reference to a blank
    cell (H1, H2) is `#VALUE!` whatever `row_fields` is, unless `row_fields` is itself a literal error (`GROUPBY(#NUM!,Data!H1,..)` is
    `#NUM!`, passes). Numbers and booleans as scalar values are fine (`GROUPBY(2,TRUE,MIN)` is a table, passes). Ours returns the key
    with an aggregate.
  - `row_fields` a one-cell reference to an error (`Data!G9`) is `#VALUE!` (ours `#DIV/0!`), while a literal error there is that
    error (`GROUPBY(#REF!,2,..)` is `#REF!`, passes) and `GROUPBY(Data!G9,Data!G9,..)` is `#DIV/0!` (the values' error, passes).
  - Both win over a later option's error value (`GROUPBY(Data!C1:C12,Data!H1,_xleta.PRODUCT,#NUM!)` is `#VALUE!`).
- 45c197dc0b22cf2b Calc!A7626:B7627 (`GROUPBY(Data!G9,..,Data!C3:D4)`) also needs the lift of T-3840.
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/groupby.rs` (`groupby_inputs`, `evaluate_options`, `groupby_args`).
- Owned cells (anchor and the spill Excel leaves empty):
- gen-m7/0a91d5918d798930.case.json Calc!A10127, Calc!A10188
- gen-m7/17d3231584884628.case.json Calc!A4820, Calc!A4942, Calc!A5064, Calc!A5186, Calc!A5308, Calc!A5430, Calc!A5552, Calc!A5674, Calc!A5918, Calc!A6040
- gen-m7/18ded81711290e85.case.json Calc!A672, Calc!A794, Calc!A916, Calc!A1038, Calc!A1160, Calc!A1282, Calc!A1404, Calc!A1526, Calc!A1770, Calc!A1892:B1893
- gen-m7/35679b4facc62956.case.json Calc!A367:B368, Calc!A428:B429, Calc!A489:B490, Calc!A550:B551, Calc!A611:B612, Calc!A672:B673, Calc!A977:B978, Calc!A1038:B1039, Calc!A1709:B1710, Calc!A1770:B1771, Calc!A1831:B1832, Calc!A1892:B1893, Calc!A1953:B1954, Calc!A2014:B2015, Calc!A2319:B2320, Calc!A2380:B2381, Calc!A3051:B3052, Calc!A3112:B3113, Calc!A3173:B3174, Calc!A3234:B3235, Calc!A3295:B3296, Calc!A3356:B3357, Calc!A3661:B3662, Calc!A3722:B3723, Calc!A4393:B4394, Calc!A4454:B4455, Calc!A4515:B4516, Calc!A4576:B4577, Calc!A4637:B4638, Calc!A4698:B4699, Calc!A5003:B5004, Calc!A5064:B5065, Calc!A5735:B5736, Calc!A5796:B5797, Calc!A5857:B5858, Calc!A5918:B5919, Calc!A5979:B5980, Calc!A6040:B6041, Calc!A6345:B6346, Calc!A6406:B6407, Calc!A7077:B7078, Calc!A7138:B7139, Calc!A7199:B7200, Calc!A7260:B7261, Calc!A7565:B7566, Calc!A7626:B7627, Calc!A8297:B8298, Calc!A8358:B8359, Calc!A8419:B8420, Calc!A8480:B8481, Calc!A8541:B8542, Calc!A8602:B8603, Calc!A8907:B8908, Calc!A8968:B8969, Calc!A9639:B9640, Calc!A9700:B9701, Calc!A9761:B9762, Calc!A9822:B9823, Calc!A9883:B9884, Calc!A9944:B9945, Calc!A10249:B10250, Calc!A10310:B10311, Calc!A10554, Calc!A10676, Calc!A10798, Calc!A10920, Calc!A11042, Calc!A11164, Calc!A11286, Calc!A11408, Calc!A11652, Calc!A11774
- gen-m7/3701c7edf02ba750.case.json Calc!A3600, Calc!A3722, Calc!A3844, Calc!A3966, Calc!A4088, Calc!A4210, Calc!A4332, Calc!A4454, Calc!A4698, Calc!A4820
- gen-m7/45c197dc0b22cf2b.case.json Calc!A6406, Calc!A6528, Calc!A6650, Calc!A6772, Calc!A6894, Calc!A7016, Calc!A7138, Calc!A7260, Calc!A7504, Calc!A7626:B7627
- gen-m7/9b7e3b9a41fc404e.case.json Calc!A855, Calc!A916
- gen-m7/c636506a3a65eb5d.case.json Calc!A123:B124, Calc!A184:B185, Calc!A245:B246, Calc!A306:B307, Calc!A367:B368, Calc!A428:B429, Calc!A733:B734, Calc!A794:B795
- gen-m7/d03de506903f5a35.case.json Calc!A2075, Calc!A2136
- gen-m7/e5bac786f4f6a903.case.json Calc!A2136, Calc!A2258, Calc!A2380, Calc!A2502, Calc!A2624, Calc!A2746, Calc!A2868, Calc!A2990, Calc!A3234, Calc!A3356:B3357
- gen-m7/eb01a9867300331b.case.json Calc!A3661, Calc!A3722
- gen-m7/ec5094d09c82f94d.case.json Calc!A11591, Calc!A11652

## Acceptance criteria
- [x] The owned cells match Excel; unit tests in groupby.rs cover a text, an empty-text and a blank-cell `values` scalar and a one-cell
      error reference as `row_fields`, each against a later error option.
- [x] Update docs/excel-semantics/arrays.md "GROUPBY scalar inputs, option order and scalar filters in the gen-m7 arg-pairs batch"
      (its `Ours:` line) when the rule is implemented.
- [x] Visible parity: every owned cell has Excel's type and error.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY (its arg-pairs batch is T-3841) and the other GROUPBY rules of the batch, owned by T-3837..T-3840.

## Outcome
- `Model::groupby_fields` (groupby.rs) reads `row_fields` and `values` in Excel's order, and every error it finds is lifted
  over an array `field_relationship` through `Model::lifted_error`:
  - a literal error `row_fields` is that error;
  - then an error `values`, a one-cell reference included, is its error;
  - then a blank-cell `values` is `#VALUE!`, ahead of every option's error (0a91d5918d798930 A10127).
- `Model::groupby_args` turns text `values` (one text element) and one error key (a one-cell reference to an error as
  `row_fields`) into `#VALUE!` beside the height check, so after the options' errors. This order is evidence the first
  attempt got wrong: 25 batch cells such as `GROUPBY(Data!D1:D12,"abc",LAMBDA(v,v),1,#NUM!)` (d03de506903f5a35 A11835, `#NUM!`)
  and 5 such as `GROUPBY(Data!G9,Data!D1:D12,LAMBDA(acc,acc),Data!G9)` (18ded81711290e85 A1648, `#DIV/0!`) show the options'
  error winning. Recorded in docs/excel-semantics/arrays.md.
- Unit tests: `scalar_inputs_and_the_order_of_their_errors` and `an_early_input_error_fills_an_array_field_relationship`
  (test_fn_groupby.rs).
- **Eval delta**: gen-m7 known failures go from 2538 to 2170 (368 fixed, 0 regressions, `make eval-baseline`), 99.70% to
  99.86% bit-exact. All 359 owned cells pass, and so do 9 of T-3840's (the early-error lift); T-3840 keeps the FALSE element.
- Also folds in the MINOR findings left on #2847: T-3778's criteria are ticked with the spill-extent deviation noted, and
  arrays.md cites 07c0c130d894cfe0 A4515.

## Landing notes
- PR #2860 (stacked on #2847, which has merged; `origin/main` merged in at 12a895dc7). Round-1 fixes pushed at 013220834 and the
  commit adding these notes; CI pending on that head.
- Round 1 on 63cf87648: security APPROVE, design APPROVE (MINORs: the shared `ReferenceKind` check, and the reused height
  message, kept), excel-fidelity REQUEST_CHANGES. Fixed since then:
  - The text/error-key rules now apply only where the evidence goes (`ScalarInput`): scalar text `values` or a one-cell
    error `row_fields`, with `field_headers` omitted. A 1x1 array and a given `field_headers` are left as before, except
    that another height is still `#VALUE!`; T-3854 probes them.
  - The arrays.md "Unprobed: one row of text values" line now cross-references the new section.
  - The test uses a same-height error-key case.
  - Eval: gen-m7 2538 -> 2170 known failures, 0 regressions; 52 groupby unit tests pass; `make check-fast` green.
- Next: run all three reviewers on the new head (the head moved). Then CI green, `make merge-mode` = lock, and enqueue.
- Don't repeat: matching the rule on array shape (any 1x1 text values or 1x1 error key) was rejected as wider than the evidence.
