---
id: T-4414
title: "Probe PIVOTBY's column subtotal of a class another row line reaches but this line does not, on intransitive runs"
milestone: M7
lane: engine
status: done
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
- [x] A probe of such runs, with column subtotals ascending and with `col_sort_order` -1, is filled. The engine
      matches it, or the rule is revised to fit it, or the cells are owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- GROUPBY and PIVOTBY's row axis (probed by T-4359 and T-4388).

## Outcome
- **Probe:** probes/t4414-pivotby-mixed-column-subtotal (tag M7, excel-graph, 2026-11-06). It holds 30 seeded five-row
  three-field intransitive runs (seed "T-4414"; docs/research/t4414/gen_probe.py on T-4388's helpers). Their p/q row
  lines are chosen so that a class above the leaves is reached by a record of one line and by none of the other. All
  30 runs have such a class both ascending (Calc!B) and with `col_sort_order` -1 (Calc!C).
- **Predictions:** the engine's 60 values were committed before the fill (docs/research/t4414/predictions.tsv, with
  `docs/research/t4344/predict_engine.py`). All 60 match Excel (660/660 cells), so the engine needs no change. Such a
  column subtotal shows the other line nothing, as T-4378's prefix filter gives. T-4388's every-record rule is only
  for a class no record reached.
- **Docs:** coercion.md, at the T-4388 paragraph of the intransitive-runs entry, now states the mixed case with
  Calc!B1 and C1.
- **Eval delta:** +1 probe case, 660/660 passing (eval/baseline/probes.txt); no other cell changed.
