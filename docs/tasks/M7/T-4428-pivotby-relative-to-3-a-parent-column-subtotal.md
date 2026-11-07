---
id: T-4428
title: "PIVOTBY relative_to 3: a parent column subtotal no record reached holds every record of the line"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-4388]
---

## Goal
PIVOTBY's `relative_to` 3 parent set of a column item whose parent is a subtotal no record reached holds every
record of the row line, as that subtotal's own cell does under T-4388's rule (`Grouping::class_reached`).

## Context
- probes/t4420-pivotby-parent-sets-three-fields (excel-graph, 2026-11-06; T-4420): Calc!B3, B4, B7, B8, B10,
  B11, B12, C3, C4, C7, C8, C10, C11, C12 are off. In each, a level-3 leaf no record reached sits under a level-2
  subtotal no record reached. Excel gives that subtotal every record of the line, so the leaf's parent set is
  every record (C3: over BxC AyB BxC BxB CyB of p p q p p, the (`ab`&U+00AD,y,`ab`&U+00AD) cell of p is
  `1, 2, 4, 5 / 1, 2, 4, 5`). The engine reads the subtotal's reached-class records (none), so the cell is `#VALUE!`
  (PERCENTOF `#DIV/0!`). The subtotal's own cell is T-4401's.
- Fix, once T-4401 (PR #3448, merged; the one helper is `Grouping::class_reached`) is on main: in `Grid::column_records` (pivotby.rs) a
  `Some(Line::Subtotal { level, start, .. })` with `!self.cols.class_reached(field, class)` returns `None`
  (every record, `ColumnRecords`). T-4420 tried this on a scratch merge of T-4401's branch: the probe gives 528 of
  528, and probes/t4382-pivotby-intransitive-paths, t4402-pivotby-parent-sets-holdout and
  t4401-pivotby-level-subtotal-holdout stay all-pass.
- docs/excel-semantics/coercion.md "PIVOTBY's other paths on intransitive runs (T-4382)" records the cells.

## Acceptance criteria
- [ ] The 14 cells above pass; no other eval cell regresses (baseline updated).
- [ ] coercion.md says the rule is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4401's own subtotal cells.

## Outcome
`Grid::column_records` (pivotby.rs) now gives a column subtotal no record reached every record of the line,
as `cell_rows` does (T-4388). The 14 cells of probes/t4420-pivotby-parent-sets-three-fields pass (528 of 528);
`eval/baseline/probes.txt` shrank. coercion.md says the rule is implemented.
