---
id: T-3901
title: "Probe typed mixed-separator and clock-first dates in a day-first locale"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Typed mixed-separator dates (`2025/07-01`, `1-Jul/2025`) and clock-first date-times in a day-first locale take Excel's value and number format.

## Context
T-3796 and T-3852 evidenced en-US only (probes/date-text-mixed-separator-typed, date-text-typed-clock-first-shapes). `mixed` in `date_text.rs` skips day-first locales and keeps the joined separators; `leading_clock_shape` reads the date in the locale's order. Needs an excel-graph locale probe (or a desktop question for M14).

## Acceptance criteria
- [ ] A probe case records Excel's typed value and format in a day-first locale; the engine matches or the finding is recorded in docs/excel-semantics/dates.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
