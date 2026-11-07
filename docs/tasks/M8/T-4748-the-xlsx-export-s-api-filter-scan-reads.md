---
id: T-4748
title: "The xlsx export's API-filter scan reads hidden rows in one pass, not one row-record scan per data row (T-4363 review follow-up)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Exporting an API-filtered table costs time linear in the table's rows plus the sheet's row records, not their product.

## Context
- Found by T-4363's security review (PR #3795, MINOR). `api_filter_column` (crates/rustsheet-xlsx/src/export/tables.rs)
  calls `Model::is_row_hidden` once per data row of the table, and `Worksheet::is_row_hidden`
  (crates/rustsheet-engine/src/worksheet.rs) scans `Worksheet::rows` linearly: O(table rows x row records), up to
  ~1e12 steps for a 1M-row table on a sheet with 1M row records. Only a table filtered through the API reaches it
  (import sets `has_filters` only with a kept criterion).
- Fix: collect the hidden rows inside the table's data rows once from `worksheet.rows` (a sorted list or a set), then
  walk the data rows. Search for an existing hidden-rows helper first (`rg "hidden" crates/rustsheet-engine/src`).

## Acceptance criteria
- [ ] `api_filter_column` reads hidden rows in one pass over the row records; the XML is unchanged
      (`a_filtered_table_without_a_criterion_exports_its_visible_values` still passes).
- [ ] A unit test exports a filtered table of 100,000 rows with 50,000 hidden row records quickly (well under a second
      in release).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other linear `is_row_hidden` callers.

## Outcome
`api_filter_column` now collects the hidden rows of the table's data range in one pass over `Worksheet::rows` (first record of a
row decides, as in `is_row_hidden`) and walks the rows against that set. XML unchanged (existing table tests pass). New test
`the_api_filter_scan_of_a_big_table_is_not_quadratic`: 100,000 rows, 50,000 hidden records, ~0.1 s in debug against ~2.6 s for the
old scan. Eval: no change. No follow-ups.
