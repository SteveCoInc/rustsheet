---
id: T-4634
title: "Probe which data-field mix switches a pivot Sum total from child cells to records, and the order of a mixed 2-D grand total"
milestone: M16
lane: research
status: todo
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
- [ ] A filled probe separates the readings above, and the pivots.md entry's UNVERIFIED part is
  closed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
