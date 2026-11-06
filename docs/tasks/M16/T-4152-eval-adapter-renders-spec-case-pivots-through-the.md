---
id: T-4152
title: "Eval adapter renders spec-case pivots through the engine"
milestone: M16
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4151, T-4124]
---

## Goal
Spec cases with a `definition` reach the engine as a pivot built from the source cells and rendered by the engine (no baked `rendered` cells), so the `pivots` suite scores the engine's own render.

## Context
crates/rustsheet-eval/src/spec_pivots.rs (T-2084) bakes cells today; keep that path for old cases. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

The decision-0245 owner paragraph of the pivot probes' visibly-off cells (moved here from T-4149)
is split by cause into the open tasks that own the cells still off (see Outcome).

## Acceptance criteria
- [x] pivots-suite cases scored through the engine render; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing the baked path.

## Outcome
The probes suite's defined pivots (17 cases, 121 pivots; there is no separate `pivots` suite) now
reach the engine as the parts the case builder writes and are rendered by the engine's refresh on
load. Design:
- `crates/rustsheet-eval/src/pivot_definition.rs` writes a definition's `pivotCacheDefinition`,
  `pivotCacheRecords` and `pivotTableDefinition` exactly as `pivot_definition.py` does (all 363 parts
  of the probes compared equal to the builder's, attribute order aside), so the engine loads what Excel
  opened.
- `rustsheet_xlsx::import::read_pivot_parts` reads such parts with the importer's own cache and table
  readers (`load_cache` split into `read_definition` + `finish_cache`), the one reader of a definition.
- `Workbook::load_pivot_parts` (doc-hidden, facade) adds the cache and tables in one edit and marks a
  `refreshOnLoad` cache due, so the next `Workbook::recalculate` refreshes and renders it after
  calculating: the T-4151 load lifecycle. The adapter (`spec_pivots::load_defined`) calls it after the
  case's cells are in, then recalculates, before typed entries (Excel's open order). A pivot the importer
  drops (calculated field, `showDataAs`) is dropped, its cells left empty. The baked `rendered` path is
  unchanged.
- The grid writer leaves an empty-text item's label cell empty (probes/t4627-pivot-field-name-collisions
  P_Rec!A6, pivots.md), which the engine render would otherwise have regressed.

Eval (`make eval-baseline`, REGRESSIONS 0): probes known failures 8616 -> 6052 (fixed 2564); every other
suite unchanged (gen-m5 3, gen-m9 1767, ironcalc-calc 266, the rest 0). Twelve of the 17 cases now pass
whole (t4134, t4135, t4137, t4138, t4582, t4583, t4586, t4589, t4609, t4612, t4614, t4616). The 292
cells still off, by cause and owner:
- `showDataAs` data fields, skipped by the importer: t4123 T to T4 (160) -> T-4163;
- a calculated field, skipped by the importer: t4128 K (28) -> T-4174 (then T-4165);
- the builder's numeric group labels refused by the importer: t4128 G (10) -> T-4171;
- custom countA/count subtotals over an error: t4622 B1, B4, B5, CA, CN (8) -> T-4168 (scope T-4636);
- refresh renames fields and `remap` refuses (`FieldsMoved`) because the pivot keeps verbatim parts:
  t4585 N1 and t4627 P_SufA, P_SufB, P_FldA, P_FldB, P_Off, P_Empty, P_Const, P_Space, P_Fold (86)
  -> T-4694 (filed here).
