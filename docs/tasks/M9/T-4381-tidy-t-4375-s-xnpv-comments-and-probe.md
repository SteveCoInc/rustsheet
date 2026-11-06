---
id: T-4381
title: "Tidy T-4375's XNPV comments and probe a name for constant text"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4375]
---

## Goal
T-4375's XNPV code carries no stale comments, its generator no dead lines, and the two cases it left unprobed have
evidence.

## Context
- Follow-up of T-4375 (PR #3404; MINOR findings of the design and Excel-fidelity reviews, left out of the approved head).
- Stale comments in `Model::xnpv_walk_error` (crates/rustsheet-engine/src/functions/financial/flows.rs): the one that
  says every other index is an empty value and an empty later date "which pass" (an empty later date is `#NUM!` now,
  so name the first empty date as the third index the walk visits), and the arm comment that says a later 0 or a date
  before the first "is left to `xnpv_dates`" (the `first_day` guard reports them at their own index).
- `docs/research/t4375/gen_case.py` writes a placeholder column and pops it again (`ncol = column(12 + 5, ...)`,
  `data.pop("R1", None)`); keep `ncol = col(16)` only.
- Unprobed, in financial.md "XNPV reads its values and dates element by element (T-3815)": a defined name for a
  constant text (`="30"`) as XNPV's values or dates, and a literal text argument (`XNPV(0.1,"30",39448)`). The
  engine's `returns_one_cell_reference` treats every defined name as a cell. Probe both (excel-probe skill,
  excel-graph) and match Excel.

## Acceptance criteria
- [ ] The two comments and the generator are tidied, with no change in behavior.
- [ ] A probe case holds the constant-text name and the literal text argument, filled on excel-graph and passing;
      financial.md states the rule and drops the Unprobed note.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR (T-3816).
