---
id: T-4247
title: "D-function field given as a reference to an error cell is #VALUE!, not the error"
milestone: M9
lane: functions-criteria
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
DAVERAGE, DCOUNT, DCOUNTA, DGET, DMAX, DMIN, DPRODUCT, DSTDEV, DSTDEVP, DSUM, DVAR and DVARP give `#VALUE!`
when `field` is a single-cell reference to a cell holding an error, as Excel does, while an error literal
`field` still returns that error.

## Context
T-3214's probe probes/m9-database-argument-kinds (excel-graph, 2026-11-04) has `field` as `#N/A`, `#DIV/0!`
and `Data!A3` (a cell holding `=NA()`). The literals propagate (Calc!A1:A2, A4:A5, ..., A34:A35 match).
The reference is `#VALUE!` in Excel and `#N/A` in ours in Calc!A3, A6, A9, A12, A15, A18, A21, A24, A27,
A30, A33, A36 (12 known failures). docs/excel-semantics/statistics.md "Database function argument modes"
has the rule.

Code: `resolve_db_field_column` in crates/rustsheet-engine/src/functions/database.rs returns any error
`field` unchanged; it receives the field already dereferenced by the SPEC-0011 lift driver (`db_call`), so
it cannot tell a literal from a referenced error. The fix likely looks at the `field` argument's node in
`db_call` (a `ReferenceKind`) or marks the dereferenced value. Unprobed and worth a cell each before
choosing: a field reference through a defined name, `INDEX(...)`, `@G1:G2` and an `IF(TRUE,Data!A3)`.

## Acceptance criteria
- [ ] The 12 cells above pass; probes for the unprobed forms above are added and match.
- [ ] `make eval-baseline` shrinks the known failures; no regressions.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells are a wrong error (`#N/A` where Excel is `#VALUE!`), so this task is in the
campaign. Its target is that every cell it owns has Excel's type and error.

## Out of scope
- Other `field` kinds; database and criteria arguments.
