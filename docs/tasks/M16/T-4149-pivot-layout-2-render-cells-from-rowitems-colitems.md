---
id: T-4149
title: "Pivot layout 2: render cells from rowItems/colItems"
milestone: M16
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4147, T-4137]
---

## Goal
The engine writes the pivot's cells from its rowItems/colItems: captions, item labels (compact indentation is a style, the text is the item), values, page field area, `(blank)`, empty-cell and error captions.

## Context
Evidence from `p-captions` and `p-geometry`. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Since the plan (2026-11-06): M6 added `crates/rustsheet-engine/src/pivot/layout.rs` (a compact layout of one row field and one data field, used for grouped spec pivots, T-4235), `pivot/number_groups.rs`, `pivot/date_groups.rs` and `rustsheet-xlsx/src/import/pivots/groups.rs`; build on them, do not add a parallel layout or grouping path.

The pivot probes' visibly-off cells (decision 0245) are owned by T-4152, which renders spec-case pivots through the engine so they pass (SPEC-0032 "Rollout" step 7); this task's render is checked against their sidecars in unit tests.

## Acceptance criteria
- [x] Pivot-area cells of the PV1 probes match the oracle.
- [x] Retire T-4235's one-field `row_layout`/`layout_extent` (pivot/layout.rs, used by `lay_out_grouped` in crates/rustsheet/src/edit/pivot.rs) onto `lay_out`'s lines, so one layout path remains (T-4147 design review).
- [x] `make check` green; reviewers approve. (`make check-fast` green locally; the coordinator runs review and CI.)

## Out of scope
- Number formats of data fields (PV2).

## Outcome
- `pivot::render(table, cache, result, layout) -> Vec<RenderedCell>` (crates/rustsheet-engine/src/pivot/layout/render.rs) writes a pivot's cells from its `PivotLayout` lines: the column and row headers (`Column Labels`/`colHeaderCaption`, `Values`/`dataCaption`, `Row Labels`/`rowHeaderCaption`, field names under the all-`compact="0"` rule and over each further label column), each line's labels from `r` on, `<item> Total`, `<item> <Function>`, `Grand Total`/`grandTotalCaption` and `Total <data field>`, a lone data field's caption (corner, over the data column, or as the only row's label), the values with `missingCaption` and `errorCaption`, and the page area (`(All)`, the item, `(Multiple Items)`) in the `rowPageCount` rows above the location. Items render as pivots.md says (numbers as numbers, booleans and errors as text, `(blank)`).
- Tests (pivot/layout/render/tests.rs) compare every sidecar cell, empty ones included, and fail on any rendered cell outside the capture window, for t4123 C, T to T4, t4138 G1 to G9, t4137 K1 to K6, t4583 R1 to R6, C1, C2, t4622 (all seven), t4134 I1, I2, I4, I5, t4135 (twelve), t4589, t4614 and t4616 (every sheet). Left out, with the owner: t4123 T to T4 column D values (show values as, T-4163); t4622 East's custom Count subtotal values over an error under Sum (CA, CN C6; B1, B4, B5 C6:C7: the result gives the error, Excel a count; T-4168, T-4636); t4128 S (`#SPILL!`, T-4151), R (formula source, T-4152), G (grouping, T-4171), K (calculated field, Top-N, sort; T-4174, T-4165); t4134 I3, t4582, t4585, t4627 (formula sources, T-4152; t4582 J3, J4 also T-4612).
- One layout path: T-4235's `row_layout`, `layout_extent`, `max_layout_rows`, `LayoutCell` and `LAYOUT_COLUMNS` are gone. `Txn::add_pivot_table`'s `lay_out_grouped` runs `compute`, `lay_out` and `render` and takes the layout's location (new `PivotError::Compute`); `compute` now honours `showAll` on an axis of one field (t4311 H), the grouped pivots' only shape, and the eval's spill-extent check reads the engine's own pivot locations (`pivot_places`) instead of the `max_layout_rows` bound. The probe harness (`pivot/probe_cases.rs`) now builds the builder's `showAll="0"`, default data-field names (`Sum of X`) and caption attributes. `lines::label_columns` is shared by the geometry and the renderer, and `result::page_selection` by the record filter and the page area.
- Eval effect: none in this task. SPEC-0032 puts writing the cells into the grid on T-4151 ("Rendering and the grid", rollout step 6) and the probes' pivot cells passing on T-4152 (rollout step 7: "the adapter renders through the engine, so the probes' pivot cells start passing"; "Until T-4152 lands, spec-case pivots with a `definition` stay unregistered and their cells known failures"), so the decision-0245 owner line moved to T-4152. Grouped spec pivots render as before.
- UNVERIFIED readings (pivots.md): number and date item subtotal labels, page fields with one visible item or a number/boolean/blank selection, `showAll` on axes of several fields or page-filtered items (new probe task T-4657); Values-inside subtotal labels, the grand caption with Values on its axis, the header over a Values label column (T-4650); `rowHeaderCaption` under the all-`compact="0"` rule (T-4609).
