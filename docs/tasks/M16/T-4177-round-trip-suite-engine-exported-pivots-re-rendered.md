---
id: T-4177
title: "Round-trip suite: engine-exported pivots re-rendered by Excel"
milestone: M16
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4176]
---

## Goal
A suite whose cases are exported by the engine and sent to the oracle: a refusal or repair fails; Excel's re-render and rowItems must equal the engine's.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §6 layer 4. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Suite and harness support; first cases filled.
- [ ] Cases for what T-4176's export left UNVERIFIED: an API-built pivot's generated `<items>` re-rendered
  (`refreshOnLoad` on); a pivot without `refreshOnLoad` whose saved `rowItems` are stale (the export writes them
  back as read); a pivot whose render collapsed to `#SPILL!` (the export writes it without `rowItems`/`colItems`);
  cache dates at serials 1 to 60, the phantom serial 60 (written as `1900-02-28T00:00:00`) and a
  1904 workbook's cache (docs/excel-semantics/xlsx-storage.md "Pivot cache dates below serial 61 use the
  1899-12-30 epoch"). The three T-4176 scratch runs (pivots.md "Excel for the web opens engine-exported pivot
  parts") become committed cases.
- [ ] Cases for T-4734's grouped-cache export (pivots.md "Excel for the web opens engine-exported grouped pivot
  caches"): its scratch runs become committed cases. They are the engine exports of probes/t4170-pivot-manual-groups,
  t4170-pivot-date-levels, t4236-pivot-date-groups-no-blank, t4228-pivot-range-groups and t4128-pivot-spike (G),
  plus the API-built cache: Amount by tens, Date by months with Quarters and Years, and Product2 as a manual group.
  Add one more case for a date field grouped in place whose `sharedItems` carry no `containsNonDate`,
  `minDate` or `maxDate`. It commits the "flags needed" result: Excel's refresh dropped the months.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
