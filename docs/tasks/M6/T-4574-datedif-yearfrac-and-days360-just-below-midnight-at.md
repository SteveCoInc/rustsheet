---
id: T-4574
title: "DATEDIF, YEARFRAC and DAYS360 just below midnight at serials 59, 60 and 61 (the fictitious 1900-02-29): probe and match"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The floor rule T-4454 found for date-serial arguments just below midnight is evidenced for DATEDIF, YEARFRAC and
DAYS360 at the bases around Excel's fictitious 1900-02-29 (serial 60), and the engine matches it there.

## Context
- T-4454 (#3490, probes/t4454-near-midnight-date-args) showed that DATEDIF, YEARFRAC and DAYS360 floor a serial just
  below midnight (N-0.4 s and N-(2^-22+2^-33) read day N-1; DAYS360 rounds to the second). Its bases avoid serials
  59/60/61, which T-4454's Context suggested. The only evidence near serial 60 is one gen-m6 DATEDIF cell
  (59.999988426).
- Serial 60 is the day that does not exist, so a floor from just below 61 lands on it and a floor from just below 60
  lands on 1900-02-28. Month and year arithmetic (DATEDIF "m"/"ym"/"md", DAYS360's end-of-February rules, YEARFRAC's
  basis 1) can differ there.
- docs/excel-semantics/dates.md: "Other date functions floor a serial just below midnight" (T-4454).
- Excel-fidelity review MINOR on #3490.

## Acceptance criteria
- [x] An excel-graph probe covers DATEDIF (units "d", "m", "y", "md", "ym", "yd"), YEARFRAC (bases 0 to 4) and
  DAYS360 (both methods) with a start or end of 59, 60 or 61 minus 0.4 s and minus 2^-22+2^-33, against the integer
  controls.
- [x] dates.md records the result with cells.
- [x] The engine matches every probe cell (or each gap has a filed task).
- [x] `make check` green; reviewers approve.

## Out of scope
- Negative WEEKNUM return types (T-4470).

## Outcome
- Probe `probes/t4574-feb29-near-midnight-date-args` (504 cells, excel-graph): DATEDIF (all six units, start and
  end), YEARFRAC (bases 0-4, start and end) and DAYS360 (both methods, start and end) at N = 59, 60, 61 minus 0.4 s,
  2^-22+2^-33, 0.6 s, `fl(N-0.5/86400)` and one ulp below it, against the integer controls.
- Found: DATEDIF's "M", "Y", "MD", "YM" and "YD" read each date with YEAR's half-second carry, while "D" floors
  (`DATEDIF(60-0.4/86400,425,"M")` is 11, from 1900-02-29; its "D" 366, from 1900-02-28). YEARFRAC basis 4 floors
  like the other bases (it does not round to the second as DAYS360 does). DAYS360 already matched.
- Review round 1 (excel-fidelity) added A20:F25: where floor and carry disagree, the floors decide a reversed pair's
  `#NUM!` and the carried dates feed the units; a date that carries past 9999-12-31 is `#NUM!` for every unit, "D"
  included.
- Engine: `datedif_value` reads the calendar units through `carried_day`; `year_fraction_parts`'s basis 4 takes the
  floored dates (the discount securities share it; no eval cell moved there). dates.md has the new entry
  "DATEDIF's calendar units carry half a second; YEARFRAC floors on every basis (around serial 60)" and the
  T-4454 entry is narrowed to DATEDIF's "D". The YEARFRAC Gate A docstring no longer says basis 4 rounds.
- Eval delta: +504 passing probe cells (91 of them off on main: 73 DATEDIF, 18 YEARFRAC basis 4); no other
  suite moved (`make eval-baseline`: 0 regressions, 0 fixed elsewhere).
