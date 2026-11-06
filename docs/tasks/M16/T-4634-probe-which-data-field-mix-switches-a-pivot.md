---
id: T-4634
title: "Probe which data-field mix switches a pivot Sum total from child cells to records, and the order of a mixed 2-D grand total"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md ("Summation order of Sum, Average and Product totals") records that a 2-D Sum total is the
sum of the child cells when Sum is the pivot's only aggregate, and an item-by-item record
accumulation when Sum sits beside Average. Settle what switches it, and the open grand totals.

## Context
Evidence: probes/t4616-pivot-total-order-3 (T-4616): S1 E7 871.3100000000001 (cells) against S2 H8
871.3099999999998 (records, Sum beside Average); S2 H9 25408.25. Open questions:
- which mixes switch Sum to records: Sum beside Count, CountNums, Max, Min, Product, StdDev or Var.
  Sum beside another Sum of a different field is known: t4135 A_sum (six Sum fields) follows the
  cells, which the Sum-only rule covers;
- whether a Sum-only pivot over one axis field sums its item cells or accumulates records (the T-4614
  S3 data cannot tell them apart);
- the grand-grand total of a mixed 2-D pivot: S2 H9 fits, among others, the row totals added in
  any order, reverse-order row-item accumulation and several cell and record orders, but not record
  order or row-major accumulation in display order; one data
  point only;
- the grand-grand total of a 2-D Var pivot: V1 E11 fits textbook Var over row-major (R, then K)
  display-order accumulation and textbook Var over per-row or per-column partial sums, among others;
- whether Average's totals ever follow the cells (S3, Average alone, follows the records).
- whether Count or CountNums beside only Max, Min, Product or a variance counts as mixed (T-4142's
  `counts_mixed` treats any non-count aggregate as mixed);
- Sum-only cells that are totals on both axes other than the grand-grand total (subtotal by subtotal,
  subtotal by grand total, more than one field per axis);
- the order of Varp and StdDevp totals (T-4142 assumes Var's).

## Acceptance criteria
- [x] A filled probe separates the readings above, and the pivots.md entry's UNVERIFIED part is
  closed (what the probe could not reach moved to T-4678).
- [x] `make check-fast` green locally and `make eval-baseline` recorded; CI and reviewers run on the PR.

## Out of scope
- Engine changes.

## Outcome
probes/t4634-pivot-total-order-4 (excel-graph, 2026-11-06; 37 pivots), recorded in pivots.md
"Summation order of Sum, Average and Product totals" and "A Count total over an error field is the
error when the pivot mixes aggregates". The data were searched with `python3 -I` so that the
candidate readings give different doubles. Every claim below is among the candidates tried.
- **Mixes:** Sum beside Count, CountNums, Max, Min, Product, StdDev, Var, Varp, StdDevp or Average
  (X1 to X12, each over its own source) accumulates the records item by item in all six totals.
  Only Sum alone (S1) adds the cells.
- **Sum alone over one axis field:** adds the item cells in display order (O1 to O4; the only
  candidate that fits all four).
- **Mixed 2-D grand-grand total:** adds the row totals' record sums down the rows. It is the only one
  of 56 readings that fits all sixteen cells (X1 to X12 H9, X10 to X12 I9, A1 E8).
- **Var and StdDev grand-grand total:** textbook Var over the rows' partial sums. It is the only
  candidate that fits V1, V6 E11 and X6, X7 I9. Row-major order, the engine's earlier reading, is
  refuted: V1 E11 is 22664.877200872772, where row-major gives 22664.87720087405.
- **Varp, StdDevp and Product grand-grand totals:** the cells fit the same rule, but they also fit
  other readings. Those orders are taken by analogy (UNVERIFIED, T-4678).
- **Varp and StdDevp row and column totals** take Var's order, with /(n·n) and not division by n
  twice (V2, V3, V7; V4 and V5 over one field).
- **Average** never follows the cells (A1 to A3, X10 to X12, T3).
- **Count or CountNums beside only Max, Min, Product, Var, Varp or StdDev** is mixed (C1 to C7: East
  subtotal B5 and Grand Total B11 are `#N/A`).
- **Two fields per axis (T1 to T3):** with Sum alone, a total adds leaf cells, never subtotals (T1;
  adding the subtotals fails seven cells). Beside Average, and for Average alone, a cell that is a
  total on both axes adds the record sums of the leaf rows below it in its column (T2, T3, every
  cell). That is one rule with the Var and Product grand-grand totals.

Engine (asked for by the task owner, though "Out of scope" listed engine changes):
- result.rs `merge_crossings` rebuilds the sum, the squares and the product of each crossing from
  the leaf-row cells. It replaces `row_totals_sum`.
- `child_cells` makes Sum-only totals add leaf cells.
- accumulate.rs adds `Folds`, `clear_folds` and `add_folds`.
- result/tests.rs, layout/render/tests.rs and layout/lines/tests.rs check all 37 t4634 sheets.

The owner line is in T-4152. Left open, in T-4678:
- Var, StdDev, Varp, StdDevp and Product at crossings with two fields per axis;
- the Varp, StdDevp and Product grand-grand order;
- Max and Min beside errors;
- a pivot with column fields only;
- Values on rows.
