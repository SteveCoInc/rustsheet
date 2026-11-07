---
id: T-5002
title: "Stage 5: keep sheet-anchored parts (drawings, notes, controls, printer settings) with their attach elements (SPEC-0034 D5.3, D5.4)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-5000, T-5001]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A regenerated save keeps a sheet's drawings (with their charts and images), legacy notes and VML, threaded comments,
form controls and printer settings. They are written with the attach elements that reference them: `<drawing>`,
`<legacyDrawing>`, `<legacyDrawingHF>`, `<picture>`, `<controls>`, `<oleObjects>`, and `pageSetup r:id`.

## Context
SPEC-0034:
- D5.1 makes these parts `ReplayIfClean`, anchored to their sheet.
- Under D5.3, a root is kept only if its attach element is a kept fragment, with its `r:id` remapped to the fresh
  relationship. Otherwise the root is dropped with `UnmappableRelationship`.
- D5.4 covers relationship-bound fragments.

T-4950 writes pivot-bound drawings from the model. Its drawings are excluded here by the closure rule: they reach a
`pivotTable`. Legacy notes are read by the importer but not exported, so D5.1's pristine-against-live comparison
applies to them.

## Acceptance criteria
- [ ] Each attach element is remapped, or its root is dropped and reported (unit tests).
- [ ] A sheet with a chart, a note, a form control and printer settings keeps all four through a fallback save
      (fixture test).
- [ ] Notes edited in the model drop the source's comments part, which is reported.
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps; findings go into docs/excel-semantics/xlsx-storage.md.
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving anchors under row and column edits (stage 5c, T-5007).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
