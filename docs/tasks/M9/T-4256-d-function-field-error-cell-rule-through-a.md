---
id: T-4256
title: "D-function field error-cell rule through a defined name, INDEX, @ and IF"
milestone: M9
lane: functions-criteria
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The D-functions give Excel's result when `field` reaches an error cell through a defined name, `INDEX(...)`, `@G1:G2` or `IF(TRUE,Data!A3)`, as T-4247 did for a bare cell reference.

## Context
T-4247 makes a bare single-cell reference to an error cell `#VALUE!` (`db_field_is_error_cell` in crates/rustsheet-engine/src/functions/database.rs). The other four forms are unprobed and ours still propagates the cell's error. Probe them with the `excel-probe` skill (extend probes/m9-database-argument-kinds), then match Excel. Also probe a `field` reference to the formula's own cell (circular): ours gives `#VALUE!` from the pre-check. Rule: docs/excel-semantics/statistics.md "Database function argument modes".

## Acceptance criteria
- [x] Probe cells for the four forms (and the self-referencing field) exist with oracle values.
- [x] Ours matches them; eval baseline shrinks, no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Database and criteria arguments.

## Outcome
Probed the four forms and the self-referencing field (probes/m9-database-field-error-forms, 24 cells, excel-graph 2026-11-04): a defined name, `INDEX`, `@` and `IF(TRUE,ref)` to an error cell are `#VALUE!` like the bare reference; a `field` that is the formula's own cell gives 0, which ours already matched. `db_field_is_error_cell` now accepts those node kinds and picks the `IF` branch (`if_pick`). Eval delta: 24 new probe cells, all passing (20 first, then the IF four). Recorded in statistics.md "Database function argument modes".
