---
id: T-4256
title: "D-function field error-cell rule through a defined name, INDEX, @ and IF"
milestone: M9
lane: functions-criteria
status: in-progress
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
- [ ] Probe cells for the four forms (and the self-referencing field) exist with oracle values.
- [ ] Ours matches them; eval baseline shrinks, no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Database and criteria arguments.
