---
id: T-3907
title: "Decode Excel's reading of a typed clock-first date with an A. suffix (10 A. 7/1/2025 is a 2161 serial in mm:ss.0)"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine reads a typed `10 A. 7/1/2025` and `10:00 A. 7/1/2025` as Excel does: the same number and the same `mm:ss.0` format.

## Context
Found by T-3852 (probes/date-text-typed-clock-first-a-period S!A1, A2, excel-graph 2026-11-03). Excel gives
95368.2763275463 (TEXT `2161-02-07 06:37:55`) and 95368.68604976853 (`2161-02-07 16:27:55`), both formatted
`mm:ss.0`. The two values differ by 9 h 50 min, as if the bare `10` were read as minutes and `10:00` as hours. The
engine keeps both as text (`leading_clock_shape` in formatter/parse_formatted.rs does not take `A.` as a clock word).
The six cells B1:D2 of that case are known failures in eval/baseline/probes.txt. See docs/excel-semantics/dates.md
"Typed clock-first date-times".
Probe more shapes first (excel-probe skill): `1 A. 7/1/2025`, `10 P. 7/1/2025`, `10 A. 1/1/1900`, `10 a.m. 7/1/2025`,
`10 A.`, and `A. 7/1/2025` alone, to find the rule before coding.

## Acceptance criteria
- [x] probes/date-text-typed-clock-first-a-period S!B1:D2 pass, the rule is probed on further shapes, and dates.md records it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Decoded and implemented. `CLOCK A. DIGITS[SEP DATE]` (also `P.`, `PM.`, any case) is (2^32 - 22 s, +1 s for P) +
the clock (a bare N is minutes, below 60; `H:M` hours and minutes) + `0.DIGITS` s rounded half up to the millisecond
+ the date after `/`, `-` or a space (read on its own; none is day 0); typed it takes `mm:ss.0`, and VALUE, `0+`,
DATEVALUE and TIMEVALUE read the same number. `date_clock::letter_period_parts` holds the rule;
`parse_date_clock_parts` (formula readers) and `parse_typed_number` (typed, via `parse_letter_period_serial_in`) use it.
Evidence: probes/t3907-typed-clock-a-period-shapes (26 typed shapes, decoded the rule) and
probes/t3907-typed-clock-a-period-edges (24 typed shapes and 6 formula readers, written after the rule was fixed, a
fresh sweep that passed on the first engine run), excel-graph 2026-11-03; ENTERED formats in
docs/research/t3907-typed-clock-a-period-2026-11-03.md; rule in docs/excel-semantics/dates.md "Typed clock-first
date-times". The 2^32 - 22 s constant is no fit: it is the exact integer residual of `0 A. 7/1/2025` (shapes A16)
after the date and the `.7` s, and every other cell then follows with no free parameter.
Eval delta: +162 passing cells (6 known failures of probes/date-text-typed-clock-first-a-period fixed, 156 new cells
in the two new cases, all passing); regressions 0.
Unprobed (dates.md lists them): `AM.`, an `H:M` hour of 24 or more, a 1904 workbook, the formula readers of shapes that
stay typed text, and the `mmm-yy` format Excel gives the text cells `60 A. 7/1/2025` and `99 A. 7/1/2025`.

