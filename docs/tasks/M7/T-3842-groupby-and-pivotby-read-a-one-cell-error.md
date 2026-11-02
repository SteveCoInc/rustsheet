---
id: T-3842
title: "GROUPBY and PIVOTBY give #VALUE! for a one-cell error reference as row_fields or col_fields once values and the options pass (gen-m7 arg-pairs)"
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
GROUPBY and PIVOTBY match Excel when `row_fields` or `col_fields` is a one-cell reference to an error cell: once no error
in `values` or an option argument has been returned, the call is `#VALUE!`, not the referenced cell's error. Where an
earlier option is invalid, that option's `#VALUE!` wins, as Excel's argument order has it.

## Context
- Found by the T-3778 arg-pairs batch (gen-m7, excel-graph, 2026-11-02). On the Data sheet of these cases G1 is 7, G2 0, G3 -2.5, G4 1.5, G5 the text "12", G6 "x", G7 an empty text, G8 FALSE, G9 `#DIV/0!` (a constant error cell), and H1 and H2 are blank; A1:E12 is the data block.
- Excel, by the oracle sidecars:
  - `GROUPBY(Data!G9,Data!G1,_xleta.PRODUCT)` is `#VALUE!` though both arguments are 1x1 (gen-m7/35679b4facc62956 A10554; also A10676 to A11652 with G2..G8 and H2 as values), and so are `GROUPBY(Data!G9,Data!B1:B12,_xleta.COUNT,Data!G4)` (gen-m7/18ded81711290e85 A1038), `PIVOTBY(Data!G9,Data!A1:B12,Data!G3,LAMBDA(y,0.3))` (gen-m7/01d099c3b6d6ae95 A1038) and `PIVOTBY(Data!B1:E12,Data!G9,Data!B1:B12,...)` (gen-m7/66bad08e2fd0f718 A8114). We return `#DIV/0!`.
  - An error in `values` or in an option is returned first: `GROUPBY(Data!G9,Data!G9,LAMBDA(z,z))` is `#DIV/0!` (gen-m7/35679b4facc62956 A11530), and a one-cell error reference as an option is itself returned (`GROUPBY(Data!G9,Data!D1:D12,LAMBDA(acc,acc),Data!G9)` is `#DIV/0!`, gen-m7/18ded81711290e85 A1648; `PIVOTBY(Data!G9,...,1,Data!G9)`, gen-m7/e4053a4410330089 A5796). These already pass and must keep passing.
  - A G9 option that loses to an earlier invalid option is `#VALUE!` where we give `#DIV/0!`: `PIVOTBY(Data!B1:E12,Data!C1:C12,Data!C1:D12,_xleta.AVERAGE,0,Data!G6,Data!B5:C6,0,Data!G9)` has row_total_depth "x" (gen-m7/cf6ab0ac7d65a5bb A2014); likewise gen-m7/08121bee6b350ca7 A672 (field_headers "") and gen-m7/06c84cc76de04bae A10676 (field_headers "x"). Those cells are listed below; their rule is the argument order of T-3846, so take them with it or after it.
  - A G9 option beside a blank one-cell `values` is `#VALUE!` too, even as the first option: `GROUPBY(Data!B1:B12,Data!H2,LAMBDA(z,Data!E8),Data!G9)` (gen-m7/0a91d5918d798930 A10188), `GROUPBY(Data!A1:B12,Data!H2,_xleta.MIN,0,Data!G9)` (gen-m7/ec5094d09c82f94d A11652), and 9b7e3b9a41fc404e A916, c97eaa8d33e9d1cf A9212, a77b7647f69bcd5a A7626, eb01a9867300331b A3722, 0098e1e4e5f8aa7c A10798. That is T-3843's precedence rule (blank values before option errors); take these cells with T-3843 or after it.
