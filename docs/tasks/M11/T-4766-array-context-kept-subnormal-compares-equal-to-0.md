---
id: T-4766
title: "Array-context kept subnormal compares equal to 0 in Excel (probe cells A3 and D1)"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Match Excel for a kept subnormal inside an array expression: `=(EXP({-745,-708}))=0` is TRUE in excel-graph, while the scalar `(EXP(-745))=0` is FALSE.

## Context
Found by T-4355 (probes/subnormal-saved-negative-and-array A3 and D1, excel-graph 2026-11-07). D1 is TRUE and spills nothing further; the engine gives FALSE. `INDEX(EXP({-745,-708}),1)*2^1000`-style recovery still works for a spilled cell (A5), so the flush may apply only in array-context comparison, or be a Graph artifact (decision 0042); the desktop question goes to M14 (decision 0039). Probe which before changing the engine.

## Acceptance criteria
- [x] Probe cell D1 is fixed or recorded as DEVIATION / desktop-owned with evidence; numbers.md "Worksheet EXP (T-1931)" updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Not a Graph artifact (the cell is a boolean, which the save does not touch): when either operand of a comparison is an array or a
range, Excel reads a left number that exceeds the right one by less than 2^-1026 as equal to it; a left number below the right
one keeps its order, and scalar comparisons stay exact. `arithmetic::handle_comparison` now applies that gap on its array paths
(`within_array_compare_gap`), and D1 passes.
- Evidence (excel-graph, 2026-11-07), five new probes: t4766-array-subnormal-compare (29 forms: broadcast scalars, spilled
  ranges, `A30:B30`, single-element arrays, aggregates), -boundary (2^-k sweep; it showed POWER flushes subnormals, so 2^-k
  cannot build test values), -exp-boundary (EXP(a), a = -720..-708.4), -order (boundary a = -711.20..-711.00 by 0.01, switch
  between -711.17 and -711.16; a 144-cell =/</> matrix with the array left, right or both).
- Not a decision 0188 R4 fit: 2^-1026 is an oracle-measured boundary, pinned by fresh A1 (2^48-1 units, equal) and A2 (exactly
  2^48 units, built from the 15-digit multiplier 281474976710656, not equal) and by the -order bracket, so there is no
  hold-out split. Probe 5, -fresh (94 cells: exact m*2^-1074 built by subtraction, m log-uniform in [2^44, 2^52) with seed 4766
  and 2^48 +-1, pairs, negative operands), was written after the rule was frozen, and the engine matches all 94. A standalone
  prediction that assumed exact operands missed E5 and E6 (E4 is TRUE either way): Excel cuts the 16-digit literal 1125899906842624
  to 15 digits (1125899906842620), so the gaps of E4:E6 are 2^48-5, 2^48-4 and 2^48-3 units and read TRUE; the engine matches because it
  reproduces that cut. There is no keep-matched branch. Unprobed, the engine's choice: two normal numbers less than 2^-1026
  apart, and an empty range element read as 0 against a tiny number.
- A3 (the saved 0 of the spilled subnormal) stays a known failure and moves to T-5051 with the new case's S!A30: decision 0042's
  companion `=(<formula>)=0` is TRUE in array context, so it cannot excuse a spilled element.
- numbers.md "Worksheet EXP (T-1931)": the comparison rule, with every probe; unit test
  `array_comparison_reads_a_tiny_excess_as_equal` (test_exp_sign.rs).
- Eval delta: probes fixed 1 (subnormal-saved-negative-and-array D1), 5 new cases (838 scored cells, 1 visibly off: t4766-array-subnormal-compare
  A30, owned by T-5051), regressions 0.
