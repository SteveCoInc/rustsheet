---
id: T-4760
title: "A relative reference in a defined name does not move under row and column inserts and deletes (T-4747 probes)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`Model::displace_defined_names` leaves a relative reference in a defined name where Excel leaves it, so the probes below read what Excel reads.

## Context
T-4747 probed a workbook name `Rel` = `=Data!A5` (relative) used from Calc!A1 and Calc!A3. Excel (excel-graph) does not shift
it under `insert_row 4` or `delete_row 4` on Data (A1 40/60, A3 60/80 = the literal addresses A5, A7 after the edit), while
absolute names do shift (T-4702). The engine shifts it (50/70 on both). Failing cells:
probes/sheet-ops/t4747-name-relative-insert-row.case.json Calc!A1 and Calc!A3; probes/sheet-ops/t4747-name-relative-delete-row.case.json
Calc!A1 and Calc!A3. Code: `displace_defined_names` in crates/rustsheet-engine/src/formula_rewrite.rs. Rule: docs/excel-semantics/references.md.

## Acceptance criteria
- [x] First probe a relative name used from the edited sheet itself (see references.md); the rule so far covers another sheet only.
- [x] The four cells above pass; absolute names (t4702-*) still shift.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed first (excel-graph, 2026-11-08, six cases probes/sheet-ops/t4760-name-*, 14 cells, predictions all right): a no-edit
control (the file text is anchored at A1), the name used from the edited sheet itself (row insert and delete), column
insert and delete, and mixed references under a row insert. Rule: a row edit leaves a name's relative rows alone and a
column edit its relative columns; only absolute parts move (references.md "Relative references in a name").
`Model::displace_defined_names` now sets `Displacement::relative_stays`, which the printer reads per reference and range
corner (`Displacement::keeps`, `of_corner`). Eval: the four t4747 cells pass, the 14 new t4760 cells pass, t4702 still
passes. Unit tests in `test_defined_name_relative.rs`.
