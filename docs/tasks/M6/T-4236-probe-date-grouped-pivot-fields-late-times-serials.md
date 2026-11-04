---
id: T-4236
title: "Probe date-grouped pivot fields: late times, serials 1 to 60, blanks, and GETPIVOTDATA on a year or omitted item"
milestone: M6
lane: engine
status: todo
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
- [ ] An excel-graph probe covers each open point above, and web.md records the result (VERIFIED or still open).
- [ ] The importer and GETPIVOTDATA match every probe cell, or the cells are known failures owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Day groups of more than one day and 1904 workbooks (the importer skips them).
