---
id: T-4953
title: "Probe PivotChart series Excel writes: error and non-integer points, collapsed with several series, chart types, format codes"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4187]
---

## Goal
Excel-written chart caches, in a committed file, evidence the readings of `rustsheet_xlsx::export::rewrite_pivot_chart`
(T-4187) that probes/t4187-pivot-chart-caches does not show and T-4938 does not cover.

## Context
docs/excel-semantics/pivots.md "Excel for the web rewrites a PivotChart's caches when it renders the pivot" and "A
PivotChart's value cache format code differs by Excel version" list these as UNVERIFIED (T-4953). T-4187's way of
recording the evidence applies: build the probe with tools/src/rustsheet_tools/pivot_chart.py (stale caches), fill it
on excel-graph, and commit the download byte for byte under eval/roundtrip/excel-saved/cases/ beside its probe case;
then check the rewrite against it as export/pivot_charts/tests.rs does. Several labels, multi-level categories, date
labels and pivots with no row or no data field are T-4938's.

## Acceptance criteria
- [ ] Probe pivots, each with a chart bound to it, and Excel's saved file committed:
  - a data field whose cell is an error (`#DIV/0!` from an average of nothing, or an error source value): the
    error point's `c:pt`;
  - non-integer and large values (0.1+0.2, 1/3, 1E+20, -0): their `c:v` spelling;
  - a pivot collapsed to `#SPILL!` with a column field (several series): each series' references and value cache;
  - line, pie, bar and stacked-column charts (pivot_chart.py writes only clustered columns): the series' children;
  - data fields with a custom format, built-in 14 and a percent format: `c:formatCode`;
  - Values on rows with two data fields of different formats: which `c:formatCode` the series gets;
  - a pivot whose first data row is its first row (no header row above the values), if Excel lays one out:
    `c:tx`;
  - a quote or apostrophe in an item label: its spelling in `c:v`.
- [ ] `rewrite_pivot_chart` matches them, or the differences are fixed or filed; pivots.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4938's shapes; calling the rewrite from the export (T-4950).
