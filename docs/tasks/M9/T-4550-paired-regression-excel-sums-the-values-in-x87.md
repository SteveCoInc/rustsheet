---
id: T-4550
title: "Paired regression: Excel sums the values in x87 and double-rounds the slope division (FORECAST table A67, A79)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The engine's FORECAST, FORECAST.LINEAR, SLOPE and INTERCEPT match Excel to the bit on every numeric point of the Gate A
tables `accuracy/{FORECAST,FORECAST.LINEAR,SLOPE,INTERCEPT}/s1`, including FORECAST A67 and A79, with no point of the
CORREL, PEARSON and RSQ tables losing its bit match; or the Outcome lists each form tried with its bit-exact count
(decision 0056 time box).

## Context
- Found by T-4094 (the paired regression tables, excel-graph). The engine is bit-exact with Excel on 998 of the 1,000
  points of the seven tables. The other two are FORECAST, both within 15 significant digits of Excel (inside the Gate A
  envelope, so no known failure):
  - A67 `FORECAST(0.838499859843194,{-0.109514844082525,-3.34908202370414},{0.838499859843194,50.1841931039674})`:
    Excel -0.1095148440825254, ours -0.10951484408252497 (about 30 ulps: the line a + b*x cancels);
  - A79 (seven pairs, x = -276319007.733847): Excel 568833.0148482913, ours 568833.0148482914.
- A model fits all 597 numeric points of the FORECAST, FORECAST.LINEAR, SLOPE and INTERCEPT tables: the sums of the
  values are x87-accumulated (each addition rounded to a 64-bit significand, then to a double: numbers.md's SUM model,
  `running_total` in tools/src/rustsheet_tools/accuracy/defs/aggregate.py), and the slope Sxy/Sxx is rounded the same
  way (to 64 bits, then to 53). The sums about the means and the line a + b*x stay plain doubles. The engine's plain
  doubles (`PairedMoments` in crates/rustsheet-engine/src/functions/statistical/correl.rs) fit 595. Recorded in
  statistics.md "Paired regression functions: single points, zero variance, text and large offsets" (T-4094 paragraph).
- T-3257 (a stray PEARSON last digit in gen-m9) may share the cause: check the x87 sums there too.

## Acceptance criteria
- [ ] FORECAST A67 and A79 match Excel to the bit; `rustsheet-eval accuracy` shows no other point of the seven paired
      tables losing its bit match, and every regression probe cell of statistics.md's entry still matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- STEYX and the COVARIANCE family (no table yet).

## Deferred (decision 0114)
`defer: phase-1b`: both points are already within 15 significant digits of Excel, so this waits until Phase 1 exits.
