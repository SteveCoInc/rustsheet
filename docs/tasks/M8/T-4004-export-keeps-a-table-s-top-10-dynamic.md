---
id: T-4004
title: "Export keeps a table's top 10, dynamic, date-group, color and icon filter criteria so Excel still calls the rows filtered"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An exported table keeps its top 10, dynamic, date-group, color and icon filter criteria, so Excel still calls its hidden
rows filtered after a round trip.

## Context
- Evidence: docs/excel-semantics/xlsx-storage.md "Table parts in the export" (T-3081; throwaway excel-graph uploads 2026-11-04, recipe there). A table with a top 10 filter (top 2 items of `Val`), imported and exported, is written as
  `<filterColumn colId="1"/>`. Excel for the web then counts its hidden rows as manually hidden:
  `SUBTOTAL(9,Sales[Val])` is 21 where the original filter made it 11 (SUBTOTAL 109 is 11 in both), and
  `SUBTOTAL(3,Sales[Cat])` is 6 where it was 2. So the round trip changes Excel's values.
- Code: `filter_column` in crates/rustsheet-xlsx/src/import/tables.rs and crates/rustsheet-xlsx/src/export/tables.rs
  (`FilterColumn`, crates/rustsheet-engine/src/table_types.rs). Keeping the criterion as validated, bounded data
  (directive 5) is enough; nothing needs to re-apply it.

## Acceptance criteria
- [x] Top 10, dynamic, color, icon and date-group criteria survive import, export and import (unit tests on the
      written XML), and the exported workbook gives Excel's SUBTOTAL values (an excel-graph upload or a probes case).
- [x] xlsx-storage.md "Table parts in the export" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter. The criterion-less and out-of-range forms (T-4003).

## Outcome
- `FilterCriterion` (crates/rustsheet-engine/src/table_types.rs) gains `Top10`, `Dynamic`, `Color` and `Icon`
  variants, and `Values` keeps its `calendar_type` and `date_groups` (`DateGroupItem`). The import reads every
  criterion in the new crates/rustsheet-xlsx/src/import/filter_criteria.rs (values and custom filters moved there
  from tables.rs), checked against the schema (enumerations for `ST_DynamicFilterType`, `ST_IconSetType`,
  `ST_CalendarType`, `ST_DateTimeGrouping`; finite doubles; ISO dates through `iso8601_to_serial`; `unsignedShort`
  date fields); a criterion that fails its check stays `Other` (criterion-less on export, as before). The export
  (`filter_column`, export/tables.rs) writes each kind back, dropping schema defaults and a color filter's `dxfId`
  past the workbook's dxfs (`dxf_attribute`).
- Evidence: excel-graph uploads 2026-11-05 of an openpyxl file with a top 10, dynamic, color, icon and date-group
  filter (plus a criterion-less control) and of our import-then-export of it: all 24 SUBTOTAL cells agree between the
  two, and each filtered kind gives SUBTOTAL 9 = 109 (filtered), where before this change the top 10 round trip gave
  21 for 11. Recorded with the recipe in docs/excel-semantics/xlsx-storage.md "Table parts in the export".
- Tests: `top10_dynamic_color_icon_and_date_criteria_survive_an_export` (written XML and re-import),
  `a_color_filter_past_the_dxfs_drops_its_dxf_id`, three `filter_criteria` unit tests (defaults, refusals), and
  `date_grouped_values_filters_keep_their_groups` (replacing the test that pinned them as `Other`).
- Eval delta: none (no eval case round-trips an export; the evidence is the upload above).
