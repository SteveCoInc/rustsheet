---
id: T-5003
title: "Stage 5: workbook.xml and styles.xml fragment slots, with the absPath option (SPEC-0034 D5.4)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-4999]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
Regenerated `workbook.xml` and `styles.xml` re-emit the source's unmodelled elements, attributes and `ext` entries.
Examples are fileVersion, workbookPr attributes the model lacks, fileRecoveryPr, `x15ac:absPath`, `colors`,
`tableStyles`, and the slicer-style ext. `SaveOptions::strip_abs_path` drops the absolute path.

## Context
SPEC-0034 D5.4, using T-5001's classifier. Decision 0275 D6 keeps `x15ac:absPath` as Excel keeps it, with an option
to strip it. Index-bound style fragments (a `dxfId`) are kept only while the dxf section is prefix-stable.

## Acceptance criteria
- [ ] The owned sets are declared next to the workbook and styles writers.
- [ ] The fragments survive a fallback save, in schema order (unit and fixture tests).
- [ ] `strip_abs_path` drops the path, and only the path (unit test).
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps; findings go into docs/excel-semantics/xlsx-storage.md.
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sheet entries for chartsheets, dialogsheets and macrosheets (they stay `NotPreserved`, D5.1).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
