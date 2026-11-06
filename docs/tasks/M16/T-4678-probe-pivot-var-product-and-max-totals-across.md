---
id: T-4678
title: "Probe pivot Var, Product and Max totals across two fields per axis, and a pivot with column fields only"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Settle the totals that T-4634 left UNVERIFIED in docs/excel-semantics/pivots.md ("Summation order of
Sum, Average and Product totals"), so that the engine's readings for them are evidenced or corrected.

## Context
T-4634 (probes/t4634-pivot-total-order-4) found that a cell that is a total on both axes adds the
partial sums of the leaf-row cells below it in its column. Its evidence is Sum and Average with two
fields per axis (T2, T3), and Var, Varp, StdDev, StdDevp, Product and Sum/Average with one field per
axis (V1 to V3, V6, V7, X1 to X12). The engine (`merge_crossings` in
crates/rustsheet-engine/src/pivot/result.rs) applies the same rule to every aggregate and axis shape.
Open:
- Var, Varp and Product at a subtotal-by-subtotal or subtotal-by-grand cell with two fields per axis
  (the engine adds the leaf-row partials, as T-4634 T2 and T3 show for Sum and Average);
- Max and Min beside an error field at such cells, and which of two different errors a total shows
  (the engine keeps the first in row-major record order; see also T-4633);
- a pivot with column fields only (no row field): whether a Sum-only total sums the item cells (T-4634
  O1 to O4 are row fields only) and whether Average and Var follow the records;
- Values on rows (`dataOnRows`) for the mixed and Sum-only readings.
Reuse T-4634's method: search the data with `python3 -I` so that each candidate gives a different
double, record the candidates, then probe.

## Acceptance criteria
- [ ] A filled probe separates the readings above, and the pivots.md UNVERIFIED part they cover is
  closed; any contradiction is fixed in result.rs or accumulate.rs with the sheets added to
  result/tests.rs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA's own aggregate (`layout::aggregate`, T-4154).
