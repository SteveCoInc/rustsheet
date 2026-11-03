---
id: T-3948
title: "Probe the letter-period shape gate's unprobed clocks and tails; DATEVALUE shares one range-check-then-shift step"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The letter-period shape gate (`date_clock::is_letter_period_shape`, T-3920) is evidenced for the clocks and tails it covers beyond the ~10 probed shapes, and DATEVALUE's letter-period branch shares the range-check-then-shift step with the other branch.

## Context
Follow-up of T-3920's review (#2987, MINOR findings). Excel fidelity: the gate turns VALUE, `0+`, DATEVALUE and
TIMEVALUE into #VALUE! for every text with a digits-and-colons clock, a spaced `A`/`P`/`AM`/`PM` word, its period and a
digit, wherever the letter-period reading rejects it; the evidence is probes/t3920-typed-a-period-unprobed-edges rows
8-23. Unprobed: multi-colon clocks (`1:2:3:4 A. 5`, `10:00:00 P. 5`), a leading-zero or long clock (`010 A. 5`,
`00010:00 A. 5`), tails with letters or a second clock after the digits (`10 A. 5x`, `10 A. 5 PM`), and the gate's
neighbours that it does not cover (`10 A. July 1, 2025`, `10 A.  ` with nothing after). Design: in
`Model::parse_datevalue_text` the letter-period branch repeats the max-serial check; one helper should do the
check-and-shift for both branches. Also restore the final newline of probes/t3907-typed-clock-a-period-shapes.case.json.

## Acceptance criteria
- [x] A probe case records the four readers and the typed cell for the shapes above; the engine matches each cell or dates.md records the gap with an owner task.
- [x] `parse_datevalue_text` has one range-check-then-shift step; the shapes case ends with a newline.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- probes/t3948-letter-period-gate-edges (A1-H16, excel-graph 2026-11-03): every multi-colon, leading-zero or
  empty-part clock and every tail with text after the period (`10 A. July 1, 2025`, `10 A. Jul-2025`, `10 A. 5x`)
  stays text and is #VALUE! to VALUE, `0+`, DATEVALUE and TIMEVALUE, so `is_letter_period_shape` now commits on any
  text after the period, not only a digit. `10 A.  ` with trailing spaces is 10 AM typed
  (`without_single_letter_dot` trims them). Recorded in docs/excel-semantics/dates.md.
- `parse_datevalue_text` maps the letter-period day back to the 1900 count and shares the range check and shift.
  The t3907 shapes and t3920 cases end with a newline again (review MINOR of #2987).
- Eval delta: the new case adds 112 cells, 104 pass (93 before the gate change). Open: row 10 (`10 A. 5:30` is
  2982616.579230324, undecoded) B10:H10, owned by T-3957; G14 (`DATEVALUE("10 A.  ")` is 0 in 1900) by T-3944.
