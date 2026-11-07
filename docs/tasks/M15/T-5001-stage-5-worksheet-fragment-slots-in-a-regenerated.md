---
id: T-5001
title: "Stage 5: worksheet fragment slots in a regenerated save (SPEC-0034 D5.4)"
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
A regenerated `sheetN.xml` re-emits the source sheet's unmodelled elements in their schema slots. Examples are
sheetPr, sheetFormatPr, printOptions, pageMargins, pageSetup without an `r:id`, headerFooter, dataValidations,
autoFilter, protection, ignoredErrors, and unknown `ext` entries.

## Context
SPEC-0034 D5.4:
- the worksheet writer declares the elements, attributes and `ext` URIs it owns, and everything else is a fragment;
- a fragment is read from the source part at save time with the patcher's bounded scan;
- it is emitted at its `CT_Worksheet` position, with its namespace declarations, and `mc:Ignorable` is rebuilt.

Fragments are classified as relationship-bound, range-bound, sheet-name-bound, index-bound or inert. This task emits
the inert, range-bound and sheet-name-bound fragments. Relationship-bound ones are dropped and reported until T-5002.
Index-bound ones are kept only while D3.3's prefix test holds.

## Acceptance criteria
- [ ] The owned set is declared next to the worksheet writer. One classifier serves all three parts (T-5003 reuses it).
- [ ] Fragments are emitted in schema order next to regenerated siblings (unit tests). Data validation, auto-filter,
      page setup and an unknown ext survive a fallback save (fixture tests).
- [ ] Hostile input (depth, count, a malformed fragment) drops the fragment with `LimitExceeded`, never panics.
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps; findings go into docs/excel-semantics/xlsx-storage.md.
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Elements the writer owns but writes differently (`sheetViews`, `cols`): stage 4 models them.
- Invalidation (T-5004).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
