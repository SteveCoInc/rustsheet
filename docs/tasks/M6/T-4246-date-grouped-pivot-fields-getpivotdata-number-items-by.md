---
id: T-4246
title: "Date-grouped pivot fields: GETPIVOTDATA number items by calendar value, and the eval adapter registers date-grouped spec pivots"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-4235]
---

## Goal
GETPIVOTDATA on a date-grouped pivot field answers as Excel does, and the eval engine adapter registers a date-grouped
spec pivot, so every cell of probes/t4236-pivot-date-groups and probes/t4236-pivot-date-groups-no-blank passes.

## Context
- Follow-up of T-4236, whose excel-graph probes established the rules (docs/excel-semantics/web.md "Grouped pivot
  fields import as their group items", the date-group bullets). The importer's record-to-group rule already matches
  them (`group_index` in crates/rustsheet-xlsx/src/import/pivots/groups.rs compares the date rounded to the second).
- GETPIVOTDATA on a date-grouped field: a text item is the group label, case-folded (`"jan"`, `"12 am"`, `"qtr1"`,
  `"1-mar"`), and the `<start`/`>end` labels work while their group has a record. A **number** item picks the group
  whose calendar value equals its integer part: hour 0 to 23 (`0` and `TIME(10,0,0)` are `12 AM`, `10` is `10 AM`),
  leap-year day 1 to 366 (`60` is `29-Feb`, `61` is `1-Mar`), month 1 to 12, quarter 1 to 4, year (`2018`, `2018.5`);
  a date serial (`DATE(2019,1,1)`, `43525`), a text number (`"1"`, `"10"`), `TRUE`, `"(blank)"` and an omitted item
  are `#REF!`. Today the importer turns the groups into text items, so number items are `#REF!`.
- On refresh Excel drops the date grouping of a field holding a blank source cell (probes/t4236-pivot-date-groups:
  every pivot with a blank renders its dates ungrouped, the blank as `(blank)`, and `"(blank)"`, an omitted item and a
  date serial item then work). The adapter builds spec pivots from their source, as a refresh does, so it should do
  the same; the cache also rounds each date to the second (two dates within a second are one item), and lists the
  ungrouped items in record order, `(blank)` at its record's place (web.md).
- The adapter must group from the source cells, not from a cache the builder writes: `pivot.py` `_iso` truncates a
  date to the second where Excel rounds it (23:59:59.5 is the next day). The blank case's rendered reads stop at row
  48 (Calc rows 41 to 58), which cuts off the ungrouped Q and Y layouts' last rows; extend the reads if it refills.
- The adapter part extends T-4235's registration of grouped spec pivots (share `apply_groups`; do not copy it).
- Owns every visibly-off cell of probes/t4236-pivot-date-groups and probes/t4236-pivot-date-groups-no-blank.

## Acceptance criteria
- [ ] GETPIVOTDATA number items on a date-grouped field match the probes' item cells.
- [ ] Both probes pass in the eval (the adapter registers date-grouped spec pivots and drops a grouping over blanks);
      baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Day groups of more than one day and 1904 workbooks (the importer skips them).
