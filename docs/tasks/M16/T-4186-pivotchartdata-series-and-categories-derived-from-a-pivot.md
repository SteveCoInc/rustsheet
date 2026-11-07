---
id: T-4186
title: "PivotChartData: series and categories derived from a pivot"
milestone: M16
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4149, T-4130]
---

## Goal
The engine exposes, for a PivotChart, its categories (multi-level from nested row fields), series (column tuples x data fields) and values (visible, show-as applied, no subtotals or grand totals), checked against Excel-written chart caches.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §7; POI WithChartSheet.xlsx and LibreOffice forum-mso-de-104083.xlsx hold Excel caches. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Matches the corpus chart caches; probes if the web rewrites caches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Drawing charts.

## Outcome
`rustsheet_engine::pivot::chart_data(table, cache, result, layout, num_fmts, work)` returns `PivotChartData`:
categories (row lines, labels outer first) and series (column lines, each with one `Option<Result<f64, Error>>` per
category, from `PivotResult::value`, `None` where no record reaches). A line is plotted when it is a `t="data"` line
holding values with no descendant line after it. Number and date labels are formatted in the engine from their pivot
field's `numFmtId` and the workbook's `num_fmts` (`get_num_fmt`, `format_number`, en-US), their text charged to the
label budget. `ChartSeries::name` (`Total` with no label) and `PivotChartData::category_levels` (the
`c:multiLvlStrCache` levels, innermost first) read the result. It is a pure, work-charged function over an existing
result and layout (crates/rustsheet-engine/src/pivot/layout/chart.rs).

Evidence: both corpus charts are transcribed into always-run engine tests and match in full. POI
WithChartSheet.xlsx gives 6 multi-level points and `Total`. LibreOffice forum-mso-de-104083.xlsx (records reduced to
the sums its chart reads) gives 3 collapsed-year series, 132 points, no `c:pt` for empty cells and `KW 01` labels.
crates/rustsheet-xlsx/tests/pivot_chart_corpus.rs also checks the fetched files (not gated). VERIFIED is limited to
what those files show (pivots.md "A PivotChart plots the pivot's item lines"). Leaving out subtotal, `subtotalTop`,
outline-item and blank lines, show-as values and filtered items are UNVERIFIED (T-4938). The 30-probe check only shows
the plotted values are Excel's pivot cells. The web cache rewrite rests on the uncommitted download (UNVERIFIED,
T-4187, which now records chart caches). New quirk: the value cache `c:formatCode` differs between Excel 12 and 16.
