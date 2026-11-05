---
id: T-4261
title: "Pivot date cache rounding at a binary-noise half second: probe and model it (after T-4246)"
milestone: M6
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4246]
---

## Goal
A date whose time sits a binary-noise hair below a half second goes in the date group Excel's pivot cache puts it
in. Every cell of probes/t4236-pivot-date-groups-no-blank that T-4246 left visibly off passes.

## Context
- probes/t4236-pivot-date-groups-no-blank pivot H (hours from 2019-01-01): Data!D4 is 43466.458327546294, written
  as 10:59:59.5. Its time of day is 39599.49999978 s, 2.2e-7 s below the half, and Excel puts it in `11 AM`
  (Calc E45 "11 AM", F13 4). `time_of_day_parts`, which matches HOUR (dates.md, probes/second-rounding: "an exact
  half goes whichever way the serial's binary noise takes it"), rounds it down, so `DateGroups::item`
  (crates/rustsheet-engine/src/pivot/date_groups.rs `split_serial`) gives `10 AM`. That one point is behind all
  15 visibly-off cells: E45:E49, F44:F49 (the layout shifts up a row), and F12, F13, F29 and F33.
- So the pivot cache rounds this date differently from the engine's HOUR model (HOUR of this serial is itself
  unprobed; include it in the probe). Two candidate models fit every probed point: round the
  time to the millisecond first and then to the second (half up), or round the serial to 15 significant digits
  before splitting it. Choosing between them is a structural model: decision 0188 R4 asks for a hold-out and a
  fresh sweep. Probe serials just below and above x.5 s at several magnitudes, where the two models disagree,
  for example a 1900 date (serial < 100, where 15 digits keep far more decimals) against a 2019 one.
- The same rule applies to the importer (it shares `DateGroups`), and the `startDate` and `endDate` comparison.

- probes/t4236-pivot-date-groups pivot H (the same Data!D4 point, ungrouped since T-4260 drops the grouping over
  its blank): the point merges with 10:59:59 into one item, so Calc E44:E50 and F43:F50 (15 cells) are off by a row.
  Those cells are this task's too.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) separates the candidate models; the code follows the winner,
      and web.md "Grouped pivot fields import as their group items" records the rule.
- [ ] probes/t4236-pivot-date-groups-no-blank and probes/t4236-pivot-date-groups pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- HOUR, MINUTE and SECOND, whose rounding is probed and differs (dates.md).
