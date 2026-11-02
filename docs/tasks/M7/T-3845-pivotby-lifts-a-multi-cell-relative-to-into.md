---
id: T-3845
title: "PIVOTBY lifts a multi-cell relative_to into one result per cell (gen-m7 arg-pairs)"
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
PIVOTBY given a multi-cell range as `relative_to` lifts over it, the way Excel does: it spills one result per cell of that range (a 2x2 of the call's error: `#VALUE!` here, since no cell holds a valid relative_to, or the error an argument gives, such as `#DIV/0!`, `#N/A`, `#NUM!`, `#NULL!`, `#REF!` or `#NAME?`), instead of returning one `#VALUE!`.

## Context
- Found by the T-3778 arg-pairs batch (gen-m7, excel-graph, 2026-11-02). On the Data sheet of these cases G1 is 7, G2 0, G3 -2.5, G4 1.5, G5 the text "12", G6 "x", G7 an empty text, G8 FALSE, G9 `#DIV/0!` (a constant error cell), and H1 and H2 are blank; A1:E12 is the data block.
- Excel, by the oracle sidecars:
  - `PIVOTBY(Data!D1:E12,Data!A1:D12,Data!A1:D12,LAMBDA(v,v),2,1,0,2,2,(Data!E1:E12=123456789.123),Data!A1:B2)` spills a 2x2 of `#VALUE!`; with `Data!C3:D4` the same.
  - We return a single `#VALUE!`, so the three other cells of the 2x2 are empty.
- The other option arguments already lift this way where the engine has them as ranges (those cells pass).
- Quirk recorded in docs/excel-semantics/arrays.md, "GROUPBY and PIVOTBY: what the T-3778 arg-pairs batch found".
- Cells (493 in 166 formulas, each anchor's failing cells as a span; all owned here):
- gen-m7/05cf579cc6560867: A4149:B4150, A4210:B4211, A5491:B5492, A5552:B5553, A6833:B6834, A6894:B6895, A8175:B8176, A8236:B8237, A9517:B9518, A9578:B9579, A10859:B10860, A10920:B10921
- gen-m7/37041df35489e178: A855:B856, A916:B917, A2197:B2198, A2258:B2259, A3539:B3540, A3600:B3601, A4881:B4882, A6223:B6224, A6284:B6285
- gen-m7/410128dc65d03200: A123:B124, A184:B185, A1221:B1222, A1282:B1283, A2563:B2564, A2624:B2625, A3905:B3906, A3966:B3967, A5247:B5248, A5308:B5309, A6589:B6590, A6650:B6651, A7931:B7932, A7992:B7993, A9273:B9274
- gen-m7/42c026cbec22a61f: A977:B978, A1038:B1039
- gen-m7/4ad4b1c6b2c2b01d: A1:B2, A62:B63, A1343:B1344, A1404:B1405, A2685:B2686, A4027:B4028, A4088:B4089
- gen-m7/5239df16e9169bec: A733:B734, A794:B795, A2075:B2076, A2136:B2137, A3417:B3418, A3478:B3479, A4759:B4760, A4820:B4821, A6101:B6102, A6162:B6163, A7443:B7444
- gen-m7/547d6192794dbf57: A733:B734, A794:B795, A2075:B2076, A2136:B2137, A3417:B3418, A3478:B3479, A4759:B4760, A6101:B6102, A6162:B6163, A7443:B7444, A7504:B7505, A8785:B8786, A8846:B8847, A10127:B10128, A10189:B10189, A11469, A11530:B11531
- gen-m7/5883a41f98113db2: A611:B612, A672:B673, A1953:B1954, A2014:B2015, A3295:B3296, A3356:B3357, A4637, A4698:B4699, A5979:B5980, A7687:B7688, A7748:B7749
- gen-m7/5c54b100d2f707be: A6223:B6224, A6284:B6285, A7565:B7566, A7626:B7627, A8907:B8908, A8968:B8969, A10249:B10250, A10310:B10311, A11591:B11592, A11652:B11653
- gen-m7/63168c3f18718748: A6345:B6346, A6406:B6407, A7687:B7688, A7748:B7749, A9029:B9030, A9090:B9091, A10371:B10372, A10432:B10433, A11713:B11714, A11774:B11775
- gen-m7/79b07511f8e77743: A733:B734, A794:B795, A2075:B2076, A2136:B2137, A3417:B3418, A3478:B3479, A4759:B4760, A4820:B4821, A6101:B6102, A6162:B6163
- gen-m7/9c128bd4a5d1d3b9: A245:B246, A306:B307, A1587:B1588
- gen-m7/a11a79d85df364b0: A9395:B9396, A9456:B9457, A10493:B10494, A10554:B10555, A11713:B11714, A11774:B11775
- gen-m7/cb6c98a18513801b: A1465:B1466, A1526:B1527, A2563:B2564, A2624:B2625, A3783:B3784, A3844:B3845, A5003:B5004, A5064:B5065, A6345:B6346, A6406:B6407, A7687:B7688, A7748:B7749, A9029:B9030, A9090:B9091, A10371:B10372, A10432:B10433, A11713:B11714
- gen-m7/ceb2c91035864a71: A6589:B6590, A6650:B6651, A7931:B7932, A7992:B7993, A8907:B8908, A8968:B8969, A10249:B10250, A10310:B10311, A11591:B11592, A11652:B11653
- gen-m7/e8a89caaafba1c89: A11347:B11348, A11408:B11409
- gen-m7/febfd6901374d79c: A3661:B3662, A3722:B3723, A4759:B4760, A4820:B4821, A5857:B5858, A5918:B5919, A7199:B7200, A7260:B7261, A8541:B8542, A8602:B8603, A9761:B9762, A9822:B9823, A11103:B11104, A11164:B11165

## Visible-parity stage
Decision 0083: every cell listed under Context is visibly off (a wrong error type, a value where Excel has an error, or a spill where Excel has none), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [ ] Every cell listed under Context matches Excel (`make eval-show` on each case), with unit tests in the function's module for the rule.
- [ ] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other rules of the batch (T-3842, T-3843, T-3844, T-3846).
