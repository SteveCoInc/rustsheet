---
id: T-3888
title: "PIVOTBY: a one-cell error row_fields or col_fields and text or blank scalar values are #VALUE!"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
PIVOTBY returns `#VALUE!` where Excel does for a one-cell reference to an error as `row_fields` or `col_fields`, and for text or blank-cell scalar `values`, ahead of a later option's error, as GROUPBY does since T-3837.

## Context
- Found by T-3841 (gen-m7 arg-pairs batch for PIVOTBY, excel-graph oracle, 2026-11-03). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
  PIVOTBY's arguments: row_fields, col_fields, values, function, field_headers, row_total_depth, row_sort_order,
  col_total_depth, col_sort_order, filter_array, relative_to.
- Rules the batch shows (the GROUPBY rules of T-3837 carry over, with `col_fields` beside `row_fields`):
  - `row_fields` or `col_fields` a one-cell reference to an error (`Data!G9`) is `#VALUE!` (ours `#DIV/0!`), whatever the options hold.
    181 cells.
  - `values` a one-cell reference to a blank cell (H1, H2) or to text (G6) is `#VALUE!` ahead of a later option's error.
    13 cells.
- Examples:
  - `=_xlfn.PIVOTBY(Data!G9,Data!A1:B12,Data!G3,LAMBDA(y,0.3))` is `#VALUE!`, ours `#DIV/0!` (01d099c3b6d6ae95 A1038)
  - `=_xlfn.PIVOTBY(Data!G9,Data!A1:B12,Data!G4,LAMBDA(x,x))` is `#VALUE!`, ours `#DIV/0!` (01d099c3b6d6ae95 A1160)
  - `=_xlfn.PIVOTBY(Data!G9,Data!B1:C12,Data!G5,LAMBDA(v,v))` is `#VALUE!`, ours `#DIV/0!` (01d099c3b6d6ae95 A1282)
  - `=_xlfn.PIVOTBY(Data!A1:D12,Data!C1:C12,Data!H1,_xleta.MIN,1,-2,#NUM!)` is `#VALUE!`, ours `#NUM!` (0098e1e4e5f8aa7c A10737)
  - `=_xlfn.PIVOTBY(Data!A1:D12,Data!B1:D12,Data!H2,_xleta.AVERAGE,1,1,Data!G9)` is `#VALUE!`, ours `#DIV/0!` (0098e1e4e5f8aa7c A10798)
  - `=_xlfn.PIVOTBY(Data!D1:D12,Data!C1:E12,Data!H2,LAMBDA(v,Data!C3),1,0,Data!C3:E11,0,Data!G9)` is `#VALUE!`, ours `#DIV/0!` (44a1feb80a78aa24 A1892)
