---
id: T-4228
title: "Probe which group Excel puts boundary values in, and GETPIVOTDATA on a grouped pivot field"
milestone: M6
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The grouped-field import of T-4225 is checked against Excel: an excel-graph probe shows which group Excel puts boundary
values in and what GETPIVOTDATA answers on a grouped field, and the importer follows it.

## Context
- Follow-up of T-4225. Since T-4225 a grouped pivot cache field imports as its group labels, with each record's group
  derived by ECMA-376's layout (crates/rustsheet-xlsx/src/import/pivots/groups.rs `apply_groups`). Excel stores no
  record's group, so the Excel-authored caches (corpus-bench 52379, 59932) cannot show the rule. web.md "Grouped pivot
  fields import as their group items" lists it as UNVERIFIED.
- Open points:
  - numeric steps at `startNum`, `endNum` and the step edges, including steps a double cannot hold exactly (0.3 from
    0 by 0.1, where the floor gives step 2);
  - a time of 23:59:59.5 or later (the importer splits a serial as HOUR and YEAR do, so it is the next day);
  - serials 1 to 60;
  - which group item a blank goes in (the importer uses item 0, which 59932's month groups label `(blank)`);
  - GETPIVOTDATA with a group label (`"Quarters","Qtr1"`), a year as text and as a number, and an omitted item.
- The probe needs a workbook whose pivot is grouped. The spec→xlsx builder writes pivots
  (tools/src/rustsheet_tools/pivot.py) but not `<fieldGroup>`, so extend it first (the excel-probe skill). This goes in probes, not probes-numeric
  (decision 0267 item 5 freezes only probes-numeric).

## Acceptance criteria
- [ ] An excel-graph probe covers each open point above, and web.md records the result (VERIFIED or still open).
- [ ] The importer and GETPIVOTDATA match every probe cell, or the cells are known failures owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Day groups of more than one day and 1904 workbooks (the importer skips them).
