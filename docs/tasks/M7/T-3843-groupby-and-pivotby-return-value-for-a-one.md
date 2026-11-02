---
id: T-3843
title: "GROUPBY and PIVOTBY return #VALUE! for a one-cell text, empty-text or blank values argument (gen-m7 arg-pairs)"
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
GROUPBY and PIVOTBY return `#VALUE!` when `values` is a single cell (a literal or a one-cell reference) holding text, numeric text, an empty text or nothing, as Excel does, instead of grouping it.

## Context
- Found by the T-3778 arg-pairs batch (gen-m7, excel-graph, 2026-11-02). On the Data sheet of these cases G1 is 7, G2 0, G3 -2.5, G4 1.5, G5 the text "12", G6 "x", G7 an empty text, G8 FALSE, G9 `#DIV/0!` (a constant error cell), and H1 and H2 are blank; A1:E12 is the data block.
- Excel, by the oracle sidecars:
  - `GROUPBY(0,"abc",LAMBDA(z,z))`, `GROUPBY(2,"2.5",_xleta.MIN)`, `GROUPBY("","",LAMBDA(v,v))`, `GROUPBY(Data!G2,Data!H2,_xleta.SUM)` and `GROUPBY(TRUE,Data!G7,...)` are `#VALUE!`, whatever the function (even a LAMBDA that ignores its argument).
  - With a number, zero, negative, fraction or boolean as the one-cell `values` the call still spills a table, and those cells pass: `GROUPBY(0,0.5,LAMBDA(x,x))` spills 0, `#VALUE!`; "Total", `#VALUE!` (gen-m7/35679b4facc62956 A1587:B1588), `GROUPBY("abc",TRUE,_xleta.AVERAGE)` spills "abc", `#DIV/0!`; "Total", `#DIV/0!` (A7321:B7322). The failing examples are A1831 and A2380 of the same case.
  - We return a table (or its first value) for the text and blank forms.
- Precedence against an error option differs by kind, and both halves must hold: a blank one-cell `values` gives `#VALUE!` before any option error, literal or a one-cell error reference (`GROUPBY(Data!C1:C12,Data!H1,_xleta.PRODUCT,#NUM!)`, gen-m7/0a91d5918d798930 A10127; `GROUPBY(Data!B1:B12,Data!H2,LAMBDA(z,Data!E8),Data!G9)`, A10188; 96ebcfc607f7eaa7 A245; ec5094d09c82f94d A11591; c97eaa8d33e9d1cf A9151), while a text, numeric-text or empty-text one loses to the option's error (`GROUPBY(Data!B1:E12,Data!G5,_xleta.COUNT,Data!G9)` is `#DIV/0!`, gen-m7/0a91d5918d798930 A3600; also A4759, ec5094d09c82f94d A6223 and A7565, b28fd0996d0a12cf A6406; these pass now and must keep passing). A literal error as a field beats both (`PIVOTBY(Data!C1:E12,#N/A,Data!H1,...)` is `#N/A`, gen-m7/18d737e85dbda437 A4027). The blank-values cells with a G9 option are listed in T-3842; this rule decides them.
- Whether a multi-cell `values` with no numeric cell also gives `#VALUE!` is not shown by this batch: probe it before generalising (excel-probe skill).
- Quirk recorded in docs/excel-semantics/arrays.md, "GROUPBY and PIVOTBY: what the T-3778 arg-pairs batch found".
- Cells (292 in 82 formulas, each anchor's failing cells as a span; all owned here):
- gen-m7/0098e1e4e5f8aa7c: A10737
- gen-m7/0a91d5918d798930: A10127
- gen-m7/35679b4facc62956: A367:B368, A428:B429, A489:B490, A550:B551, A611:B612, A672:B673, A977:B978, A1038:B1039, A1709:B1710, A1770:B1771, A1831:B1832, A1892:B1893, A1953:B1954, A2014:B2015, A2319:B2320, A2380:B2381, A3051:B3052, A3112:B3113, A3173:B3174, A3234:B3235, A3295:B3296, A3356:B3357, A3661:B3662, A3722:B3723, A4393:B4394, A4454:B4455, A4515:B4516, A4576:B4577, A4637:B4638, A4698:B4699, A5003:B5004, A5064:B5065, A5735:B5736, A5796:B5797, A5857:B5858, A5918:B5919, A5979:B5980, A6040:B6041, A6345:B6346, A6406:B6407, A7077:B7078, A7138:B7139, A7199:B7200, A7260:B7261, A7565:B7566, A7626:B7627, A8297:B8298, A8358:B8359, A8419:B8420, A8480:B8481, A8541:B8542, A8602:B8603, A8907:B8908, A8968:B8969, A9639:B9640, A9700:B9701, A9761:B9762, A9822:B9823, A9883:B9884, A9944:B9945, A10249:B10250, A10310:B10311
- gen-m7/44a1feb80a78aa24: A1038
- gen-m7/5c54b100d2f707be: A3661
- gen-m7/79b07511f8e77743: A5857
- gen-m7/96ebcfc607f7eaa7: A245
- gen-m7/9b7e3b9a41fc404e: A855
- gen-m7/a77b7647f69bcd5a: A7565
- gen-m7/c636506a3a65eb5d: A123:B124, A184:B185, A245:B246, A306:B307, A367:B368, A428:B429, A733:B734, A794:B795
- gen-m7/c97eaa8d33e9d1cf: A9151
- gen-m7/d03de506903f5a35: A2075
- gen-m7/eb01a9867300331b: A3661
- gen-m7/ec5094d09c82f94d: A11591

## Visible-parity stage
Decision 0083: every cell listed under Context is visibly off (a wrong error type, a value where Excel has an error, or a spill where Excel has none), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [ ] Every cell listed under Context matches Excel (`make eval-show` on each case), with unit tests in the function's module for the rule.
- [ ] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Multi-cell values; the other rules of the batch (T-3842, T-3844, T-3845, T-3846).
