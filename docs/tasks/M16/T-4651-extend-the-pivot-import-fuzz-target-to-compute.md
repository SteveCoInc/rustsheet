---
id: T-4651
title: "Extend the pivot_import fuzz target to compute and lay out every imported pivot"
milestone: M16
lane: engine
status: todo
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
- [ ] The target calls `compute` and, on success, `lay_out` for each pivot; a short local run (nightly, `cargo fuzz run pivot_import -- -max_total_time=60`) finds no panic.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rendering cells (T-4149 extends the target again).
