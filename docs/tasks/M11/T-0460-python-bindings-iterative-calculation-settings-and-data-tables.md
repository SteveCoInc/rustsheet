---
id: T-0460
title: "Python bindings: iterative calculation settings and data tables"
milestone: M11
lane: api
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`import rustsheet` can turn iterative calculation on and off with its count and delta, and can add
a what-if data table, mirroring the Rust facade.

## Context
T-0447 added `Workbook::set_iteration` / `Iteration` and T-0441 added `Txn::add_data_table` to
crates/rustsheet. The architecture review asked for binding mirrors; bindings/python has neither.

## Acceptance criteria
- [ ] Binding methods for both, with typed errors, `.pyi` stubs and docs.
- [ ] Python tests: an iterating cycle converges, and a column data table fills its results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing data tables from Python.

## Outcome
`Workbook.iteration` / `Workbook.set_iteration(enabled, max_iterations, max_change)` and
`Transaction.add_data_table(range, row_input, column_input)` in `bindings/python/src/facade.rs`,
with `.pyi` stubs and docstrings. Invalid iteration settings raise `ValueError`; data-table
errors raise `EditError` when the `with` block exits and roll the block back. Four Python tests
(settings round trip, converging cycle, column data table, rollback). Eval delta: none (bindings only).
