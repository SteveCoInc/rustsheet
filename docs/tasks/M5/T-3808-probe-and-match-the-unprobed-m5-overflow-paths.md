---
id: T-3808
title: "Probe and match the unprobed M5 overflow paths: SUBTOTAL 101/106/109, AGGREGATE PRODUCT and variance numbers under every option, SUMX2MY2 overflow"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Every M5 aggregate path that the engine sends through the overflow check without Excel evidence has Excel
cells, and the engine gives Excel's value or error on each: SUBTOTAL 101, 106 and 109, AGGREGATE 6, 7, 8, 10 and 11
and AGGREGATE 1 and 9 under options other than 6, SUMX2MY2 when a square overflows, and MEDIAN and GEOMEAN with
huge direct arguments.

## Context
- Gap triage 2026-11-02 (`make eval` on main 2290b8769). Every failing Excel cell of the M5 math functions already has
  an open owner (`make unowned-off ALL=1` lists none). What is left are paths numbers.md marks as unprobed. This
  task fills those evidence gaps (decision 0031). It owns no failing cell today.
- numbers.md "Aggregate overflow" (T-0192, VERIFIED, probes/aggregate-overflow): SUM, SUBTOTAL 9, 1 and 6, AGGREGATE
  9 and 1 (option 6 only, Calc!A27:C29), SUMX2PY2 (A42) and SUBTOTAL 7, 8, 10, 11, 107, 108, 110 and 111
  (probes/subtotal-variance-one-vs-two-pass K10:K17) overflow to `#NUM!`. Its implementation note says "SUBTOTAL
  101/106/109 share the code but are unprobed". It also says "SUMX2MY2 overflow itself is unprobed": A41 is
  `SUMX2MY2({9E+153},{0})`, a finite 8.1E+307. And it says "direct arguments not probed" for MEDIAN and GEOMEAN
  (A47, A48 read F1:F2). probes/aggregate-grid covers AGGREGATE 1-19 by option 0-7, but only over ranges with no
  overflow. probes/subtotal-106-no-numbers covers hidden and filtered rows, but not overflow.
- What to probe (excel-probe skill, excel-graph; value, `&""` and `ERROR.TYPE` per formula as aggregate-overflow
  does):
  - SUBTOTAL 101, 106 and 109 over F1:F3 = 9E+307, 9E+307, -9E+307 and F5:F6 = 1E+200 twice, with no row hidden and
    with a manually hidden row that holds one of the huge values (hidden rows need a sheet the oracle saves with row
    visibility, as probes/subtotal-106-no-numbers does).
  - AGGREGATE 6, 7, 8, 10 and 11, and 1 and 9, each under options 0, 1, 2, 3, 4, 5 and 7 over the same ranges, plus
    one range that also holds an error cell (options 2, 3, 6 and 7 ignore it).
  - SUMX2MY2 where both squares overflow but their difference (2.9E+307) is finite (`{1.5E+154},{1.4E+154}`) and
    where the difference overflows too (`{1E+155},{0}`), with SUMXMY2 over the same pairs as the control.
  - MEDIAN and GEOMEAN with direct arguments `9E+307,9E+307` and `1E+200,1E+200,1E+200`.
- Code: `CalcResult::finite_number_or_num` at the ends of functions/subtotal.rs (SUBTOTAL), functions/aggregate.rs
  (AGGREGATE), math_and_trigonometry/sum.rs (`fn_sumx2my2` and its SUMX2PY2/SUMXMY2 siblings), product.rs and
  sumproduct.rs. T-2384 owns SUMXMY2's last digit. Leave its accumulation alone.
- If a probed cell shows the overflow check in the wrong place (per term, not on the result, or the reverse), that is
  a semantic rule: write it up in numbers.md and fix it here if the change stays in the function's own module. Give
  anything wider to a follow-up task.
- Tier: opus. The Excel evidence does not exist yet, and hidden-row probes need judgment about the oracle's row
  visibility.

## Acceptance criteria
- [x] A probe case under eval/suites/probes (excel-graph sidecar) covers every bullet of "What to probe".
- [x] Every probed cell passes bit-exact: no new line in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions), or each remaining one is owned by a follow-up task filed in this PR.
- [x] numbers.md "Aggregate overflow" drops "SUBTOTAL 101/106/109 share the code but are unprobed", "SUMX2MY2 overflow
      itself is unprobed" and "direct arguments not probed", and states the probed results with their cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Last-digit accumulation of SUMXMY2 (T-2384) and Gate A reference tables (T-3676, T-3677).
- Statistical aggregates (VAR, STDEV, DEVSQ: already probed; functions-stat).

## Outcome
Probed and matched. No engine change was needed: the engine already gives Excel's value or error on every M5 cell.
- probes/t3808-m5-aggregate-overflow (M5, 576 cells, excel-graph 2026-11-02; A value, B `&""`, C ERROR.TYPE):
  SUBTOTAL 101/106/109 are `#NUM!` (Calc!A20:C25). AGGREGATE 1, 6, 7, 8, 9, 10 and 11 are `#NUM!` under every
  option 0-7 over F1:F2, F1:F3 and F5:F6. Over a range that also holds `#N/A`, options 0, 1, 4 and 5 give `#N/A` and
  options 2, 3, 6 and 7 give `#NUM!` (Calc!A26:C177). SUMX2MY2 is `#NUM!` when a square overflows, even with a finite
  difference (2.9E+307); SUMXMY2 is the control (A178:C181). On the Hide sheet (rows 3, 7, 9 hidden) the hidden rows
  are dropped before the check: SUBTOTAL 109 and 101 and AGGREGATE 9 under options 1, 5 and 7 over 9E+307 (hidden),
  9E+307, 1 are finite, and the codes that count hidden rows are `#NUM!` (Hide!A20:C49).
- probes/t3808-median-geomean-direct-overflow (M9, split off because MEDIAN and GEOMEAN are M9 in the catalog;
  18 cells): MEDIAN and GEOMEAN with direct arguments 9E+307 x2 and 1E+200 x3 match. One extra cell beyond the task,
  `GEOMEAN(9E+307,9E+307,1E-300)` (A6), is one ulp high (product past the double range). It is filed as
  T-3847 (phase-1b) and is the case's only baseline line.
- numbers.md "Aggregate overflow": the three "unprobed" / "not probed" clauses are replaced with the probed results.
- Also filed T-3848 (sonnet): criteria.md wording follow-up from the T-3807 fidelity review (MINOR).
- Eval delta: probes +2 cases, +594 cells, 593 bit-exact. One new known failure (A6, owned by T-3847) and
  0 regressions.
