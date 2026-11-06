---
id: T-4574
title: "DATEDIF, YEARFRAC and DAYS360 just below midnight at serials 59, 60 and 61 (the fictitious 1900-02-29): probe and match"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: opus
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
- [ ] An excel-graph probe covers DATEDIF (units "d", "m", "y", "md", "ym", "yd"), YEARFRAC (bases 0 to 4) and
  DAYS360 (both methods) with a start or end of 59, 60 or 61 minus 0.4 s and minus 2^-22+2^-33, against the integer
  controls.
- [ ] dates.md records the result with cells.
- [ ] The engine matches every probe cell (or each gap has a filed task).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Negative WEEKNUM return types (T-4470).

## Handoff (resumed)
Probe `probes/t4574-near-midnight-serial-60` (234 cells, excel-graph 2026-11-06, committed with its oracle sidecar) is filled; the engine passes 198 and
fails 36, so this is beyond a probe-and-match for sonnet. Rows 1-9 are s = 59-0.4s, 59-(2^-22+2^-33), 59, 60-0.4s, 60-(2^-22+2^-33), 60, 61-0.4s,
61-(2^-22+2^-33), 61. `make eval-show` lists the failing cells; the patterns:
- DATEDIF "m", "ym", "md", "yd" (D, J, G/H, K/L): Excel gives the same value for 59-eps as for 59 (e.g. G1:G3 = 15, 15, 15; H7:H9 = 0, 0, 0; D7:D9 = 2, 2, 2),
  i.e. it does NOT floor the serial for these units here (it reads the next day), while "d" and "y" (and the engine) floor. The engine floors for all units.
- YEARFRAC basis 4 (U, V): the opposite. Excel floors (U1 = 123.05 = the value for serial 58, U3 = 123.0472 for 59); the engine reads 59-0.4s as 59. Other bases pass.
- DAYS360 (W:Z) passes.
Next: probe at a modern base with N-0.4 day, N-0.6 day and N-0.4 s to separate ceil, round and floor for DATEDIF's calendar units, find where the
engine's DATEDIF/YEARFRAC basis 4 take the serial from (`datedif_value` floors up front; check `yearfrac` basis 4), implement, record the rule in dates.md
"Other date functions floor a serial just below midnight", and baseline. The probe's off cells need owners (`scripts/new_case_owners.py`) until fixed. Do not
treat the rule as a plain floor: the evidence says it is per unit.
