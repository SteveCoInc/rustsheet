---
id: T-4414
title: "Probe PIVOTBY's column subtotal of a class another row line reaches but this line does not, on intransitive runs"
milestone: M7
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-4388]
---

## Goal
PIVOTBY's column subtotals match Excel on an intransitive run where a class above the leaves is reached by
records of another row line but by no record of this one.

## Context
- T-4388 (probes/t4388-pivotby-intransitive-subtotals columns D and E): a column subtotal of a class that no record
  reached shows every record of its row line, as the class's leaves do (`Grouping::class_reached`, the
  `Slot::Subtotal` arm of `Grid::cell_rows` in pivotby.rs). The rule was read from 15 cells, all of the same shape:
  no record anywhere reached the class.
- The mixed case was not probed. There, records of another row line reach the class but none of this line's do. The
  engine keeps T-4378's prefix filter for it and shows an empty cell, which fits the transitive case. The
  excel-fidelity review of T-4388 (PR #3430) asked for this probe.
- Generate seeded runs with docs/research/t4388/gen_probe.py's helpers: three fields, p/q row lines, picked so that a
  reached class above the leaves has no record of one row line. Commit the engine's predictions before the fill with
  `python3 -I docs/research/t4344/predict_engine.py <case> <out>` (decision 0188 R4).

## Acceptance criteria
- [ ] A probe of such runs, with column subtotals ascending and with `col_sort_order` -1, is filled. The engine
      matches it, or the rule is revised to fit it, or the cells are owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY and PIVOTBY's row axis (probed by T-4359 and T-4388).
