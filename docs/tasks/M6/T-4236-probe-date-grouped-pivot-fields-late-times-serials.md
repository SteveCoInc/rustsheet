---
id: T-4236
title: "Probe date-grouped pivot fields: late times, serials 1 to 60, blanks, and GETPIVOTDATA on a year or omitted item"
milestone: M6
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The date-group open points T-4228 left are settled by an excel-graph probe, recorded in web.md, and the importer and
GETPIVOTDATA match it or the cells are owned.

## Context
- Split from T-4228, which probed numeric range groups (probes/t4228-pivot-range-groups) and extended the case builder
  with a `group` key for them (tools/src/rustsheet_tools/pivot.py `_Group`). The builder writes no date grouping and
  refuses blanks in a pivot source, so extend it first: `groupBy` months, quarters, years, hours with `startDate` and
  `endDate`, source cells holding dates (Excel groups a field by date only when the source cells are dates: check the
  spec's number formats), and blank source cells.
- What the T-4228 probe showed that carries over: excel-graph lays a grouped pivot out itself on load (the builder renders
  no cells), and a `<groupItems>` list whose count is not Excel's layout makes Excel rebuild the labels in another order
  (pivot 3 of the probe), so write exactly the layout's items (web.md "Grouped pivot fields import as their group items").
- Open points (web.md, same entry): a time of 23:59:59.5 or later (the importer splits a serial as HOUR and YEAR do, so it
  is the next day); serials 1 to 60 (the 1900 leap-year bug); which group item a blank goes in (the importer uses item 0,
  which corpus-bench 59932's month groups label `(blank)`); GETPIVOTDATA with a group label (`"Quarters","Qtr1"`), a year
  as text and as a number, and an omitted item.

## Acceptance criteria
- [x] An excel-graph probe covers each open point above, and web.md records the result (VERIFIED or still open).
- [x] The importer and GETPIVOTDATA match every probe cell, or the cells are known failures owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Day groups of more than one day and 1904 workbooks (the importer skips them).

## Outcome
- The case builder (tools/src/rustsheet_tools/pivot.py) writes date groups (`"by"` with `startDate`/`endDate` from
  date serials, the row field cached as `<d>` dates on the OLE epoch) and blank cells in a grouped row field (`<m/>`);
  the eval case schema takes the `by` key (crates/rustsheet-eval/src/case.rs `PivotGroupSpec`).
- Two excel-graph probes: probes/t4236-pivot-date-groups (months, hours, quarters, years, days; four pivots hold a
  blank) and probes/t4236-pivot-date-groups-no-blank (the same with the blanks replaced). Findings (web.md "Grouped
  pivot fields import as their group items", date-group bullets, VERIFIED): the cache rounds a date to the second and
  compares the rounded date to start and end; a date equal to `endDate` is inside; serials below 61 group by the 1900
  calendar (60 is `29-Feb`, in `Feb`); a refresh drops the date grouping of a field holding a blank (so which group a
  blank takes is not probeable); GETPIVOTDATA number items pick the group by calendar value (hour, leap-year day,
  month, quarter, year; integer part), labels are case-folded, date serials and `"(blank)"` are `#REF!`.
- Importer: `group_index` compares the second-rounded date with `startDate`/`endDate` (2018-12-31 23:59:59.5 was
  below a 2019 start; now `Jan`), with a unit test.
- Eval delta: two new probe cases, 684 checked cells (376 pass, 308 known failures in eval/baseline/probes.txt and
  the visible-parity lock); the failing cells are owned by T-4246
  (number items, the dropped grouping over blanks, and the adapter's date groups; it depends on T-4235). No existing
  case changes.
- Not probed: the decimal label format of numeric range groups (web.md, still UNVERIFIED).
