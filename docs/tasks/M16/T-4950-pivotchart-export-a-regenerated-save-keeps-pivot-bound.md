---
id: T-4950
title: "PivotChart export: a regenerated save keeps pivot-bound drawings and charts, series rewritten by rewrite_pivot_chart"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4187]
---

## Goal
A workbook whose pivots the engine re-renders (a `refreshOnLoad` pivot on the first recalculation, or an API refresh) and
which therefore saves through the regenerating export keeps its PivotCharts: the sheet's drawing, its chart parts
(`c:pivotSource`, `c:pivotFmts`, the c14/c15/c16 extensions) and their relationships and content types, with each
chart's series written by `rustsheet_xlsx::export::rewrite_pivot_chart` from the pivot's `chart_data`.

## Context
Split from T-4187, which committed Excel's evidence (eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx,
probes/t4187-pivot-chart-caches) and the rewrite as a pure function that reproduces Excel's four saved chart parts byte
for byte (`export/pivot_charts.rs`). Nothing calls it yet: `save_preserving` keeps charts only in its Verbatim and
Patched tiers, and `save_xlsx_to_writer` writes no drawing, so the Regenerated tier drops every drawing and chart. The
roundtrip-excel-saved case shows it: its four pivots are `refreshOnLoad`, the first recalculation renders them, and R0
loses the drawing and chart parts and their relationships (eval/baseline/roundtrip-excel-saved.txt).

Shape (docs/research/xlsx-round-trip-2026-11-04.md ownership matrix, decision 0275: charts and drawings are `Replay`, a
drawing after a structural edit `ReplayIfClean`): in the Regenerated tier, for each source sheet whose drawing's closure
is pivot charts only (chart, chartStyle, chartColorStyle), copy the drawing and its charts into the regenerated package
under export-owned names, add the sheet's `<drawing r:id>` in schema order, its relationship and the content-type
overrides, and pass each chart through `rewrite_pivot_chart` with the pivot its `c:pivotSource` names (`[file]Sheet!Name`)
and the grid's collapsed flag (`#SPILL!`). A drawing with anything else stays dropped, with a log line, until M15's
replay layer (SPEC-0034 D5, T-4567; implemented by T-5002) owns it. Map sheets by the pristine model; drop a drawing whose sheet was renamed, moved
or structurally edited (harness-only today, decision 0098) unless SPEC-0034 D5.5 (T-5004) has settled it. Use the package validator.

## Acceptance criteria
- [ ] roundtrip-excel-saved's R0 keeps the drawing and chart parts and relationships (baseline lines removed by
  `make eval-baseline`), and each saved chart's series equal Excel's saved ones.
- [ ] A drawing whose closure holds a picture or a non-pivot chart is dropped with a log line, the package still valid.
- [ ] docs/excel-semantics/pivots.md "Excel for the web rewrites a PivotChart's caches when it renders the pivot" names
  the export as implementing it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-pivot charts, pictures and shapes (M15's replay layer).
- What Excel writes outside the series (`c:pivotSource` name, the added `c:pivotFmts`, the axis `c:numFmt`): the
  rewrite keeps the part's (pivots.md lists them).
