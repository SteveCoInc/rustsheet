---
id: T-3948
title: "Probe the letter-period shape gate's unprobed clocks and tails; DATEVALUE shares one range-check-then-shift step"
milestone: M6
lane: functions-date
status: in-progress
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
- [ ] A probe case records the four readers and the typed cell for the shapes above; the engine matches each cell or dates.md records the gap with an owner task.
- [ ] `parse_datevalue_text` has one range-check-then-shift step; the shapes case ends with a newline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
