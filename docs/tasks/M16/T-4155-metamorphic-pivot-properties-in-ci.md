---
id: T-4155
title: "Metamorphic pivot properties in CI"
milestone: M16
lane: quality
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4151]
---

## Goal
Property tests over generated pivots: refresh is idempotent; GETPIVOTDATA of each value cell's tuple returns the cell; import-export-import is a fixed point (once export exists); permuting source rows changes only order-dependent outputs.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §6 layer 5. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Proptest-style tests committed and run in CI time budget.
- [x] `make check` green; reviewers approve.

## Out of scope
- Oracle-backed checks.

## Outcome
- `crates/rustsheet-engine/src/test/test_pivot_properties.rs`: seeded property tests (the test-support xorshift `Rng`, 48
  seeds each, no new dependency) over generated sources (1-30 records, Region and Product texts with about one in
  eight in lower case, integer Sales so sums are exact) and a Sum of Sales pivot (Region rows, Product columns, grand
  totals) rendered by `plan_pivot_refresh` / `apply_pivot_refresh`:
  - a second refresh changes no cell, nor the pivot table, nor the cache;
  - each value cell, grand totals included, is the brute-force sum of its case-folded (Region, Product) records;
  - `GETPIVOTDATA("Sales",$A$3,...)` of each non-empty value cell's tuple returns the cell;
  - permuting the source rows keeps every value and the same case-folded items (order and first-seen casing may
    change).
  All four pass on main as it is; together they run in under 0.1 s.
- The fourth property of the plan, import, export and import as a fixed point, needs the xlsx crate (T-4176's
  export): follow-up T-4756.
- Eval delta: none (tests only).
