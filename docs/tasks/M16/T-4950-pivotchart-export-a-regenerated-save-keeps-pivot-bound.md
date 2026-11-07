---
id: T-4950
title: "PivotChart export: a regenerated save keeps pivot-bound drawings and charts, series rewritten by rewrite_pivot_chart"
milestone: M16
lane: engine
status: done
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
- [x] roundtrip-excel-saved's R0 keeps the drawing and chart parts and relationships (baseline lines removed by
  `make eval-baseline`), and each saved chart's series equal Excel's saved ones.
- [x] A drawing whose closure holds a picture or a non-pivot chart is dropped with a log line, the package still valid.
- [x] docs/excel-semantics/pivots.md "Excel for the web rewrites a PivotChart's caches when it renders the pivot" names
  the export as implementing it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Non-pivot charts, pictures and shapes (M15's replay layer).
- What Excel writes outside the series (`c:pivotSource` name, the added `c:pivotFmts`, the axis `c:numFmt`): the
  rewrite keeps the part's (pivots.md lists them).

## Outcome
Done in one piece (size M; no split). `save_preserving`'s Regenerated tier now writes through
`export::save_regenerated`, which builds a `DrawingPlan` (crates/rustsheet-xlsx/src/export/pivot_drawings.rs) from the
source package before writing: for each source sheet (mapped by the pristine model and the importer's `part_paths`)
whose one drawing relationship leads to a drawing of chart frames only, every relationship a chart, each chart's only
relationships `chartStyle`/`chartColorStyle` and every `r:` reference resolved, it copies the drawing byte for byte as
`xl/drawings/drawingN.xml`, each chart as `xl/charts/chartN.xml` (style and colors parts as `styleN.xml`/`colorsN.xml`),
with relationships keeping the source's ids, the overrides, and the sheet's `<drawing r:id>` (after the hyperlinks,
before `tableParts`; its relationship numbered after the pivot tables'). Each chart goes through `rewrite_pivot_chart`
with the pivot its `c:pivotSource` names (new `pivot_charts::pivot_source`: `[file]Sheet!Name`, quoted sheets
unquoted), laid out again with one `MAX_PIVOT_WORK` budget, `collapsed` when the grid saved the location as
its anchor holding `#SPILL!`. Dropped with a `RustSheet: export dropped drawing of sheet …` line: a picture, shape or
other anchor content, a non-chart graphic frame, any other relationship type, an unresolved reference, nesting past
`MAX_DEPTH`, a chart not bound to a pivot or whose pivot the export does not write (`PivotPlan::exports_pivot`) or
whose cells are not the engine's layout, any rewrite error, a chart two drawings share, and the drawing of a sheet
renamed, moved, deleted or whose records other than cells changed (`patch::sheet_structure_same`, extracted from the
patcher's check; the model has no trace of decision 0098's structural edits, so this stands in for one, and T-4567 is
still blocked). Reads stay within the import's limits; kept parts are capped together at 256 MiB (`MAX_KEPT_BYTES`).
Directive 3: reuses the importer's `Package`, `part_relationships`, `part_folder_and_rels`, `resolve_relative_target`,
`part_paths`, `reference_of` and the export's `log_drop` (now crate- or module-visible) rather than new readers.

Tests: crates/rustsheet/tests/pivot_chart_save.rs (Excel's file loaded, its `refreshOnLoad` pivots rendered, saved: the
four drawings are Excel's bytes and the four chart parts equal Excel's saved ones; a stale cache is rewritten to Excel's;
style parts kept; a picture relationship, a non-pivot chart and a shape each drop their drawing; renamed, row-changed
and moved sheets drop theirs; the package checked for dangling targets, overrides and `r:id`s every time) and
export/pivot_drawings/tests.rs (the part checks and `pivot_source`).

Eval: roundtrip-excel-saved R0 49.89% → 77.54% kept (361 → 349 failing items, the 12 drawing and chart lines removed by
`make eval-baseline`, 0 regressions; the harness's package validator, V1-V6, finds nothing in the saves); R1 and
R0/again 100%. roundtrip-poi: 52348.xlsx's empty drawing (no anchor, no relationship) is now kept, 335 -> 332 failing
items. roundtrip-fixtures and every cell suite are unchanged.
Follow-ups: T-4991 (probe: the quoted sheet, case-insensitive sheet and exact pivot name readings of `c:pivotSource`,
UNVERIFIED in pivots.md) and T-4988 (keep the drawing when a sheet's row or column records change without a
structural edit, which this check is too strict for: `Txn::set_row_hidden` drops it today).
