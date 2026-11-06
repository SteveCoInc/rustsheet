---
id: T-4382
title: "PIVOTBY on intransitive runs: probe the subtotal error gate, three-field column subtotals, row-axis subtotals and PERCENTOF parents"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4378]
---

## Goal
The PIVOTBY paths on an intransitive run that T-4378's subtotal rule (a subtotal holds the records that reached its
class) has not been checked on are probed against Excel, and match it or carry an owner.

## Context
- From T-4378's fidelity review (PR #3403, MINOR findings). The rule is evidenced only for two column fields with a
  first-field subtotal: probes/t4358-pivotby-unreached-column-leaf column D and probes/t4378-pivotby-subtotal-holdout.
- `Grid::has_records` (functions/lookup_and_reference/pivotby.rs) still decides a subtotal's error gate for
  `check_totals` (one function over several value columns: an error total makes PIVOTBY `#VALUE!`, arrays.md T-2311)
  from the union of the leaf buckets. On an intransitive run that can now disagree with `cell_rows`: a subtotal whose
  records reached its class but no leaf, with every leaf bucket empty, or an unreached leaf that holds every record
  while the subtotal is empty.
- Also unprobed on such runs: a three-field column tree (subtotals at level 1 and 2), row-axis subtotals (the row
  grouping's own intransitive classes), and PERCENTOF's parent sets (`relative_of`, relative_to.rs, which still read
  `Grouping::group_of`).
- Texts as in docs/excel-semantics/coercion.md "GROUPBY on an intransitive run": A = ab, B = ab&U+00AD,
  C = U+00AD&ab. docs/research/t4358 and t4378 have generators and an engine-prediction script to reuse.

## Acceptance criteria
- [x] A probe with the engine's predictions committed before the fill covers each path above. Each cell passes, or
      the rule is recorded in coercion.md and its cells are owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- probes/t4382-pivotby-intransitive-paths (excel-graph, 2026-11-05): 42 seeded five-row runs of A = ab,
  B = ab&U+00AD, C = U+00AD&ab, 129 cells, generator, engine predictions (committed before the fill) and scorer in
  docs/research/t4382. The predictions match 63 of 129 cells.
- Findings, in docs/excel-semantics/coercion.md "PIVOTBY's other paths on intransitive runs (T-4382)":
  - the subtotal error gate (one function over two value columns) is the leaf buckets', as `Grid::has_records`
    decides: 12 of 15 gate cells pass, and the three misses are a placement defect (T-4403: Calc!B2, B5, B14);
  - row subtotals hold the records that reached the row class (its 16 cells pass since T-4359, #3409, which
    merged during this task), and a level-2 column subtotal whose class no record reached holds every record of
    the line (T-4401: 9 cells);
  - relative_to 3 and 4 take the parent subtotal's records by that rule, and an empty parent set is PERCENTOF
    `#DIV/0!` and a two-argument LAMBDA `#VALUE!` (T-4402: 30 cells; 8 more pass since T-4359).
  - docs/research/t4382/score.py gives 214 of 214 first-field subtotal and parent cells with the rule, chosen on
    them; T-4401 and T-4402 owe its hold-out.
- Eval delta: probes 129 new cells, 87 pass on main after T-4359; 42 new-coverage failures, 0 regressions, all
  owned by T-4401 (9), T-4402 (30) and T-4403 (3).
