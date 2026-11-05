---
id: T-4388
title: "PIVOTBY's subtotals of a class no record reached and its row-axis parents on an intransitive run"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4359, T-4378]
---

## Goal
PIVOTBY's subtotals match Excel on an intransitive run where GROUPBY's rule is known but PIVOTBY's is unprobed: a
column subtotal of a class no record of the row group reached, and the row axis's subtotals and value-sort parents.

## Context
- T-4359 (probes/t4359-groupby-value-sort-parents, probes/t4359-groupby-value-sort-holdout): a GROUPBY parent
  holds the rows that reached its class, and every row when none did (a repeated class whose leaves no row
  reaches), for its subtotal line and its value-sort order (`Grouping::class_rows`, grouping.rs). PIVOTBY's row
  axis shares that code (`Grouping::line_parts`, `sort_by_aggregate`), unprobed on intransitive runs.
- T-4378: a PIVOTBY column subtotal holds the row group's records whose classes start with its prefix (the
  `Slot::Subtotal` arm of `Grid::cell_rows`, pivotby.rs), so a class no record reached gets none, where GROUPBY's
  gets every row and a PIVOTBY column leaf no record reaches gets every record of the row group (T-4358).
- Probe both (seeded runs as docs/research/t4359/gen_probe.py, with A = ab, B = ab&U+00AD, C = U+00AD&ab), with
  predictions committed before the fill (decision 0188 R4), then implement what differs.

## Acceptance criteria
- [ ] A probe of PIVOTBY row-axis subtotals and value sorts, and column subtotals over a class no record reached,
      on intransitive runs, filled; the engine matches it or the cells are owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
