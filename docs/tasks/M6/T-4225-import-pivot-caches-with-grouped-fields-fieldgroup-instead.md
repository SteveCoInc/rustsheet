---
id: T-4225
title: "Import pivot caches with grouped fields (fieldGroup) instead of skipping them (corpus-bench 52379, 59932)"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A pivot cache whose fields carry a `fieldGroup` (date or numeric grouping, discrete groups) imports with its groups
instead of the whole cache being skipped, so its pivot tables and GETPIVOTDATA over them survive import.

## Context
- Follow-up of T-3182. T-2082 (SPEC-0015) skips a cache holding a `fieldGroup` ("a grouped field",
  `read_definition` in crates/rustsheet-xlsx/src/import/pivots.rs). After T-3182 fixed their pre-1900-03-01 dates,
  corpus-bench/52379/1_52379_input.xlsx and corpus-bench/59932/1_59932_input.xlsx (Excel-authored) still lose their
  caches for this reason (`rustsheet-eval run --suite corpus-bench --filter 52379` logs it). 59932 groups field
  `Date` into `Quarters` and `Years` (`rangePr startDate="1899-12-30T00:00:00"`, first group item `<1/0/1900`).
  T-2662's calipers workbook logged the same skip.
- Open evidence: how `rangePr` start and end dates and `groupItems` read (xlsx-storage.md "Pivot cache dates below
  serial 61 use the 1899-12-30 epoch" leaves them UNVERIFIED), and what GETPIVOTDATA addresses on a grouped field
  (excel-probe skill, excel-graph). A spec section in SPEC-0015 first if the model needs new types.

## Acceptance criteria
- [ ] The two corpus-bench workbooks import their caches with no "skipped pivot cache" line; unit tests in
      crates/rustsheet-xlsx/src/import/pivots/tests.rs pin a date group and a numeric group.
- [ ] GETPIVOTDATA over a grouped field matches an excel-graph probe, or the rule is marked UNVERIFIED in
      docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh and export of pivot caches.
