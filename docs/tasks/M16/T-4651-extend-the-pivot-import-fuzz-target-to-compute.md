---
id: T-4651
title: "Extend the pivot_import fuzz target to compute and lay out every imported pivot"
milestone: M16
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The `pivot_import` fuzz target runs every imported pivot through `pivot::compute` and `pivot::lay_out` under a small work budget, so untrusted definitions and caches reach the result and layout stages, not only GETPIVOTDATA.

## Context
SPEC-0032 "No panics" makes the fuzz target extended to compute and render an acceptance criterion of T-4142 and T-4147; neither could build the separate nightly fuzz workspace (T-4142's and T-4147's Outcome notes). `fuzz/fuzz_targets/pivot_import.rs` loads through the facade; the engine's `Model` (via `rustsheet_xlsx::import::load_from_xlsx_bytes`) exposes `workbook.pivot_tables` and `pivot_caches`. Charge a budget well below `MAX_PIVOT_WORK` so one input stays fast.

## Acceptance criteria
- [x] The target calls `compute` and, on success, `lay_out` for each pivot; a short local run (nightly, `cargo fuzz run pivot_import -- -max_total_time=60`) finds no panic.
- [x] `make check` green; reviewers approve.

## Out of scope
- Rendering cells (T-4149 extends the target again). Done here after all, see the Outcome.

## Outcome
`fuzz/fuzz_targets/pivot_import.rs` now imports each rebuilt package through the engine importer as well
(`rustsheet_xlsx::import::load_from_xlsx_bytes`) and runs every imported pivot through `pivot::compute` and, on
success, `pivot::lay_out` and `pivot::render`, under a work budget of `MAX_PIVOT_WORK >> 12` units per pivot. Render
is included because T-4149 (done) did not extend the target, so this closes that gap too. Local runs on nightly
found no panic: `cargo fuzz run pivot_import -- -max_total_time=60` (816 runs, an empty corpus), and 90 s seeded
with the template's own three pivot parts (1,324 runs, coverage 17,133 → 18,181 features). The engine is unchanged.
