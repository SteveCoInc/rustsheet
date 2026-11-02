---
id: T-3852
title: "Probe other typed clock-first date shapes (AM/PM, seconds, out-of-range, month-name dates) and extend leading_clock_shape"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed clock-first date-times other than `H:MM <m/d/yyyy>` match Excel (value and number format).

## Context
T-3796 limited `leading_clock_shape` (crates/rustsheet-engine/src/formatter/parse_formatted.rs) to the evidenced `10:00 7/1/2025`. Unprobed typed shapes: `10 AM 7/1/2025`, `10:00 AM 7/1/2025`, seconds, fractions, out-of-range clocks, month-name dates after the clock, double spaces. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case types each shape above and records Excel's value and format; `leading_clock_shape` matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
