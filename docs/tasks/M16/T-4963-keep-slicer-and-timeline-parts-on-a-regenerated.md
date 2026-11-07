---
id: T-4963
title: "Keep slicer and timeline parts on a regenerated save"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4188, T-4956]
---

## Goal
A workbook whose pivots are driven by slicers or timelines keeps them when the save regenerates the package, written
as Excel saves them.

## Context
T-4188 imports slicer and timeline caches as filter state on the pivots they drive (pivots.md "Slicers and timelines
filter the pivot they drive"), but keeps no part: the source-preserving export (SPEC-0034) keeps every slicer,
timeline, slicer cache and timeline cache part, the drawing frames and the workbook and sheet extension entries only
in its verbatim and patched tiers. A regenerated save (crates/rustsheet-xlsx/src/export/preserving.rs) writes none,
so the reopened file shows its pivots unfiltered. How Excel saves a slicer's state is UNVERIFIED until T-4956 makes
the sidecar snapshot record it (pivots.md "Capability matrix 3": Excel was seen to save a slicer's unselected items
as the pivotField's `h="1"`, and a timeline's selection as a `dateBetween` pivot filter with `useWholeDay="1"`).
Size it again once T-4956 is done: it may need a spec (directive 4) if the parts are to be kept verbatim beside a
regenerated package.

## Acceptance criteria
- [ ] A regenerated save of probes/t4767-pivot-slicers and probes/t4767-pivot-timelines reopens with the same
  rendered pivots, in Excel for the web and in the engine.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Drawing slicers.
