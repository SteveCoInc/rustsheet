---
id: T-4381
title: "Tidy T-4375's XNPV comments and probe a name for constant text"
milestone: M9
lane: functions-financial
status: done
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
- [x] The two comments and the generator are tidied, with no change in behavior.
- [x] A probe case holds the constant-text name and the literal text argument, filled on excel-graph and passing;
      financial.md states the rule and drops the Unprobed note.
- [x] `make check` green; reviewers approve.

## Out of scope
- XIRR (T-3816).

## Outcome
Tidied the two stale `xnpv_walk_error` comments and dropped the dead placeholder lines of
docs/research/t4375/gen_case.py (no behavior change). Probed a text literal and a defined name for a constant text
as XNPV's values and dates (probes/t4381-xnpv-literal-and-named-text, excel-graph): numeric and date text is read
as its number in both forms, non-numeric text is `#VALUE!`. The name rows already passed (the engine reads a name
as a cell); a text literal now reads the same way (`reads_text_as_number`, `Node::StringKind`), fixing A1:A3.
financial.md states the rule and drops the Unprobed note. Computed text (`"3"&"0"`) stays unprobed. Eval: the new
case 10/10, no regressions.
