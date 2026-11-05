---
id: T-4382
title: "PIVOTBY on intransitive runs: probe the subtotal error gate, three-field column subtotals, row-axis subtotals and PERCENTOF parents"
milestone: M7
lane: engine
status: todo
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
- [ ] A probe with the engine's predictions committed before the fill covers each path above. Each cell passes, or
      the rule is recorded in coercion.md and its cells are owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
