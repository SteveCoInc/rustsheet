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
- it is emitted at its `CT_Worksheet` position, or for `mc:AlternateContent`, an element with no slot or an
  unknown-namespace element, by D5.4's placement rules;
- each fragment carries its namespace declarations, counting prefixes named in `mc:` attribute values;
- `mc:Ignorable` is rebuilt.

This task builds the one classifier (relationship-bound, range-bound, sheet-name-bound, id-bound, inert) that T-5003
reuses. Relationship-bound fragments are dropped and reported until T-5002 and T-5008 remap them. Id-bound fragments
are kept only while the export writes the same id.

## Acceptance criteria
- [ ] The owned set is declared next to the worksheet writer, and the classifier lives in one module.
- [ ] Fragments are emitted in schema order next to regenerated siblings, including an `mc:AlternateContent` block and a
      no-slot element (unit tests).
- [ ] Data validation, auto-filter, page setup and an unknown ext survive a fallback save (fixture tests).
- [ ] Hostile input (depth, count, emitted bytes, a malformed fragment) drops the fragment with `LimitExceeded` and
      never panics.
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps. Findings go into docs/excel-semantics/xlsx-storage.md (D5.8).
- [ ] The round-trip baselines are rerecorded; entries only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Elements the writer owns but writes differently (`sheetViews`, `cols`): stage 4 models them.
- Invalidation (T-5004).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
