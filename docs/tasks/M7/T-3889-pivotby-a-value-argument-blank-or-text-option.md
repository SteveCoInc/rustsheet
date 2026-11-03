---
id: T-3889
title: "PIVOTBY: a #VALUE! argument (blank or text option, mismatched fields) wins over a later option's error"
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
PIVOTBY returns `#VALUE!` where Excel does when an earlier argument is invalid (a blank or text option, a scalar or short `row_fields`/`col_fields`, `""` as a field) and a later option holds an error, instead of that error.

## Context
- Found by T-3841 (gen-m7 arg-pairs batch for PIVOTBY, excel-graph oracle, 2026-11-03). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
  PIVOTBY's arguments: row_fields, col_fields, values, function, field_headers, row_total_depth, row_sort_order,
  col_total_depth, col_sort_order, filter_array, relative_to.
- Excel reads PIVOTBY's arguments so that the invalid argument's `#VALUE!` wins over a later option's error value; ours returns the later error.
  As in GROUPBY (T-3838: a text, empty-text or blank option is `#VALUE!` ahead of a later option's error). Shapes seen:
  a blank-cell option (`Data!H1`, `Data!H2`) as row/col sort order or total depth; a text option; a scalar `row_fields`
  (`2`, `Data!G4`) or a short `col_fields` (`Data!A1:B2`) beside 12-row fields; `""` as `col_fields`. Probe one row per shape
  before choosing the order (excel-probe skill), and record it in arrays.md.
- Examples:
  - `=_xlfn.PIVOTBY(2,Data!D1:D12,Data!A1:B12,_xleta.MAX,2,-1,Data!C$5,1,Data!A7:A10,(Data!D1:D12<>1),#NUM!)` is `#VALUE!`, ours `#NUM!` (05cf579cc6560867 A3905)
  - `=_xlfn.PIVOTBY(Data!G4,Data!A1:C12,Data!A1:D12,LAMBDA(v,Data!A$10),0,-1,Data!C4:E6,-1,Data!E1:E1,(Data!C1:C12<>1e-15),Data!G9)` is `#VALUE!`, ours `#DIV/0!` (05cf579cc6560867 A7992)
  - `=_xlfn.PIVOTBY(Data!A1:D12,Data!B1:E12,Data!A1:D12,LAMBDA(x,x),3,-1,2,-1,Data!C5,#REF!)` is `#VALUE!`, ours `#REF!` (0dac515958ce4371 A11713)
  - `=_xlfn.PIVOTBY(Data!C1:E12,Data!D1:E12,Data!B1:C12,LAMBDA(w,w),1,2,Data!H2,2,Data!G9)` is `#VALUE!`, ours `#DIV/0!` (0dac515958ce4371 A9944)
  - `=_xlfn.PIVOTBY(Data!A1:B12,Data!A1:B2,Data!B1:B12,LAMBDA(w,w),1,1,Data!B1:C10,2,Data!E1,(Data!A1:A12=1e-15),#NUM!)` is `#VALUE!`, ours `#NUM!` (37041df35489e178 A6589)
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/pivotby.rs` (`fn_pivotby`), sharing `Model::filter_rows`
  and the GROUPBY helpers in `groupby.rs`.
- Owned cells (24 calls, 24 failing cells; each span runs from the call's anchor over
  the failing cells of its spill or lift):
- gen-m7/05cf579cc6560867.case.json Calc!A3905, Calc!A7992
- gen-m7/0dac515958ce4371.case.json Calc!A9944, Calc!A11713
- gen-m7/223090e4684153e5.case.json Calc!A8907
- gen-m7/37041df35489e178.case.json Calc!A6589
- gen-m7/410128dc65d03200.case.json Calc!A9639
- gen-m7/44a1feb80a78aa24.case.json Calc!A1038
- gen-m7/547d6192794dbf57.case.json Calc!A4576, Calc!A5125, Calc!A5186
- gen-m7/5883a41f98113db2.case.json Calc!A6345, Calc!A6406
- gen-m7/5c54b100d2f707be.case.json Calc!A3661
- gen-m7/88f592c463d2ffb0.case.json Calc!A7138 (a text `values` before the blank `row_sort_order`; moved from T-3888)
- gen-m7/7ab307dba9cb2062.case.json Calc!A9883, Calc!A9944
- gen-m7/9077d156f68bf703.case.json Calc!A11225, Calc!A11286
- gen-m7/cb6c98a18513801b.case.json Calc!A12079, Calc!A12140
- gen-m7/ceb2c91035864a71.case.json Calc!A4027, Calc!A4088
- gen-m7/d7b4849a4fbe3485.case.json Calc!A8541

## Acceptance criteria
- [ ] The owned cells match Excel; unit tests cover a blank option, a text option and a short field, each against a later error option.
- [ ] docs/excel-semantics/arrays.md records the PIVOTBY rule (its `Ours:` line) when it is implemented.
- [ ] Visible parity: every owned cell has Excel's type and error.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY's other batch rules (T-3888, T-3890); GROUPBY.
