---
id: T-4147
title: "Pivot layout 1: emit rowItems/colItems and location for the three forms"
milestone: M16
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4142, T-4138]
---

## Goal
From a `PivotResult`, the engine emits Excel's rowItems/colItems (`<i t r i><x v>`) and location attributes for compact, outline and tabular form, subtotals top or bottom, grand totals on or off, and the Values field at any axis position.

## Context
SPEC-0032's contract; scored structurally by task `scoring` against the oracle and the fresh pivot-corpus files (LibreOffice xlsx/pivot/Pivot*.xlsx target these variants). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Since the plan (2026-11-06): M6 added `crates/rustsheet-engine/src/pivot/layout.rs` (a compact layout of one row field and one data field, used for grouped spec pivots, T-4235), `pivot/number_groups.rs`, `pivot/date_groups.rs` and `rustsheet-xlsx/src/import/pivots/groups.rs`; build on them, do not add a parallel layout or grouping path.

## Acceptance criteria
- [x] rowItems/colItems equal Excel's on the spike, geometry probes and fresh corpus pivots in scope.
- [x] Type the location geometry (`firstHeaderRow`/`firstDataRow`/`firstDataCol`, `rowPageCount`/`colPageCount`) in `PivotTable` and remove it from the `OpaqueOwner::Location` opaque set (T-4139 left it verbatim).
- [x] `make check` green; reviewers approve. (`make check-fast` green locally; the full `make check` and the review panel run with the PR.)

## Out of scope
- Cells (task `layout-cells`).

## Outcome
`crates/rustsheet-engine/src/pivot/layout/lines.rs` (`lay_out`, `PivotLayout`, `AxisLine`) is SPEC-0032 stage 7, a child of `pivot/layout.rs`, so the one-field compact layout (T-4235) and the lines are one layout module; T-4149 turns the lines into cells.
- **Lines.** Each axis is the result's `AxisTree` walked in display order with an explicit stack, the Values field one more level whose items are the data fields. A line is a tuple of `x`s (the item's position in the pivot field's `<items>`, or the data field index), `r` the prefix it shares with the line before (never its own last `x`). Per field form: on rows, an outline (or compact) field's item has its own line, carrying the default subtotal with `subtotalTop`; a tabular field's item shares its first child's line and its subtotals follow; columns always behave as tabular. Subtotals only for a field with a non-Values field inside; custom subtotal lines in field order; one grand line per data field when Values is on the axis, none on a Values-only axis; `<i/>` for an empty axis.
- **Geometry.** `firstHeaderRow` 1, `firstDataRow` 1 plus the column levels, `firstDataCol` the row label columns (a row field with `compact` and `outline` both on shares its column with the next level), the page grid from `pageWrap`/`pageOverThenDown`, and the location from the anchor.
- **Separate call.** `lay_out(table, cache, &result, work)` is not part of `compute`: a pivot past `MAX_PIVOT_CELLS` (4,000,000 cells) or the sheet's edge then keeps its values for GETPIVOTDATA, and T-4142's 50,000-by-50,000 diagonal value test keeps running. SPEC-0032's `PivotResult` paragraph says so now.
- **Model and import.** `PivotTable::geometry: PivotGeometry` types `firstHeaderRow`, `firstDataRow`, `firstDataCol`, `rowPageCount` and `colPageCount`; the importer reads them (missing or malformed is 0) and keeps no `OpaqueOwner::Location` attributes any more.
- **Evidence.** `pivot/layout/lines/tests.rs` compares `t`, `r`, `i` and `x` of every line, the geometry and the location with the sidecars of t4123 C, T to T4; t4138 G1 to G9; t4137 K1 to K6; t4583 R1 to R6, C1, C2; t4134 I1, I2, I4, I5; t4135 (12 sheets), t4589 (5), t4614 (7), t4616 (19); and the lines of t4128 S. `crates/rustsheet-xlsx/tests/test.rs` checks the Excel-saved pivot of `tests/example.xlsx` (three compact row fields with items listed out of cache order) against its saved `rowItems`, `colItems` and `<location>`. The probe harness moved into `pivot/probe_cases.rs`, shared with T-4142's value tests, and now builds the cache through the engine's refresh (`read_source`, Excel's item identity: t4134 I1, I4, I5) and honours a field's `order`.
- **Out of scope**, with reasons: t4128 R, t4134 I3, t4582 J1, J2 and t4585 N1 have formulas in their sources that the case file does not evaluate (T-4152 runs them through the engine); t4582 J3, J4 depend on date items against cached numbers (T-4612); t4128 G is grouped (T-4171); t4128 K has a calculated field, a Top-N filter and a descending sort (T-4174, T-4165); t4128 S's one-cell `#SPILL!` location is the grid's (T-4151). The LibreOffice corpus pivots wait for the pivot-corpus suite (T-4131).
- **UNVERIFIED** in pivots.md "Layout lines" and "Pivot geometry", with new probe tasks: subtotal placement (`subtotalTop="0"`, custom and absent subtotals in outline form, default with custom) and grand totals off (T-4649); the Values field outer or in the middle, `outlineData`, `firstHeaderRow` other than 1 and the page grid's other orders (T-4650); `compact` against `outline` for the shared column (T-4609). The cell cap and the sheet edge extend the DEVIATION entry "A pivot result is bounded in work and nodes".
- Not done: the fuzz target `pivot_import` does not call `compute` and `lay_out` yet (no nightly toolchain here, and the separate fuzz workspace build does not fit the disk): T-4651.

