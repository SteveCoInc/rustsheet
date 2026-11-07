---
id: T-3693
title: "Gate A slice 4l: reference table for AGGREGATE's array forms (functions 14-19)"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3551]
---

## Goal
AGGREGATE has a committed, oracle-filled `s1` Gate A table over its array forms, functions 14-19 (LARGE, SMALL,
PERCENTILE.INC, QUARTILE.INC, PERCENTILE.EXC, QUARTILE.EXC), with a cited mpmath definition.

## Context
- Decision 0228 (T-3551): functions 1-13 take references only (numbers.md "AGGREGATE"), so the gen suites and probes
  drive them; functions 14-19 take an array and k, which the generator's one-row array literal (T-3586) can write, and
  they have floating-point kernels: `AGGREGATE(18,0,{3,1,4,1,5,9},0.3)` is 1.2000000000000002 in Excel (probes/aggregate-grid
  H45). So AGGREGATE needs a table, drawn over function_num 14-19.
- The definition: the order statistic (14, 15) or the inclusive/exclusive percentile and quartile interpolation (16-19)
  of the array's doubles; options 0-7 do not change an array-literal argument's numbers (an array literal holds no
  hidden rows or errors when drawn as numbers). Cite statistics.md's percentile entries and numbers.md "AGGREGATE" and
  "AGGREGATE 14-19: k is read before the array" (statistics.md).
- k outside the valid range (k < 1 or above the count for 14 and 15; outside [0, 1] for .INC, (0, 1) and the
  count-dependent bounds for .EXC) is a domain error.

## Acceptance criteria
- [x] A definition for AGGREGATE over function_num 14-19, with tests against the probes/aggregate-grid H41-H46 values.
- [x] `eval/accuracy/tables/AGGREGATE/s1.{case,math,oracle}.json` generated, filled (excel-graph) and committed;
      `rustsheet-eval accuracy` reports `missing tables` lower by 1; failing points owned (`known-failures.tsv`).
- [x] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE 1-13 (reference-only; the eval suites drive them).

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/aggregate_array.py`: AGGREGATE over function_num 14-19 on an array
  literal's doubles. 14/15 take the ceil(k)-th largest / floor(k)-th smallest (statistics.md "LARGE/SMALL"), 16/18 the
  exact PERCENTILE.INC / .EXC interpolation (ranks 1 + k(n-1) and k(n+1)), 17/19 the percentile at floor(k)/4. Domain
  errors: k below 1 or ceil(k) above n (14, 15), k outside [0, 1] (16), k outside (0, 1) or rank outside [1, n] (18),
  quart outside 0-4 / 1-3 (17, 19), and function_num outside 14-19 (1-13 over an array literal is `#VALUE!`,
  numbers.md "AGGREGATE"). Options 0-7 are drawn and do not enter the math. Draws keep k off the bounds Excel compares
  in doubles, except bounds that are exact doubles (n + 1 = 4 or 8), and keep k >= 1e-6 for PERCENTILE.INC (Excel's
  one-based rank loses a smaller k). Tests: `tools/tests/test_accuracy_aggregate_array.py` (probes/aggregate-grid
  H41:H46 and the statistics.md LARGE/SMALL cells).
- `Definition.literal_arrays` (model.py, eval_math.py): an eval cell gets a math value only where each array
  parameter is written as an array literal. AGGREGATE over a range is skipped ("not an array literal"): the math does
  not model hidden rows or the options that skip them. The probes math sidecar of aggregate-grid gains H41:H46
  (`make eval-math-check`: current). `test_definitions_follow_the_catalog_parameter_order` lets a repeated catalog
  parameter (AGGREGATE's `ref`, which holds the array and then k) cover a fixed-arity definition's later parameters.
- Table `s1` (200 points, 11 domain errors), filled from excel-graph on 2026-11-04. `rustsheet-accuracy verify`: 0
  differences. Every Excel error (11 `#NUM!`) is a math domain error and the reverse. Excel's widest gap from the math
  is 2.8e-13 relative (A165, an interpolation that cancels), 7 points above 1e-15; none is an R3 candidate.
  `rustsheet-eval accuracy`: 14354 points in 94 tables, 0 failing, 0 table defects; missing tables 200 → 199. No
  `known-failures.tsv` line (no failing point).
- Eval delta: none (no engine change).
