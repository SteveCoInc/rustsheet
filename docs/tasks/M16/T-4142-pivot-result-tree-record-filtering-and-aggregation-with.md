---
id: T-4142
title: "Pivot result tree: record filtering and aggregation with subtotals and grand totals"
milestone: M16
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4139, T-4135]
---

## Goal
`PivotResult` computes, from definition and cache, the filtered records (page field selections, hidden items) and the accumulators for every (row tuple, column tuple, data field), its subtotals and grand totals, with Excel's rules for the eleven aggregates.

## Context
SPEC-0032; evidence from `p-aggregates`; aggregate kernels reused from GETPIVOTDATA's folds and excel_variance. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Since the plan (2026-11-06): M6 added `crates/rustsheet-engine/src/pivot/layout.rs` (a compact layout of one row field and one data field, used for grouped spec pivots, T-4235), `pivot/number_groups.rs`, `pivot/date_groups.rs` and `rustsheet-xlsx/src/import/pivots/groups.rs`; build on them, do not add a parallel layout or grouping path.

## Acceptance criteria
- [x] Decide, with a probe if needed, what a malformed `pageField@item` means. T-4139 reads it leniently as absent, which is `(All)`.
- [x] Unit tests; GETPIVOTDATA answers unchanged.
- [x] Bench on 100k records (crates/rustsheet/benches).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Layout.

## Outcome
`crates/rustsheet-engine/src/pivot/result.rs` (`compute`, `PivotResult`, `AxisTree`) and `pivot/accumulate.rs` (`Accumulator`) implement SPEC-0032 stages 1 to 3:
- **Stage 1.** Page selections and hidden items of axis fields filter the records (t4137 K4). A malformed `pageField@item` (past the field's `<items>`, naming a non-data line, or unparsable, which the import already reads as absent) stays `(All)`: no evidence exists, so the reading is UNVERIFIED in pivots.md "Record filters: page selections and hidden items" and probe T-4633 is filed (with hidden items off every axis, which filter nothing, and cells with no number).
- **Stage 3.** One row-major pass over the records sorted by (row leaf, column leaf, record) feeds every covering accumulator, so cells accumulate in record order and totals item by item in display order. A Sum-only pivot's totals sum their child cells (the grand-grand total its row totals); Var is the textbook one-pass formula and Varp (n·Σx² − (Σx)²)/n²; Count totals covering an error are the error only beside a non-count aggregate, subtotals included (all per probes/t4616-pivot-total-order-3, T-4616; open points UNVERIFIED with T-4634).
- **Reuse.** The display order and the folds are shared with GETPIVOTDATA's semantics, but the result does not reuse `layout::aggregate`/`excel_variance` for the variances: t4616 refutes the worksheet functions' two-pass switch for pivot totals. GETPIVOTDATA (`pivot_index.rs`, `layout.rs`) is untouched, so its answers are unchanged; T-4154 moves it onto the result.
- **Limits.** `MAX_PIVOT_WORK` (2^28 units) and `MAX_PIVOT_NODES` (10,000,000), a DEVIATION entry in pivots.md. `compute` takes `work: &mut u64` rather than an `AreaBudget`, so it stays a public pure function; the caller wiring it into `Evaluation::pivot_results` (T-4149, T-4154) passes the run budget's units.
- **Tests.** `pivot/result/tests.rs` builds each probe's cache and definition from its case file and checks every value cell of the sidecar through its own `rowItems`/`colItems`: t4135 (627 cells), t4589, t4614 and t4137 K1 to K6. The t4616 sheets pass too against that branch's sidecar (checked locally, not committed, since the files land with T-4616); add them to the test once it merges. serde_json's `float_roundtrip` is on for the engine's tests, as for rustsheet-eval, so sidecar doubles parse exactly.
- **Bench.** `crates/rustsheet/benches/pivot.rs` times `compute` over the 100,000-record source (Region by Product, Sum and Average), plain and with filters: about 10 ms each (min 10.2 ms and 9.1 ms, bench profile, this container).
- Not done: extending the fuzz target `pivot_import` to `compute` (SPEC-0032 lists it under T-4142 and T-4147); no nightly toolchain here, left to T-4147's fuzz extension.

