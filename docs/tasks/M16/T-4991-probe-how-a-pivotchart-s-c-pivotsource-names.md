---
id: T-4991
title: "Probe: how a PivotChart's c:pivotSource names a pivot on a sheet whose name needs quotes, and whether sheet and pivot names match ignoring case"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Repository evidence of three readings T-4950 makes of a PivotChart's `c:pivotSource` `c:name` (`[file]Sheet!Name`),
all UNVERIFIED in docs/excel-semantics/pivots.md "Excel for the web rewrites a PivotChart's caches when it renders the
pivot": how Excel writes the sheet when its name needs quotes (a space, a quote, an R1C1-shaped name such as `C`, which
t4187 writes bare), whether a name whose sheet differs in case from the sheet's binds the chart, and whether the pivot
name matches exactly or ignoring case.

## Context
crates/rustsheet-xlsx/src/export/pivot_charts.rs (`pivot_source`, `split_pivot_source`) reads a quoted sheet with its
doubled quotes undone; export/pivot_drawings.rs (`rewrite`) finds the sheet ignoring case and the pivot by its exact
name, and drops the drawing otherwise. The evidence (eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx)
has only bare names in matching case. A probe in the shape of probes/t4187-pivot-chart-caches
(tools/src/rustsheet_tools/pivot_chart.py) on sheets `My Sheet`, `It's` and one in different case settles it.

## Acceptance criteria
- [ ] A committed Excel-saved file shows the `c:name` Excel writes for a pivot on a sheet that needs quotes, and what
  Excel does with a `c:name` whose sheet or pivot differs in case; pivots.md's three UNVERIFIED readings are settled and
  the code follows the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other UNVERIFIED pieces of the rewrite (T-4938, T-4953).
