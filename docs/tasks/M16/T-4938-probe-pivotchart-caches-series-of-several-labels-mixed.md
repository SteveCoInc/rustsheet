---
id: T-4938
title: "Probe PivotChart caches: series of several labels, mixed-depth categories, date and formatted labels"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4186]
---

## Goal
Excel-written chart caches, in a committed file, evidence every reading `pivot::chart_data` (T-4186) still takes on
its own: which layout lines a pivot chart plots beyond leaf and grand-total lines, show-as and filtered values, series
names of several labels, mixed-depth categories, date and formatted labels, and the edge pivots.

## Context
docs/excel-semantics/pivots.md "A PivotChart plots the pivot's item lines" lists these as UNVERIFIED (T-4938). The two
corpus files with a PivotChart (POI WithChartSheet.xlsx, LibreOffice forum-mso-de-104083.xlsx) have no subtotal line,
one-label series and categories of equal depth only. Chart caches are only in the oracle's downloaded file, not the
sidecar (probes/t4130-pivot-chart-cache), so either the probe tooling records them in the sidecar (T-4187 needs the
same) or the evidence is a committed file. Charts bound to pivots are written by
tools/src/rustsheet_tools/pivot_chart.py.

## Acceptance criteria
- [ ] Probe pivots, each with a chart bound to it and its chart caches committed:
  - two row fields in outline form with subtotals at the top (`subtotalTop`) and another at the bottom, in compact
    and tabular form, with `insertBlankRow` on the outer field (which lines are plotted);
  - a show-as data field (`percentOfTotal`, `runTotal`) beside a plain one (values as shown);
  - a hidden item, a page-field selection and a label filter (filtered values left out);
  - column fields with Values on columns, so each series has several labels (its name);
  - a collapsed outer row item beside expanded ones (mixed-depth `c:multiLvlStrCache` levels);
  - a collapsed row item with Values inside it, directly and past a hidden field (t4699 CV, CV3 shapes: the
    engine labels the hidden level empty, pivots.md "Blank rows and collapsed items beyond an outer row field");
  - a date row field, and a number row field whose pivot field and cache field `numFmtId` differ (labels);
  - a pivot with no row field, and one with no data field.
- [ ] `chart_data` matches them, or the differences are fixed or filed; pivots.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rewriting the chart part on export (T-4187).
