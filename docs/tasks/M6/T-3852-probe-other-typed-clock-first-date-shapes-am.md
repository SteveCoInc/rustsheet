---
id: T-3852
title: "Probe other typed clock-first date shapes (AM/PM, seconds, out-of-range, month-name dates) and extend leading_clock_shape"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed clock-first date-times other than `H:MM <m/d/yyyy>` match Excel (value and number format).

## Context
T-3796 limited `leading_clock_shape` (crates/rustsheet-engine/src/formatter/parse_formatted.rs) to the evidenced `10:00 7/1/2025`. Unprobed typed shapes: `10 AM 7/1/2025`, `10:00 AM 7/1/2025`, one-digit hours (`9:00 7/1/2025`), seconds, fractions, out-of-range clocks, double spaces, and other dates after the clock: month-name (`10:00 July 1, 2025`, `10:00 1-Jul-2025`), ISO (`10:00 2025-07-01`), year-less (`10:00 7/1`), two-digit-year (`10:00 7/1/25`) and mixed-separator (`23:59 7/1-2025`). Also unprobed: typed mixed-separator dates (`2025/07-01`, `1-Jul/2025`) in a day-first locale, which T-3796 left on the joined-separator formats; clock-first date-times in a day-first locale (`leading_clock_shape` reads the date in the locale's order but always gives `m/d/yyyy h:mm`); and untyped mixed shapes that T-3796's `mixed` rule generalizes to (`2025/7-1`, `1-Jul/25`). Use the `excel-probe` skill.

## Acceptance criteria
- [x] A probe case types each shape above and records Excel's value and format; `leading_clock_shape` matches.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 22 en-US shapes (probes/date-text-typed-clock-first-shapes, excel-graph 2026-11-03): a typed clock-first date-time
is a number with the clock's format (`m/d/yyyy h:mm`, `mm:ss.0` for fractions, General out of range) for any readable date
after it. `leading_clock_shape` now classifies the clock with `typed_clock_shape` and lets the date reader decide the rest;
trailing spaces tolerated. Recorded in docs/excel-semantics/dates.md. Eval delta: see PR.
Not probed (stay open): day-first locales (clock-first and mixed-separator typed dates); no probe backend for locale yet.
