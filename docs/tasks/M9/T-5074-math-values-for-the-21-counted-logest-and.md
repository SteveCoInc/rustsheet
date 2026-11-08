---
id: T-5074
title: "Math values for the 21 counted LOGEST and GROWTH cells of t4590, t4765 and t4793: a 3-argument INDEX(LOGEST(..)) and a one-value GROWTH outside INDEX"
milestone: M9
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4780]
---

## Goal
The 21 visibly-off LOGEST and GROWTH cells of probes/t4590-growth-overflow, t4765-growth-factor-edges,
t4793-growth-several-x and t4793-growth-one-factor-underflow get math values. Decision 0188 R1(c) can then sort them.
Today they count only because they have no math value (decision 0325), and their one owner, T-4763, is a digit fit
that must not start on them before the math is attached.

## Context
- Gap triage 2026-11-08 (`make eval` on origin/main 6f0c3d8e5, excel-graph). None of the four cases has a
  `.math.json`, and `rustsheet-accuracy math --check --suite probes` reports them current: `eval_math.candidates`
  drops every one of their formulas before any mpmath runs. In t4590-growth-overflow, for example, it drops 38
  `INDEX(LOGEST(..))` cells for `argument count` and 23 `GROWTH(..)` cells as `an array result outside INDEX`.
- The two candidate rules that drop them, both in `tools/src/rustsheet_tools/accuracy/defs/array_regression.py`
  `_definition`:
  - every parameter is required. `=INDEX(LOGEST({1.00709088702808e+152,1,9.92959039626498e-153},{1,2,3},TRUE),1,2)`
    passes known_y, known_x and const but no stats. With the row and column, that is 5 arguments where the
    definition takes 6. Excel's defaults are const TRUE and stats FALSE, and a missing stats changes no coefficient,
    so the trailing flags should be optional.
  - GROWTH and TREND are `indexed=True`, so a call outside INDEX is skipped even when its new_x is a single point, one value or one row
    of the x variables (`=GROWTH({..},{1,2,3},2,TRUE)` is a 1x1 result, the value itself). The cell has its own formula, so it is not
    the spilled-cell keying that T-4780 designs. It should still follow T-4780's anchor rule once that lands: the
    anchor is element (1, 1). That is why this task depends on T-4780.
- The math is the existing exact least-squares solve with ln y taken above the working precision (T-4096, decision
  0271). Nothing new is fitted. These fits sit near the double range (ln y about ±350 to ±354), which mpmath handles
  as it is. If a definition then declines a cell (`NoValueError`: rank deficiency, an exact fit's residual cell),
  record the reason in the sidecar's `skipped` list.
- Likely result: Excel and the engine are 5E-14 to 3E-13 relative apart (T-4763). With a math value most cells should
  become Phase 1b evidence (the engine within 1e-9 of the math) and leave the M9 probe row's count.

Cells this task owns for their math value (the digit fit stays with T-4763):
- probes/t4590-growth-overflow.case.json (14): Int!A1, B1, B2, B4, B5, B6, B7, C4, C6, C7, C10, C11, Slope!A4, A5
- probes/t4765-growth-factor-edges.case.json (4): Cut!A1, A5, SmallB!A1, A3
- probes/t4793-growth-several-x.case.json (2): Multi!A4, SubProd!A5
- probes/t4793-growth-one-factor-underflow.case.json (1): One!A2

## Acceptance criteria
- [ ] A definition test shows `INDEX(LOGEST(y,x),r,c)` and `INDEX(LOGEST(y,x,const),r,c)` take Excel's defaults
      (const TRUE, stats FALSE), and that a GROWTH or TREND call outside INDEX whose new_x is a single point is a candidate.
      A call with several new_x points outside INDEX is still skipped as `an array result outside INDEX`, or keyed by
      T-4780's rule.
- [ ] `cd tools && uv run rustsheet-accuracy math --suite probes --function LOGEST --function GROWTH` writes the four
      sidecars. Each of the 21 cells has a value or a named skip, `rustsheet-accuracy math --check` passes, and
      `rustsheet-accuracy verify LINEST`, `LOGEST`, `TREND` and `GROWTH` show no change.
- [ ] After `make eval`, the Outcome gives per case the count that became Phase 1b evidence (`r1c_left_out`) and,
      for each cell still counted, Excel-minus-math and ours-minus-math. `eval/baseline/` gains no new line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes to LOGEST or GROWTH (T-4763, T-2006).
- Spilled-cell keying (T-4780) and the linest, logest and growth modes probes.
