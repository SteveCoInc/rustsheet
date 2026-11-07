---
id: T-4962
title: "Probe the engine's slicer and timeline readings: off-axis and page slicers, unlisted items, two caches on one field, timeline days"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4188]
---

## Goal
Each slicer and timeline reading the engine makes without evidence is probed on Excel for the web, and the engine
follows what it shows.

## Context
T-4188 applies slicers and timelines as pivot filters (docs/excel-semantics/pivots.md "Slicers and timelines filter
the pivot they drive"). probes/t4767-pivot-slicers and probes/t4767-pivot-timelines evidence one tabular slicer on a
row field and one timeline on a field on no axis. The entry's UNVERIFIED list is the engine's own reading for:
- a slicer on a field on no axis, on the page axis (and whether the page cell shows `(Multiple Items)`), or on a
  column field;
- a slicer cache that leaves an item of the field unlisted, or selects no item;
- a timeline on a field on an axis, beside that field's hidden items;
- a timeline cache with no `selection` beside the pivot's own filter on its field (the engine sets no timeline, so
  the filter still applies);
- two slicer caches on one field of a pivot, and two timelines on one field;
- a timeline over a date-time on the selection's last day, and over a text or blank record;
- a timeline beside a caption or value filter on its field (not only a date filter);
- a slicer on a grouped field (its `x` over the group items);
- one slicer cache driving two pivots of one pivot cache (`pivotTables` with two entries), and a `showAll` field
  under a slicer.
The probes are spec cases built by tools/src/rustsheet_tools/pivot_slicers.py (it may need an entry key for a
second pivot in `pivotTables`); the engine code is crates/rustsheet-engine/src/pivot/slicer.rs, `result::filter`
and crates/rustsheet-xlsx/src/import/pivots/slicers.rs.

## Acceptance criteria
- [ ] A probe answers each reading above, and pivots.md records the answers (the UNVERIFIED list shrinks).
- [ ] The engine matches every new probe; owners for any cell it cannot match yet (decision 0245).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Saving slicer state (T-4963, after T-4956).
