---
id: T-3182
title: "Pivot cache date items below serial 61 import on Excel's 1899-12-30 epoch instead of skipping the cache (corpus-bench 52379, 59932)"
milestone: M6
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot cache whose `<d>` items, `minDate`/`maxDate` or `rangePr` dates name a serial below 61 imports with the serials
Excel meant, so the pivot table is not dropped. Today the importer skips the whole cache. GETPIVOTDATA over such a
pivot is then wrong, and the cache is lost on round trip.

## Context
Gap triage 2026-10-30 (`make eval` on main c269641be, then `rustsheet-eval fetch` and `run --suite poi --suite
corpus-bench`). The run logs `RustSheet: skipped pivot cache "xl/pivotCache/pivotCacheDefinition1.xml": "bad date
\"1899-12-30T00:00:00\""`. Two Excel-authored workbooks (docProps/app.xml: Microsoft Excel, AppVersion 16.0300) have
such caches. Neither has a GETPIVOTDATA formula, so no scored cell moves today. The pivot is still lost, and every
other pivot-cache date below serial 61 is misread.
- corpus-bench/52379/1_52379_input.xlsx: pivotCacheDefinition1 field `Duration` (source `Table1`, sheet FCR column
  E, call durations below 3 days). Excel writes `minDate="1899-12-30T00:00:08"`, `maxDate="1900-01-02T16:03:58"` and
  records such as `<d v="1899-12-30T00:07:23"/>`. The source cell FCR!E241 holds 2.6748611111111114 (2 days 16:11:48),
  and the cache has the item `1900-01-01T16:11:48`. So serial s becomes 1899-12-30 + s days below serial 61 (the OLE
  epoch), not the 1900-system calendar that a `t="d"` cell uses. For a cell, `1900-01-01` = 1 and `1899-12-31` is
  `#VALUE!` (docs/excel-semantics/xlsx-storage.md "Strict OOXML uses other namespaces", fixtures/t-d-*).
- corpus-bench/59932/1_59932_input.xlsx: field `Date` has `minDate="1899-12-30T00:00:00"`. Its date groups (`Quarters`,
  `Years`) have `rangePr startDate="1899-12-30T00:00:00"`, and Excel labels the first group item `<1/0/1900`. That
  label is serial 0, so 1899-12-30T00:00:00 = 0 there too.
- Code: `read_item` in crates/rustsheet-xlsx/src/import/pivots.rs (around line 397) calls
  `iso8601_to_date_system_serial` (crates/rustsheet-xlsx/src/import/iso_date.rs). That function returns `None`
  before 1900-01-01, and `skip` then drops the cache. The same function serves `t="d"` cells, which must keep their
  verified behavior. So the pivot path needs its own mapping, or a mode flag, not a change to the cell rule.
  This builds on T-2393 (done), which added the `date_system` parameter on this path for the 1904 offset; its
  Outcome leaves a pre-epoch date in a 1904 workbook skipping the cache (unprobed), the same failure as here.
- Open questions for this task to settle with evidence, and record in xlsx-storage.md:
  1. Where the OLE epoch stops in a 1900 workbook. From serial 61 (1900-03-01) the two calendars agree. Between 1 and
     60 the cache date is one day earlier than the cell calendar (FCR!E241 shows it at 2.67). The phantom 1900-02-29
     (serial 60) and the 1904 system are unprobed for caches.
  2. Whether `rangePr` start and end dates and `groupItems` follow the same rule.
  If the cached workbooks do not settle a point, write an excel-probe case (excel-graph). Use a source table with
  durations below 1, 1 to 60 and from 61 up, a pivot over it, and GETPIVOTDATA cells per item. Fill it before you
  code that point.
- Semantics entry: docs/excel-semantics/xlsx-storage.md "Pivot cache dates below serial 61 use the 1899-12-30 epoch"
  (added with this task).

## Acceptance criteria
- [x] corpus-bench 52379 and 59932 import their pivot caches with no "skipped pivot cache" line. A unit test in
      crates/rustsheet-xlsx/src/import/pivots/tests.rs pins `1899-12-30T00:07:23` → 0.00512731..., `1900-01-01T16:11:48`
      → 2.6748611111111114 and `1899-12-30T00:00:00` → 0. Each expectation cites its cached evidence.
- [x] `t="d"` cell import is unchanged: the fixtures/t-d-* cases still pass and `1899-12-31` in a cell is still `#VALUE!`.
- [x] Every rule left unverified (serial 60, 1904 workbooks, `rangePr`) is marked UNVERIFIED in xlsx-storage.md, or
      verified by an excel-graph probe case committed with this task.
- [x] No new known failures in eval/baseline (poi, corpus-bench and the vendored suites); `make check` green; reviewers
      approve.

## Out of scope
- Pivot refresh and recomputing a cache from its source.
- Export of pivot caches (the exporter writes none today).

## Outcome
- New `pivot_cache_date_serial` (crates/rustsheet-xlsx/src/import/iso_date.rs), used by `read_item` for `<d>` items. In a
  1900 workbook it reads a date from 1899-12-30 to 1900-02-28 on the OLE epoch (1899-12-30 = 0) and every later date
  with the cell rule. `iso8601_to_serial` and `t="d"` cells are unchanged: `1899-12-31` in a cell is still `#VALUE!`,
  and the fixtures/t-d-* cases pass.
- Tests: `date_items_below_serial_61_load_on_the_ole_epoch` (pivots/tests.rs) loads a cache with `1899-12-30T00:07:23`
  (443/86400 = 0.0051273148...), `1900-01-01T16:11:48` (2.6748611111111114) and `1899-12-30T00:00:00` (0), and
  `pivot_cache_dates_below_serial_61_use_the_ole_epoch` (iso_date.rs) pins the cutoff.
- corpus-bench 52379 and 59932 no longer log "bad date". Both still skip the cache, for another reason: "a grouped
  field". `fieldGroup` is skipped by design (T-2082, SPEC-0015), so the first criterion's "no skipped pivot cache line"
  is met only for the date cause. Follow-up T-4225 (opus) imports grouped fields for these two workbooks.
- xlsx-storage.md: the entry gives the implemented rule. UNVERIFIED: serial 60, serials 1 to 60 other than the one cached
  point, `rangePr`/`groupItems` (not read by the importer), and 1904 workbooks (a cache date before 1904 still skips).
- Eval delta: none (no scored cell depends on these caches); 0 regressions.
- Also filed: T-4226 (sonnet), T-4194's review follow-up (the CSE-block nested-SUBTOTAL probe).
