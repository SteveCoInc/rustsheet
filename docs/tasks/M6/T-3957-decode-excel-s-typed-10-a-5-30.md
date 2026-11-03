---
id: T-3957
title: "Decode Excel's typed '10 A. 5:30' (2982616.579, 60 x (2^32-22) s + 10 h + 5.5 s): a colon in the letter-period tail"
milestone: M6
lane: research
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The engine reads `N A. F:G` text (a colon after the letter-period digits) as Excel does, or dates.md records the decoded rule and why the engine leaves it text.

## Context
Typed `10 A. 5:30` is a number on excel-graph: 2982616.579230324, i.e. 257698072445.5 s = 60 x (2^32 - 22) s
+ 36005.5 s (10 h + 5.5 s); TEXT with a date format is #VALUE! (beyond 9999-12-31), VALUE and `0+` give the same
number, DATEVALUE 2982616 and TIMEVALUE 0.5792303238995373 (probes/t3948-letter-period-gate-edges row 10, cells
B10:H10, excel-graph 2026-11-03). Without the colon (`10 A. 5`) the base is 2^32 - 22 seconds plus 10 minutes plus
0.5 s (docs/excel-semantics/dates.md "Typed clock-first date-times"), so the colon looks like it moves every unit up
one place (base in minutes, the clock in hours) with the `30` unaccounted for. One point cannot fix the rule: probe a
sweep (`10 A. 5:0`, `5:59`, `5:60`, `0:30`, `10:00 A. 5:30`, `10 P. 5:30`, `10 A. 5:30:15`, `10 A. 5:30 7/1/2025`;
and, for T-3948's any-tail gate, `10 a.m.`, `10:00 a.m.`, `10 A. x`, `10 A. 5x 7/1/2025`, review MINOR of #2996)
typed and through the four readers, decode it, then implement it in `date_clock::letter_period_parts` (the engine
leaves the text a string, and the readers give #VALUE!, via `is_letter_period_shape`).

## Acceptance criteria
- [ ] A sweep case decodes the colon tail; probes/t3948-letter-period-gate-edges B10:H10 pass, or dates.md records why not with evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
