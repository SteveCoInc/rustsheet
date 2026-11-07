---
id: T-4624
title: "DATEDIF calendar units when the floored and second-rounded date orders differ: probe and match"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Settle what is still unevidenced about DATEDIF's near-midnight reads on `main`: "Y"'s reading (floored or carried), the
reversed order when start and end share a floored day but the carried dates are reversed (e.g. start N-0.4 s,
end N-0.6 s), and any past-the-calendar cell not covered by probes/t4574-feb29-near-midnight-date-args A20:F25.

## Context
- On `main` (T-4574, #3647) `datedif_value` (crates/rustsheet-engine/src/functions/date_and_time.rs) checks the
  reversed pair on the floored serials and reads the calendar units, "Y" included, and the calendar check through
  `carried_day` (`floor(x + 0.5/86400)`). This is the same cut as rounding to the nearest second (dates.md "DATEDIF's
  calendar units carry half a second"). "D" past the calendar is #NUM! and is evidenced
  (`DATEDIF(1,2958466-0.4/86400,"D")`, A24:F25 of that probe); do not restore a floored "D" count there.
- Not evidenced: "Y" (every cell of probes/t4574-near-midnight-calendar-units gives one value either way, I1:J8);
  `DATEDIF(N-0.4/86400,N-0.6/86400,u)` (floors equal, carried dates reversed) at other N than 59/60/45020; an end
  just below 2958466 for the units other than "D" at the top of the calendar.

## Acceptance criteria
- [x] An excel-graph probe covers every unit with start and end on one floored day in reversed carried order, and
  with an end just below 2958466, against integer controls.
- [x] The probe tells "Y"'s reading apart, e.g. `DATEDIF(44927,45292-0.4/86400,"y")`: 0 if floored, 1 if carried
  (the engine carries it today, unevidenced).
- [x] dates.md records the result with cells; the engine matches every probe cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- The ordinary near-midnight reads (T-4574).

## Outcome
Probe probes/t4624-datedif-reversed-and-calendar-end (60 cells, excel-graph) settles all three questions: "Y" carries
(1, not 0), the carried-reversed pair reads y -1, m -1, d 0, ym 11, md 30, yd 365 at every N, and an end or start just
below 2958466 is #NUM! in every unit. The engine already matched every cell, so no engine change; dates.md records the
cells. Eval delta: 60 new passing probe cells, no regressions.
