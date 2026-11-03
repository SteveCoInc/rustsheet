---
id: T-3892
title: "PIVOTBY: an early error fills the shape of an array relative_to or option it lifts over"
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
PIVOTBY fills the lifted shape with an early error, as Excel does, when it lifts over an array argument (a 2x2 `relative_to` such as `Data!A1:B2` or `Data!C3:D4`) and an earlier argument already makes the call an error; ours returns one error in the anchor and leaves the rest of the shape empty.

## Context
- Found by T-3841 (gen-m7 arg-pairs batch for PIVOTBY, excel-graph oracle, 2026-11-03). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank.
  PIVOTBY's arguments: row_fields, col_fields, values, function, field_headers, row_total_depth, row_sort_order,
  col_total_depth, col_sort_order, filter_array, relative_to.
- In every owned call Excel returns the same error in all four cells of the 2x2 lift (anchor, the cell below and the two to
  its right), and ours only in the anchor (anchors pass). The error comes from an earlier argument: a text or scalar
  `row_fields` (`"abc"`, `Data!G6`, `2`, `Data!G1`), an error option (`#N/A`, `#NULL!`, `#REF!`, `SQRT(-1)`, `Data!G9`).
  GROUPBY does this since T-3837 (`Model::lifted_error`; arrays.md "GROUPBY scalar inputs, ...": "An error `row_fields` or
  `values` fills the lifted shape"), and T-3840 covers its lift beside a filter.
- Examples: `=_xlfn.PIVOTBY("abc",Data!B1:E12,Data!A1:A12,LAMBDA(z,Data!A2),0,-2,0,2,Data!B11:E12,(Data!D1:D12>3.14159),Data!A1:B2)`, `=_xlfn.PIVOTBY(Data!G6,Data!A1:C12,Data!B1:D12,LAMBDA(x,x),2,2,0.5,2,Data!B11:D12,(Data!A1:A12<1e-15),Data!C3:D4)`, `=_xlfn.PIVOTBY(Data!A1:D12,Data!E1:E12,Data!B1:D12,LAMBDA(v,v),3,-2,Data!A2:A9,Data!G6,2,(Data!A1:A12<>100),Data!C3:D4)`.
- Check, with one probe row, whether PIVOTBY's lift beside a filter column spends the filter as GROUPBY's does (arrays.md
  "A lifted GROUPBY's filter serves one element", T-3840); the batch has no failing cell of that shape.
- Code: `crates/rustsheet-engine/src/functions/lookup_and_reference/pivotby.rs` (`fn_pivotby`), sharing `Model::filter_rows`
  and the GROUPBY helpers in `groupby.rs`.
- Owned cells (169 calls, 507 failing cells; each span runs from the call's anchor over
  the failing cells of its spill or lift):
- gen-m7/05cf579cc6560867.case.json Calc!A4149:B4150, Calc!A4210:B4211, Calc!A5491:B5492, Calc!A5552:B5553, Calc!A6833:B6834, Calc!A6894:B6895, Calc!A8175:B8176, Calc!A8236:B8237, Calc!A9517:B9518, Calc!A9578:B9579, Calc!A10859:B10860, Calc!A10920:B10921
- gen-m7/37041df35489e178.case.json Calc!A855:B856, Calc!A916:B917, Calc!A2197:B2198, Calc!A2258:B2259, Calc!A3539:B3540, Calc!A3600:B3601, Calc!A4881:B4882, Calc!A6223:B6224, Calc!A6284:B6285
- gen-m7/410128dc65d03200.case.json Calc!A123:B124, Calc!A184:B185, Calc!A1221:B1222, Calc!A1282:B1283, Calc!A2563:B2564, Calc!A2624:B2625, Calc!A3905:B3906, Calc!A3966:B3967, Calc!A5247:B5248, Calc!A5308:B5309, Calc!A6589:B6590, Calc!A6650:B6651, Calc!A7931:B7932, Calc!A7992:B7993, Calc!A9273:B9274, Calc!A9334:B9335
- gen-m7/42c026cbec22a61f.case.json Calc!A977:B978, Calc!A1038:B1039
- gen-m7/4ad4b1c6b2c2b01d.case.json Calc!A1:B2, Calc!A62:B63, Calc!A1343:B1344, Calc!A1404:B1405, Calc!A2685:B2686, Calc!A4027:B4028, Calc!A4088:B4089
- gen-m7/5239df16e9169bec.case.json Calc!A733:B734, Calc!A794:B795, Calc!A2075:B2076, Calc!A2136:B2137, Calc!A3417:B3418, Calc!A3478:B3479, Calc!A4759:B4760, Calc!A4820:B4821, Calc!A6101:B6102, Calc!A6162:B6163, Calc!A7443:B7444, Calc!A7504:B7505
- gen-m7/547d6192794dbf57.case.json Calc!A733:B734, Calc!A794:B795, Calc!A2075:B2076, Calc!A2136:B2137, Calc!A3417:B3418, Calc!A3478:B3479, Calc!A4759:B4760, Calc!A4820:B4821, Calc!A6101:B6102, Calc!A6162:B6163, Calc!A7443:B7444, Calc!A7504:B7505, Calc!A8785:B8786, Calc!A8846:B8847, Calc!A10127:B10128, Calc!A11530:B11531
- gen-m7/5883a41f98113db2.case.json Calc!A611:B612, Calc!A672:B673, Calc!A1953:B1954, Calc!A2014:B2015, Calc!A3295:B3296, Calc!A3356:B3357, Calc!A4698:B4699, Calc!A5979:B5980, Calc!A6040:B6041, Calc!A7687:B7688, Calc!A7748:B7749
- gen-m7/5c54b100d2f707be.case.json Calc!A6223:B6224, Calc!A6284:B6285, Calc!A7565:B7566, Calc!A7626:B7627, Calc!A8907:B8908, Calc!A8968:B8969, Calc!A10249:B10250, Calc!A10310:B10311, Calc!A11591:B11592, Calc!A11652:B11653
- gen-m7/63168c3f18718748.case.json Calc!A6345:B6346, Calc!A6406:B6407, Calc!A7687:B7688, Calc!A7748:B7749, Calc!A9029:B9030, Calc!A9090:B9091, Calc!A10371:B10372, Calc!A10432:B10433, Calc!A11713:B11714, Calc!A11774:B11775
- gen-m7/79b07511f8e77743.case.json Calc!A733:B734, Calc!A794:B795, Calc!A2075:B2076, Calc!A2136:B2137, Calc!A3417:B3418, Calc!A3478:B3479, Calc!A4759:B4760, Calc!A4820:B4821, Calc!A6101:B6102, Calc!A6162:B6163
- gen-m7/9c128bd4a5d1d3b9.case.json Calc!A245:B246, Calc!A306:B307, Calc!A1587:B1588, Calc!A1648:B1649
- gen-m7/a11a79d85df364b0.case.json Calc!A9395:B9396, Calc!A9456:B9457, Calc!A10493:B10494, Calc!A10554:B10555, Calc!A11713:B11714, Calc!A11774:B11775
- gen-m7/cb6c98a18513801b.case.json Calc!A1465:B1466, Calc!A1526:B1527, Calc!A2563:B2564, Calc!A2624:B2625, Calc!A3783:B3784, Calc!A3844:B3845, Calc!A5003:B5004, Calc!A5064:B5065, Calc!A6345:B6346, Calc!A6406:B6407, Calc!A7687:B7688, Calc!A7748:B7749, Calc!A9029:B9030, Calc!A9090:B9091, Calc!A10371:B10372, Calc!A10432:B10433, Calc!A11713:B11714, Calc!A11774:B11775
- gen-m7/ceb2c91035864a71.case.json Calc!A6589:B6590, Calc!A6650:B6651, Calc!A7931:B7932, Calc!A7992:B7993, Calc!A8907:B8908, Calc!A8968:B8969, Calc!A10249:B10250, Calc!A10310:B10311, Calc!A11591:B11592, Calc!A11652:B11653
- gen-m7/e8a89caaafba1c89.case.json Calc!A11347:B11348, Calc!A11408:B11409
- gen-m7/febfd6901374d79c.case.json Calc!A3661:B3662, Calc!A3722:B3723, Calc!A4759:B4760, Calc!A4820:B4821, Calc!A5857:B5858, Calc!A5918:B5919, Calc!A7199:B7200, Calc!A7260:B7261, Calc!A8541:B8542, Calc!A8602:B8603, Calc!A9761:B9762, Calc!A9822:B9823, Calc!A11103:B11104, Calc!A11164:B11165

## Acceptance criteria
- [ ] The owned cells match Excel; a unit test covers an early `#VALUE!` and an early error option lifted over a 2x2 `relative_to`.
- [ ] docs/excel-semantics/arrays.md records the PIVOTBY rule (its `Ours:` line) when it is implemented.
- [ ] Visible parity: every owned cell has Excel's type and error.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY's other batch rules (T-3888, T-3889, T-3890, T-3893); GROUPBY.
