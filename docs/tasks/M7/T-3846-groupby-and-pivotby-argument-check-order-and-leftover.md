---
id: T-3846
title: "GROUPBY and PIVOTBY argument-check order and leftover cells of the gen-m7 arg-pairs batch"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
The leftover GROUPBY and PIVOTBY cells of the T-3778 arg-pairs batch match Excel: chiefly which bad argument wins when a late option is a literal error and an earlier option is also invalid.

## Context
- Found by the T-3778 arg-pairs batch (gen-m7, excel-graph, 2026-11-02). On the Data sheet of these cases G1 is 7, G2 0, G3 -2.5, G4 1.5, G5 the text "12", G6 "x", G7 an empty text, G8 FALSE, G9 `#DIV/0!` (a constant error cell), and H1 and H2 are blank; A1:E12 is the data block.
- Excel, by the oracle sidecars:
  - `PIVOTBY(Data!A1:C12,Data!B1:E12,Data!C1:D12,LAMBDA(w,w),1,-1,Data!H1,#REF!)` is `#VALUE!` (we give `#REF!`): a blank row_sort_order seems to be checked before the col_total_depth error, while `PIVOTBY(...,0,2,#NULL!)` is `#NULL!`.
  - `GROUPBY(Data!B1:D12,Data!B1:C12,LAMBDA(z,z),0,2,Data!B11:B11,(...),#REF!)` is `#VALUE!` (we give `#REF!`).
  - `GROUPBY(...,Data!A1:B2)` as field_relationship spills Excel errors in the body where we give values (`B8054`, `B5736`, `B4150`).
  - `PIVOTBY(Data!E1:E12,Data!A1:C12,Data!B1:B12,LAMBDA(x," x "),...)`: a header text differs (`"?b, 3.14159"` against our `", ?b, 3.14159"`).
- Work out each rule from the cells (excel-probe skill for any that need more evidence), fix what is one rule, and file a task for any other rule.
- Record each rule in docs/excel-semantics/arrays.md.
- Cells (68 in 21 formulas, each anchor's failing cells as a span; all owned here):
- gen-m7/05cf579cc6560867: A3905
- gen-m7/06c84cc76de04bae: B5736
- gen-m7/0dac515958ce4371: A11713
- gen-m7/223090e4684153e5: A8907
- gen-m7/2e564b1fac9bc839: A4027, A4881:C4891
- gen-m7/37041df35489e178: A6589
- gen-m7/410128dc65d03200: A9639
- gen-m7/45c197dc0b22cf2b: A7565:B7566
- gen-m7/547d6192794dbf57: A5125
- gen-m7/7ab307dba9cb2062: A9883
- gen-m7/7ab51243a366f57b: B8054
- gen-m7/9077d156f68bf703: A11225
- gen-m7/cb6c98a18513801b: A12079
- gen-m7/ceb2c91035864a71: A4027, B8785
- gen-m7/d7b4849a4fbe3485: A8541
- gen-m7/eb01a9867300331b: A2563:B2564, A2624:B2625
- gen-m7/f310ddb6bc8ec33a: B4150
- gen-m7/febfd6901374d79c: A11230:F11232

## Visible-parity stage
Decision 0083: every cell listed under Context is visibly off (a wrong error type, a value where Excel has an error, or a spill where Excel has none), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [ ] Every cell listed under Context matches Excel (`make eval-show` on each case), with unit tests in the function's module for the rule.
- [ ] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The rules of T-3842, T-3843, T-3844 and T-3845.
