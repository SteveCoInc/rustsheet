---
id: T-4188
title: "Slicers and timelines as pivot filters"
milestone: M16
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4165, T-4176]
---

## Goal
Slicer and timeline caches are imported as filter state on the pivots they drive, applied by PivotResult, and preserved on export.

## Context
excel-docs.md slicer/timeline rows; x14/x15 parts. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.
- Owner of the visibly-off cells of probes/t4767-pivot-slicers (decision 0245): the engine adapter ignores a spec's
  `slicers`, so it renders the pivots unfiltered where Excel for the web applies the slicer's selection (SL, SM, SH,
  SD) and lets it win over the pivot's hidden items (SX). Also owner of probes/t4767-pivot-timelines, whose timeline
  selections Excel applies the same way (TL, TA, TD0, TD1, TX). Findings: pivots.md "Capability matrix 3" (T-4767).

## Acceptance criteria
- [x] Probes pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- Drawing slicers.

## Outcome
Not split. probes/t4767-pivot-slicers (47 cells) and probes/t4767-pivot-timelines (41) pass: their 24 failing cells
(SL, SM, SH, SD, SX; TL, TA, TD0, TD1, TX) are fixed and leave eval/baseline/probes.txt.

Design:
- **Model** (crates/rustsheet-engine/src/pivot/slicer.rs, `PivotField`): a slicer's selection is the field's
  `slicer` bitset of unselected cache items, which replaces `hidden` in what the pivot shows (`PivotField::excludes`,
  used by `result::filter`, the `showAll` axis and the page cell); `hidden` stays the `item@h` the export writes. A
  timeline's selection is the field's `timeline` (`DateSpan`, whole days, both ends kept): `result::filter` keeps the
  records whose day it spans, on an axis or not, and `pivot::filter` drops the pivot's own `<filters>` on that field.
- **Import** (crates/rustsheet-xlsx/src/import/pivots/slicers.rs): the workbook's extension list
  (`x14:slicerCaches`, `x15:timelineCacheRefs`, `x15:slicerCaches`) names the cache parts; each is streamed, linked
  to its pivots by `pivotTables/pivotTable` `tabId` and name and to its field by `sourceName`, and applied one at a
  time, all bounded (1,000 caches, `MAX_PIVOTS` pivots and `MAX_ITEMS` items per cache, the text limit on names).
  OLAP slicers, table slicers and a `timelinePivotFilter` are skipped and logged (not modelled).
- **Eval adapter**: the eval does not read the built xlsx; it builds each defined pivot's parts with the Rust mirror of
  the case builder and loads them through the importer (`Workbook::load_pivot_parts`). The mirror now writes the slicer
  and timeline cache parts too (crates/rustsheet-eval/src/pivot_definition/slicers.rs, mirroring
  tools/src/rustsheet_tools/pivot_slicers.py; `SheetSpec` gains `slicers` and `timelines`), which `PivotParts` carries
  to the importer. New golden parts in tools/tests/slicer_parts_golden/ pin the two builders to each other
  (tools/tests/test_slicer_parts_golden.py and the Rust module's tests).
- **Export**: nothing new. The source-preserving save keeps every slicer and timeline part byte for byte in its
  verbatim and patched tiers, as it keeps any part the model does not hold; a regenerated save writes none, since how
  Excel saves slicer state is unprobed (T-4956). Filed as T-4963.

Findings: pivots.md "Slicers and timelines filter the pivot they drive" (new), and "Capability matrix 3"'s
Implemented-in line. Follow-ups: T-4962 (probe the engine's unevidenced readings: off-axis, page and column slicers,
unlisted or no selected items, two caches on one field, timeline day bounds and non-date records, a timeline beside
other filter kinds, grouped fields, one cache over two pivots) and T-4963 (keep the parts on a regenerated save,
after T-4956). The visible-parity lock still holds 14 and 10 for the two cases: `make eval-baseline` never writes a
drop, and the steward's daily `--lower-visible-lock` takes the slack back (decision 0093).

Review round 1: GETPIVOTDATA refuses a pivot that a slicer or timeline drives (`pivot_index::supported`), as it
refuses one with hidden items, rather than answer from unfiltered records. A slicer cache's selection is a bitset built
once per part, applied to each distinct pivot it names (a repeated `pivotTable` entry, `r:id` or part is read once).
`result::filter` tests each record against one bitset per field, a timeline's built once over the field's cache items.
No benchmark numbers: crates/rustsheet/benches/pivot.rs was not run in the review's time limit.