- A literal error (`#NUM!`, `#N/A`) as a field is returned as itself; that already passes.
- Quirk recorded in docs/excel-semantics/arrays.md, "GROUPBY and PIVOTBY: what the T-3778 arg-pairs batch found". Where the field read needs a 1x1-array wrapper, reuse `one_cell_error_ref_as_array` (functions/spill_functions.rs, T-3506) rather than a new helper.
- Cells (359 in 314 formulas, each anchor's failing cells as a span; all owned here):
- gen-m7/0098e1e4e5f8aa7c: A10798
- gen-m7/01d099c3b6d6ae95: A794, A916, A1038, A1160, A1282, A1404, A1526, A1648, A1892, A2014
- gen-m7/02f825428f6d3da3: A3234, A3356, A3478, A3600, A3722, A3844, A3966, A4088, A4332, A4454:B4455
- gen-m7/0486b869a0305fdb: A5430, A5552, A5674, A5796, A5918, A6040, A6162, A6284, A6528, A6650:B6651
- gen-m7/05cf579cc6560867: A62, A184, A428, A550, A7992
- gen-m7/06c84cc76de04bae: A10676, A12018
- gen-m7/08121bee6b350ca7: A672, A12018
- gen-m7/0a91d5918d798930: A10188
- gen-m7/0dac515958ce4371: A9944
- gen-m7/1561268b29d149e7: A11530, A11652, A11774, A11896, A12018, A12140
- gen-m7/17d3231584884628: A4820, A4942, A5064, A5186, A5308, A5430, A5552, A5674, A5918, A6040
- gen-m7/18d737e85dbda437: A2990, A3112, A3234, A3356, A3478, A3600, A3722, A3844, A4088, A4210
- gen-m7/18ded81711290e85: A672, A794, A916, A1038, A1160, A1282, A1404, A1526, A1770, A1892:B1893
- gen-m7/223090e4684153e5: A2014, A3356
- gen-m7/24bc4a707df18160: A916, A2258, A3600, A4942, A6284, A7626, A8968, A10310, A10676, A10798, A10920, A11042, A11164, A11286, A11408, A11530, A11652, A11774, A11896
- gen-m7/29f8e28340bf4440: A9700, A9822, A9944, A10066, A10188, A10310, A10432, A10554, A10798, A10920
- gen-m7/2e564b1fac9bc839: A9822, A11164
- gen-m7/35679b4facc62956: A10554, A10676, A10798, A10920, A11042, A11164, A11286, A11408, A11652, A11774
- gen-m7/3701c7edf02ba750: A3600, A3722, A3844, A3966, A4088, A4210, A4332, A4454, A4698, A4820
- gen-m7/37041df35489e178: A3722, A3844, A3966, A4088, A4210, A4332, A4454, A4576, A4820, A4942:B4943
- gen-m7/3a4c7294ff5c4af3: A9822, A11042
- gen-m7/3db804a002b664b1: A7748, A10432, A11774
- gen-m7/410128dc65d03200: A5064, A6406, A9334:B9335
- gen-m7/42c026cbec22a61f: A6650, A7992
- gen-m7/44a1feb80a78aa24: A1892
- gen-m7/45c197dc0b22cf2b: A6406, A6528, A6650, A6772, A6894, A7016, A7138, A7260, A7504, A7626:B7627
- gen-m7/4ad4b1c6b2c2b01d: A1526, A1648, A1770, A1892, A2014, A2136, A2258, A2380, A2624, A2746:B2747
- gen-m7/519cbbbc9fbcd166: A1038, A11774
- gen-m7/5239df16e9169bec: A3234, A4576, A7504:B7505
- gen-m7/53473f95ff58029d: A4088
- gen-m7/542087608aa65000: A6406, A6528, A6650, A6772, A6894, A7016, A7138, A7260, A7504, A7626
- gen-m7/547d6192794dbf57: A4576, A4820:B4821, A5186
- gen-m7/56474ca2acd1fd2e: A794, A11530
- gen-m7/5883a41f98113db2: A6040:B6041, A6406
- gen-m7/5c54b100d2f707be: A3722
- gen-m7/63168c3f18718748: A1526, A1648, A1770, A1892, A2014, A2136, A2258, A2380, A2624, A2746
- gen-m7/65a4611a6242a59f: A9578, A10920
- gen-m7/66bad08e2fd0f718: A7016, A7138, A7260, A7382, A7504, A7626, A7748, A7870, A8114, A8236:B8237
- gen-m7/69ea6a841b47d26d: A2746, A3966
- gen-m7/709913a7469ca178: A5308
- gen-m7/75038e7ef3522165: A8358, A9700
- gen-m7/78330db25d7a3822: A794, A2136
- gen-m7/79b07511f8e77743: A5918, A11896
- gen-m7/7ab307dba9cb2062: A9944
- gen-m7/8145169c978856f2: A8602, A8724, A8846, A8968, A9090, A9212, A9334, A9456, A9700, A9822
- gen-m7/88f592c463d2ffb0: A7138
- gen-m7/9077d156f68bf703: A11286
- gen-m7/96ebcfc607f7eaa7: A306
- gen-m7/9b7e3b9a41fc404e: A916
- gen-m7/9c128bd4a5d1d3b9: A1648:B1649
- gen-m7/a11a79d85df364b0: A1892, A3234
- gen-m7/a77b7647f69bcd5a: A7626
- gen-m7/b4486fce63db71a2: A11896, A12018, A12140
- gen-m7/c97eaa8d33e9d1cf: A9212
- gen-m7/cb6c98a18513801b: A11774:B11775, A12140
- gen-m7/ceb2c91035864a71: A4088
- gen-m7/cf6ab0ac7d65a5bb: A2014, A3356
- gen-m7/d03de506903f5a35: A2136
- gen-m7/d7232ffc680e4bac: A8114, A8236, A8358, A8480, A8602, A8724, A8846, A8968, A9212, A9334:B9335
- gen-m7/d7b4849a4fbe3485: A10310, A10432, A10554, A10676, A10798, A10920, A11042, A11164, A11408, A11530:B11531
- gen-m7/e4053a4410330089: A4820, A4942, A5064, A5186, A5308, A5430, A5552, A5674, A5918, A6040:B6041
- gen-m7/e5bac786f4f6a903: A2136, A2258, A2380, A2502, A2624, A2746, A2868, A2990, A3234, A3356:B3357
- gen-m7/e8a89caaafba1c89: A3722, A5064
- gen-m7/eb01a9867300331b: A3722, A9822, A11164
- gen-m7/ec5094d09c82f94d: A11652
- gen-m7/f310ddb6bc8ec33a: A8968, A10066
- gen-m7/f84158fac01b5cb5: A1160
- gen-m7/fbb32b6c948cc84e: A62, A184, A306, A428, A550, A794, A916
- gen-m7/febfd6901374d79c: A9578, A10920

## Visible-parity stage
Decision 0083: every cell listed under Context is visibly off (a wrong error type, a value where Excel has an error, or a spill where Excel has none), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [ ] Every cell listed under Context matches Excel (`make eval-show` on each case), with unit tests in the function's module for the rule.
- [ ] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The values argument, literal errors, and the other rules of the batch (T-3843, T-3844, T-3845, T-3846).
