---
id: T-4225
title: "Import pivot caches with grouped fields (fieldGroup) instead of skipping them (corpus-bench 52379, 59932)"
milestone: M6
lane: engine
status: done
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
- [x] The two corpus-bench workbooks import their caches with no "skipped pivot cache" line; unit tests in
      crates/rustsheet-xlsx/src/import/pivots/tests.rs pin a date group and a numeric group.
- [x] GETPIVOTDATA over a grouped field matches an excel-graph probe, or the rule is marked UNVERIFIED in
      docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh and export of pivot caches.

## Outcome
- A grouped pivot cache field imports as an ordinary cache field: its items are the `<groupItems>` labels and its
  records each record's group, derived from the `base` field (`apply_groups`, new
  crates/rustsheet-xlsx/src/import/pivots/groups.rs). This covers `rangePr` numbers and dates (seconds to years),
  `discretePr` lists, a field grouped in place, and the `databaseField="0"` grouping fields, which Excel writes after
  the database fields and leaves out of the records part. The engine model gets no new type. SPEC-0015's import section
  and unknown 12 say so.
- corpus-bench 52379 (`Date/Time` by minutes, `Hours`, `Days`) and 59932 (`Date` by months, `Quarters`, `Years`) now
  import their caches and pivots with no skip line. On main both logged `skipped pivot cache ...: "a grouped field"`.
- Unit tests in import/pivots/tests.rs pin a date group (months in place, quarters and years), an hour group, a numeric
  step group, a discrete group, a blank base value, and the malformed layouts that still skip the cache.
- Quirk recorded: web.md "Grouped pivot fields import as their group items". It gives the group-item layouts Excel
  writes (counts per `groupBy`, 29-Feb always present) and 59932's `(blank)` first month item. Which group Excel puts
  each value in, and GETPIVOTDATA on a grouped field, are UNVERIFIED. Excel stores no record's group, so the caches
  cannot show it. xlsx-storage.md now records that `rangePr` dates are read on the pivot-cache date rule.
- Still skipped, logged: day groups of more than one day (date ranges), date groups in 1904 workbooks, an item count
  other than the layout's, and a range group over text.
- Follow-up: T-4228 (opus) probes the group boundaries and GETPIVOTDATA on a grouped field.
- Eval delta: none. No gated cell reads a grouped pivot (corpus-bench is crash-only for these two workbooks), and there
  are 0 regressions.
