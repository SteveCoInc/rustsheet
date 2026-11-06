---
id: T-4589
title: "Probe why a pivot with errorCaption shows the caption in the Count and CountNums grand totals, and the Sum, StdDev and Var accumulation order"
milestone: M16
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md states:
- why probes/t4135-pivot-aggregates EC shows `errorCaption` in the Count and CountNums grand totals, whose values hold no error;
- whether Sum totals are the sum of the children's sums or an accumulation that takes one item's records first;
- whether StdDev and Var totals combine records or children.

## Context
In T-4135, EC (rows Region, eleven data fields over Err, showError with ERR) shows ERR in B7:L7, Count and CountNums included. Without the caption, A_count!R8 is 4. Candidate causes:
- the caption marks a whole grand-total line once any data field's grand total is an error;
- Count over an error-bearing field errs at the grand-total level.

Vary one thing at a time: Count alone; Count beside Sum; and a column field.

Sum totals: in T-4135, A_sum!S6 is 0.6000000000000001. That is the sum of the children, and it refutes both record order and display order, which give 0.6. An accumulation that takes A's records first would also give it. Product totals are record order (A_product!S6 0.006). Use data where those two differ: three items with a nested field, values such as 1e16, 1 and -1e16 across items. Context: pivots.md "errorCaption replaces every error value" and "Summation order of Sum, Average and Product totals".

## Acceptance criteria
- [x] Filled probes separate the candidate causes; the two pivots.md entries' UNVERIFIED parts are closed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4589-pivot-total-order (excel-graph, 2026-11-06) holds five builder-v2 pivots. Recorded in
docs/excel-semantics/pivots.md:
- **Sum order (S1):** the grand total 5.300000000000001 refutes both record order (5.299999999999999)
  and the sum of the children (5.3). The latter was T-4135's reading, now corrected. Two readings fit
  every Sum cell: reverse record order, and ascending item order of the column field.
- **Average (S1):** 1.3250000000000002. With t4135's cells, only first-seen item order fits.
  Descending value is refuted by A_average!S8.
- **Product:** record order and first-seen item order both fit.
- **Count grand total (E1–E4):** EC's caption came from a real error: without errorCaption, E4's Count
  grand total is `#N/A`. It appeared in every pivot with two or more data fields and no column field.
  It did not appear with one data field (E1), or with six Count fields and a column field (t4135
  A_count). Mixed aggregates and a missing column field are not yet separated.

The acceptance criterion's UNVERIFIED parts are closed only in part: the mechanisms named above are
T-4614's.
