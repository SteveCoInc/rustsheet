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
A regenerated save keeps a sheet's drawings with their charts, images and chart style parts, its background picture
and its printer settings. Each is written with the attach element that references it: `<drawing>`, `<picture>`, or
the `r:id` of `pageSetup`.

## Context
SPEC-0034:
- D5.1 makes these `ReplayIfClean` and attach-bound;
- D5.3 remaps the attach element's `r:id` to the fresh relationship, or drops the root with `UnmappableRelationship`.

D5.2 excludes two kinds of drawing:
- a drawing holding `a14:compatExt` shapes is part of the legacy-drawing group, handled by T-5008;
- a drawing whose charts hold `c:pivotSource` is T-4950's, written from the model. Pivot charts are bound by that text,
  not by a relationship.

## Acceptance criteria
- [ ] Each attach element is remapped, or its root is dropped and reported (unit tests).
- [ ] A sheet with a chart, an image, a background picture and printer settings keeps all four through a fallback save
      (fixture test).
- [ ] A pivot chart's drawing is left to T-4950, and a compatExt drawing is left to T-5008 (unit tests).
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps. Findings go into docs/excel-semantics/xlsx-storage.md (D5.8).
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving anchors under row and column edits (stage 5c, T-5007).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
