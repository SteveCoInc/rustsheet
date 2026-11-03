---
id: T-3907
title: "Decode Excel's reading of a typed clock-first date with an A. suffix (10 A. 7/1/2025 is a 2161 serial in mm:ss.0)"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine reads a typed `10 A. 7/1/2025` and `10:00 A. 7/1/2025` as Excel does: the same number and the same `mm:ss.0` format.

## Context
Found by T-3852 (probes/date-text-typed-clock-first-shapes S!A35, A36, excel-graph 2026-11-03). Excel gives
95368.2763275463 (TEXT `2161-02-07 06:37:55`) and 95368.68604976853 (`2161-02-07 16:27:55`), both formatted
`mm:ss.0`. The two values differ by 9 h 50 min, as if the bare `10` were read as minutes and `10:00` as hours. The
engine keeps both as text (`leading_clock_shape` in formatter/parse_formatted.rs does not take `A.` as a clock word).
The six cells B35:D36 are known failures in eval/baseline/probes.txt. See docs/excel-semantics/dates.md
"Typed clock-first date-times".
Probe more shapes first (excel-probe skill): `1 A. 7/1/2025`, `10 P. 7/1/2025`, `10 A. 1/1/1900`, `10 a.m. 7/1/2025`,
`10 A.`, and `A. 7/1/2025` alone, to find the rule before coding.

## Acceptance criteria
- [ ] probes/date-text-typed-clock-first-shapes S!B35:D36 pass, the rule is probed on further shapes, and dates.md records it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
