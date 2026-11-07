---
id: T-5009
title: "Stage 5: keep slicer and timeline groups with their caches when their pivot or table is unchanged (SPEC-0034 D5.2, D5.5)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-5002]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A regenerated save keeps each slicer group as one unit while the pivot or table it filters is unchanged. The group is
the `slicer` part, the drawing holding its `sle:slicer` graphic frame, the sheet's `x14:slicerList` ext and the
workbook's `slicerCache`. Timelines are kept the same way (`timeline`, `x15:timelineRefs`, `timelineCache`).

## Context
SPEC-0034 D5.2, the id-bound class of D5.4, and D5.5's `PivotOrTableEdit` trigger. A slicer cache names its table by
`tableId` and its pivot by `pivotCacheId`, and the export numbers tables from 1 (`export/tables.rs`). The group is
kept only while the export writes the same ids; otherwise it is dropped. The importer reads slicer caches for pivot
filtering (`import/pivots/slicers.rs`), so a change to their model view is a trigger too.

## Acceptance criteria
- [ ] The group is kept whole with its attach elements remapped, or dropped whole and reported (unit tests).
- [ ] A table slicer and a pivot slicer survive a fallback save when their targets are unchanged. They are dropped with
      `StaleAfterEdit` when their ids or their pivot change (fixture tests).
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps. Findings go into docs/excel-semantics/xlsx-storage.md (D5.8).
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing slicers from the model.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
