---
id: T-3852
title: "Probe other typed clock-first date shapes (AM/PM, seconds, out-of-range, month-name dates) and extend leading_clock_shape"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: opus
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
Probed 33 en-US shapes (probes/date-text-typed-clock-first-shapes, excel-graph 2026-11-03): a typed clock-first date-time
is a number with the clock's format (`m/d/yyyy h:mm`, `mm:ss.0` for fractions, General out of range) for any readable date
after it. `leading_clock_shape` now classifies the clock with `typed_clock_shape` and lets the date reader decide the rest;
trailing spaces tolerated. Recorded in docs/excel-semantics/dates.md. Eval delta: new probe case passes 66/66 (probes baseline +0 failures); no regressions.
Not probed: day-first locales, filed as T-3901.

## Handoff
PR #2928, head 36028661600d (CI not yet seen green on it). Implemented and probed (case rows A1-A33, all excel-graph, 66/66 pass; `make check-fast` green).
Security and design reviewers approved. excel-fidelity requested changes twice (round 1: unprobed shapes, fixed by A23-A33; round 2 open):
- MAJOR: doc rule "a clock before a date is a number whenever the same clock after it would be" breaks for the hour-only single-letter clock: `10 A 7/1/2025`, `10 A. 7/1/2025`, `10:00 A. 7/1/2025` stay text in the engine (`clock_end` in `leading_clock_shape` only takes a full AM/PM word). Next: add these three rows to the case, fill from excel-graph (`rustsheet-oracle fill -b excel-graph --suite probes --force --case probes/date-text-typed-clock-first-shapes.case.json`), match the result.
- MINOR: add an UNVERIFIED sentence in dates.md "Typed clock-first date-times" for inherited shapes (`99:00`, `10:99`, `10:00:30.5 AM`, `0 AM`, `12 PM` before a date) or probe them; `10:00 2025/7/1`, `10:00 Jul-25`, `10:00 1-Jul`, `10 AM 7/1` also lack clock-first rows.
- MINOR: score full serials (not MONTH*100+DAY+MOD) for rows with a year (C23-C27, C30-C33); only A14 and A29 need the year-less formula.
Do not repeat: do not run `rustsheet-accuracy math --suite probes` (about 30 min, rewrites unrelated sidecars such as t3350; I had to revert one); `make eval-math-check` says current. After changing the case, refill oracle, `make eval-baseline`, `make check-fast`.
