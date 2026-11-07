---
id: T-5008
title: "Stage 5: keep a sheet's legacy drawing group (VML, notes, threaded comments, form and ActiveX controls, OLE objects) as one unit (SPEC-0034 D5.2)"
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
A regenerated save keeps each sheet's legacy drawing group as one unit. The group is the VML of `<legacyDrawing>`, the
legacy `comments`, the `threadedComment` parts and the workbook `person` list, every `ctrlProp` with `<controls>`,
ActiveX `control` parts, `oleObject`/`package` embeddings with `<oleObjects>`, and the drawing holding their
`a14:compatExt` shapes.

## Context
SPEC-0034 D5.2. Excel ties these together by shape ids that no relationship records:
- `<control shapeId>` names VML `_x0000_s<n>`, and the drawing's `compatExt spid` names the same shape;
- one VML part holds both note shapes and control shapes (corpus-bench 17-35);
- a threaded comment needs its legacy comment (calipers officejs/comment_add).

Keeping a member without the others risks an Excel repair. ActiveX and OLE are kept in a plain `.xlsx`, as Excel
keeps them (poi/45540_form_Footer.xlsx, poi/WithEmbeded.xlsx). Legacy notes are read by the importer, so a
`NotesEdit` (D5.5) drops the whole group.

## Acceptance criteria
- [ ] The group is kept whole, with every attach element remapped (`<legacyDrawing>`, `<legacyDrawingHF>`,
      `<controls>`, `<oleObjects>`, including inside `mc:AlternateContent`), or dropped whole and reported (unit tests).
- [ ] A sheet with notes, a threaded comment, a form control and an OLE object keeps all four through a fallback save
      (fixture test).
- [ ] The group can be dropped as one unit through the call T-5004's `NotesEdit` trigger uses (unit test of the drop;
      the trigger and its scoring are T-5004's).
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps. Findings go into docs/excel-semantics/xlsx-storage.md (D5.8). ActiveX and OLE retention also gets a desktop Excel check (M14) before its entry is VERIFIED (D5.7).
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing notes from the model.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