- Check GROUPBY's order evidence (docs/excel-semantics/arrays.md "GROUPBY scalar inputs, option order and scalar filters in the
  gen-m7 arg-pairs batch": a text `values` and an error-key `row_fields` lose to an earlier-read option's error there) against
  these cells before reusing `Model::groupby_fields` / `Model::groupby_args`: here the error-reference field wins over every option.
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/pivotby.rs` (`fn_pivotby`), sharing `Model::filter_rows`
  and the GROUPBY helpers in `groupby.rs`.
- Owned cells (194 calls, 218 failing cells; each span runs from the call's anchor over
  the failing cells of its spill or lift):
- gen-m7/0098e1e4e5f8aa7c.case.json Calc!A10737, Calc!A10798
- gen-m7/01d099c3b6d6ae95.case.json Calc!A794, Calc!A916, Calc!A1038, Calc!A1160, Calc!A1282, Calc!A1404, Calc!A1526, Calc!A1648, Calc!A1892, Calc!A2014
- gen-m7/02f825428f6d3da3.case.json Calc!A3234, Calc!A3356, Calc!A3478, Calc!A3600, Calc!A3722, Calc!A3844, Calc!A3966, Calc!A4088, Calc!A4332, Calc!A4454:B4455
- gen-m7/0486b869a0305fdb.case.json Calc!A5430, Calc!A5552, Calc!A5674, Calc!A5796, Calc!A5918, Calc!A6040, Calc!A6162, Calc!A6284, Calc!A6528, Calc!A6650:B6651
- gen-m7/05cf579cc6560867.case.json Calc!A62, Calc!A184, Calc!A428, Calc!A550
- gen-m7/1561268b29d149e7.case.json Calc!A11530, Calc!A11652, Calc!A11774, Calc!A11896, Calc!A12018, Calc!A12140
- gen-m7/18d737e85dbda437.case.json Calc!A2990, Calc!A3112, Calc!A3234, Calc!A3356, Calc!A3478, Calc!A3600, Calc!A3722, Calc!A3844, Calc!A4088, Calc!A4210
- gen-m7/24bc4a707df18160.case.json Calc!A916, Calc!A2258, Calc!A3600, Calc!A4942, Calc!A6284, Calc!A7626, Calc!A8968, Calc!A10310, Calc!A10676, Calc!A10798, Calc!A10920, Calc!A11042, Calc!A11164, Calc!A11286, Calc!A11408, Calc!A11530, Calc!A11652, Calc!A11774, Calc!A11896
- gen-m7/29f8e28340bf4440.case.json Calc!A9700, Calc!A9822, Calc!A9944, Calc!A10066, Calc!A10188, Calc!A10310, Calc!A10432, Calc!A10554, Calc!A10798, Calc!A10920
- gen-m7/37041df35489e178.case.json Calc!A3722, Calc!A3844, Calc!A3966, Calc!A4088, Calc!A4210, Calc!A4332, Calc!A4454, Calc!A4576, Calc!A4820, Calc!A4942:B4943
- gen-m7/44a1feb80a78aa24.case.json Calc!A1892
- gen-m7/4ad4b1c6b2c2b01d.case.json Calc!A1526, Calc!A1648, Calc!A1770, Calc!A1892, Calc!A2014, Calc!A2136, Calc!A2258, Calc!A2380, Calc!A2624, Calc!A2746:B2747
- gen-m7/542087608aa65000.case.json Calc!A6406, Calc!A6528, Calc!A6650, Calc!A6772, Calc!A6894, Calc!A7016, Calc!A7138, Calc!A7260, Calc!A7504, Calc!A7626
- gen-m7/5c54b100d2f707be.case.json Calc!A3722
- gen-m7/63168c3f18718748.case.json Calc!A1526, Calc!A1648, Calc!A1770, Calc!A1892, Calc!A2014, Calc!A2136, Calc!A2258, Calc!A2380, Calc!A2624, Calc!A2746
- gen-m7/66bad08e2fd0f718.case.json Calc!A7016, Calc!A7138, Calc!A7260, Calc!A7382, Calc!A7504, Calc!A7626, Calc!A7748, Calc!A7870, Calc!A8114, Calc!A8236:B8237
- gen-m7/78330db25d7a3822.case.json Calc!A794, Calc!A2136
- gen-m7/79b07511f8e77743.case.json Calc!A5857, Calc!A5918
- gen-m7/8145169c978856f2.case.json Calc!A8602, Calc!A8724, Calc!A8846, Calc!A8968, Calc!A9090, Calc!A9212, Calc!A9334, Calc!A9456, Calc!A9700, Calc!A9822
- gen-m7/88f592c463d2ffb0.case.json Calc!A7138
- gen-m7/96ebcfc607f7eaa7.case.json Calc!A245, Calc!A306
- gen-m7/a77b7647f69bcd5a.case.json Calc!A7565, Calc!A7626
- gen-m7/b4486fce63db71a2.case.json Calc!A11896, Calc!A12018, Calc!A12140
- gen-m7/c97eaa8d33e9d1cf.case.json Calc!A9151, Calc!A9212
- gen-m7/d7232ffc680e4bac.case.json Calc!A8114, Calc!A8236, Calc!A8358, Calc!A8480, Calc!A8602, Calc!A8724, Calc!A8846, Calc!A8968, Calc!A9212, Calc!A9334:B9335
- gen-m7/d7b4849a4fbe3485.case.json Calc!A10310, Calc!A10432, Calc!A10554, Calc!A10676, Calc!A10798, Calc!A10920, Calc!A11042, Calc!A11164, Calc!A11408, Calc!A11530:B11531
- gen-m7/e4053a4410330089.case.json Calc!A4820, Calc!A4942, Calc!A5064, Calc!A5186, Calc!A5308, Calc!A5430, Calc!A5552, Calc!A5674, Calc!A5918, Calc!A6040:B6041
- gen-m7/fbb32b6c948cc84e.case.json Calc!A62, Calc!A184, Calc!A306, Calc!A428, Calc!A550, Calc!A794, Calc!A916

## Acceptance criteria
- [ ] The owned cells match Excel; unit tests cover an error-reference `row_fields`, an error-reference `col_fields`, and blank-cell and text `values`, each against a later error option.
- [ ] docs/excel-semantics/arrays.md records the PIVOTBY rule (its `Ours:` line) when it is implemented.
- [ ] Visible parity: every owned cell has Excel's type and error.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY's other batch rules (T-3889, T-3890); GROUPBY.
