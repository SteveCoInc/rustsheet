---
id: T-5071
title: "Record table slicer caches in the oracle sidecar snapshot"
milestone: M16
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4956]
---

## Goal
An Excel-saved file's oracle sidecar records its table slicer caches (`x15:tableSlicerCache`) and their slicer
shapes, so the pivots.md "Capability matrix 3" claim that Excel for the web keeps every `tableSlicerCache` attribute
can cite a committed sidecar.

## Context
- T-4956 made the snapshot record each slicer and timeline cache on the pivots it drives (`slicers`, `timelines` of a
  sidecar pivot; crates/rustsheet-eval/src/xlsx_snapshot/pivots/slicers.rs). A table slicer's cache names no pivot, so
  no sidecar section keeps it: probes/t4767-pivot-slicers TS is unrecorded, and the claim stays UNVERIFIED.
- Needs a workbook-level sidecar section (snapshot field, `tools/src/rustsheet_tools/oracle/cli.py` fill, and
  `OracleFile` in crates/rustsheet-eval/src/case.rs), reusing `XmlElement` and `PivotSlicer`; then refill
  probes/t4767-pivot-slicers and move the claim.

## Acceptance criteria
- [ ] The sidecar records every slicer cache that drives no pivot (a table slicer's), with its shapes.
- [ ] probes/t4767-pivot-slicers is refilled and the pivots.md tableSlicerCache claim cites it (VERIFIED or corrected).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing table slicers on export.
