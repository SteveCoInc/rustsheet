---
id: T-4596
title: "TAKE/DROP/TRIMRANGE over a picker: probe a picked defined name, INDEX and CHOOSE one-cell pick, and name TRIMRANGE in arrays.md"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The arrays.md entry "TAKE and DROP keep the one-cell reference a picker hands them" (T-4469) claims only what Excel
cells evidence. TRIMRANGE is named in its rule, and a picked defined name, a picked INDEX and a CHOOSE one-cell pick are
each probed or dropped from the rule.

## Context
- Review MINORs on #3610 (T-4469, reviewer-excel-fidelity):
  - probes/t4469-take-drop-if-one-cell-reference A11:D13 show TRIMRANGE over IF, IFS and SWITCH keeps the reference.
    The code cites them, but the arrays.md entry names TAKE and DROP only and cites A1:D10.
  - The entry's rule also covers a pick that is "a defined name or an INDEX", and says CHOOSE "already kept it". No
    t4469 cell probes those; the CHOOSE claim describes the engine, not Excel.
- Code: `is_reference_form` and `take_drop_value_source` in crates/rustsheet-engine/src/functions/lookup_and_reference/drop_take.rs,
  and the `keep_cell_variables` arm of `evaluate_picked_source`.
- Probe (excel-probe skill, excel-graph, tag M7), through TYPE, ISREF, `&"x"` and `ISBLANK(INDEX(X,1,1))` as t4469 does:
  `TAKE(IF(TRUE,nm),1)` with `nm` a one-cell name (blank and a number), `TAKE(IF(TRUE,INDEX(Y1:Y2,1)),1)`,
  `TAKE(CHOOSE(1,Z1),1)`, `DROP(CHOOSE(1,Y1),0)`, `TRIMRANGE(CHOOSE(1,Y1))`.

## Acceptance criteria
- [ ] A probe case covers the forms above; every scored cell passes, or each one that does not has an owner task.
- [ ] arrays.md's entry names TRIMRANGE with A11:D13, and its rule lists only the evidenced pick forms, with cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pickers other than IF, CHOOSE, IFS and SWITCH; OFFSET's counts.
