---
id: T-3844
title: "GROUPBY and PIVOTBY return #VALUE! for a one-cell filter_array instead of broadcasting it (gen-m7 arg-pairs)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
GROUPBY and PIVOTBY return `#VALUE!` when `filter_array` is a single value (a number, a boolean, text, or a one-cell reference) instead of broadcasting it over the rows, as Excel does.

## Context
- Found by the T-3778 arg-pairs batch (gen-m7, excel-graph, 2026-11-02). On the Data sheet of these cases G1 is 7, G2 0, G3 -2.5, G4 1.5, G5 the text "12", G6 "x", G7 an empty text, G8 FALSE, G9 `#DIV/0!` (a constant error cell), and H1 and H2 are blank; A1:E12 is the data block.
- Excel, by the oracle sidecars:
  - `GROUPBY(Data!B1:B12,Data!B1:E12,_xleta.SUM,TRUE,-1,-2,0.5)`, `GROUPBY(Data!A1:A12,Data!B1:E12,LAMBDA(v,v),1,TRUE,-2,-1)`, `GROUPBY(Data!A1:D12,Data!C1:E12,LAMBDA(y,Data!C4),3,-2,Data!G1,Data!G1)` are `#VALUE!`.
  - `PIVOTBY(Data!C1:D12,Data!D1:E12,Data!B1:D12,_xleta.PRODUCT,3,-1,1,0,2,0.5)` and `PIVOTBY(...,TRUE)` are `#VALUE!`.
  - We broadcast the scalar and spill the whole table, so most cells here are spill members Excel leaves empty.
- A one-cell filter is a size mismatch against the 12 data rows, like any other wrong-height filter_array.
- Quirk recorded in docs/excel-semantics/arrays.md, "GROUPBY and PIVOTBY: what the T-3778 arg-pairs batch found".
- Cells (12173 in 43 formulas, each anchor's failing cells as a span; all owned here):
- gen-m7/06c84cc76de04bae: A245:E255, A2563:G2575, B3783:B3784
- gen-m7/07c0c130d894cfe0: A3661:F3672, A3722:G3742, A3905:E3916, A3966:G3979, A4027:C4036, A6223:G6235, A6467:E6478, A6589:D6610, A7077:D7098
- gen-m7/2e564b1fac9bc839: A1831:F1842, A2502
- gen-m7/3a4c7294ff5c4af3: A3722:F3734, A4088:E4099, A5125:E5136, A5735:E5745, A11591:E11599, A11652:F11663, A11774:G11784
- gen-m7/3db804a002b664b1: A4393:D4405, A4515:G4526, A5003:D5025, A12140:G12164
- gen-m7/53473f95ff58029d: A5308:D5316, A5979:F5990, A6284:G6296, A8968:E8989, A9090
- gen-m7/5883a41f98113db2: A6345
- gen-m7/709913a7469ca178: A5247
- gen-m7/75038e7ef3522165: A977:X1002, A3051:AQ3065, A3661:AB3687
- gen-m7/9077d156f68bf703: A1343:BW1370
- gen-m7/9c128bd4a5d1d3b9: A3783:AI3797, A6101:AM6127, A6345:BR6361, A6467:CH6483
- gen-m7/a11a79d85df364b0: A4393:AC4405
- gen-m7/e8a89caaafba1c89: A8480:W8492
- gen-m7/f84158fac01b5cb5: A7931:AO7955

## Visible-parity stage
Decision 0083: every cell listed under Context is visibly off (a wrong error type, a value where Excel has an error, or a spill where Excel has none), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [ ] Every cell listed under Context matches Excel (`make eval-show` on each case), with unit tests in the function's module for the rule.
- [ ] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other rules of the batch (T-3842, T-3843, T-3845, T-3846).
