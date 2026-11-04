---
id: T-4228
title: "Probe which group Excel puts boundary values in, and GETPIVOTDATA on a grouped pivot field"
milestone: M6
lane: engine
status: done
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
- [x] An excel-graph probe covers each open point above, and web.md records the result (VERIFIED or still open).
- [x] The importer and GETPIVOTDATA match every probe cell, or the cells are known failures owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Day groups of more than one day and 1904 workbooks (the importer skips them).

## Outcome
Split (work-loop step 3): this PR settles the numeric-range points; the date-group points (23:59:59.5 or later, serials
1 to 60, a blank's group, GETPIVOTDATA on a year or omitted item) moved to T-4236, which also extends the builder for
dates and blanks.
- Case builder: a pivot may group its row field by numeric ranges (`"group": {"start", "end", "interval", "items"}`,
  `tools/src/rustsheet_tools/pivot.py` `_Group`), written as `<fieldGroup><rangePr>` with the labels the case gives.
  It renders no cells for a grouped pivot: excel-graph lays the pivot out itself on load (the probe reads Excel's
  layout through `=P!A3&""`). The eval case model accepts the key (`PivotGroupSpec`) and the engine adapter leaves a
  grouped spec pivot unregistered until T-4235.
- probes/t4228-pivot-range-groups (excel-graph, 2026-11-04): five pivots whose S values are distinct powers of two, so
  each group's sum names its records. Found (web.md "Grouped pivot fields import as their group items"): the step is
  floor of q = (v - start) / interval rounded to 15 significant digits (0.3 by 0.1 is in `0.3-0.4`; relative, not an
  absolute tolerance: pivot 5); `endNum` itself is in the last inner group but a step that rounds past it goes in
  `>end` (0.9999999999999999 is in `>1`); a number item in GETPIVOTDATA picks the group whose lower bound equals it
  (`endNum` picks `>end`, the `<start` bound and inner numbers are `#REF!`); an omitted item and `"(blank)"` are
  `#REF!`; a `<groupItems>` list of the wrong length makes Excel relabel and reorder the groups.
- The importer's `group_index` floors the unrounded quotient and has no `>end` overflow rule, so it disagrees on 0.3,
  0.6, 0.7 and the near-end values: T-4235 owns the fix and every visibly-off cell of the probe.
- Eval delta: probes +1 case, 168 cells checked, 46 pass (the `#REF!` cells), 122 known failures owned by T-4235
  (new-coverage, no regression; visible-parity lock 122 cells in the new case). No other suite changed.
- Follow-ups: T-4235 (importer step rule, GETPIVOTDATA number items, grouped spec pivots in the eval adapter), T-4236
  (date-group probe).
