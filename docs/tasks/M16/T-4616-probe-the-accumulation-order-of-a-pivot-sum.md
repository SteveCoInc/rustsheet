---
id: T-4616
title: "Probe the accumulation order of a pivot Sum total across a column field, StdDev/Var totals, and Count subtotals"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md states how a Sum total across a column field accumulates. In t4135, East's row total across Kind equals the sum of its cells (0.6000000000000001), while totals over one axis field accumulate item by item in display order. Average in the same pivot follows item order.

## Context
Evidence so far: probes/t4135-pivot-aggregates A_sum!S6 and S8; probes/t4614-pivot-total-order-2 S2, S3 (T-4614). Candidates:
- the sum of the row's cells, in display order or in some other order;
- item-by-item accumulation in another order than display order.

Build a 2-D pivot (rows R, columns K, several records per cell) whose candidate sums all differ (search with Python floats first). Also cover column totals across row items, and the grand-grand total. Ride-alongs:
- StdDev and Var totals' order;
- Product with a second separating cell (ascending value versus display order);
- Count/CountNums at inner-field subtotals;
- Count beside CountNums in a pivot with an error field (are they "mixed"?). SPEC-0032 stage 3 (T-4142) depends on it.

## Acceptance criteria
- [x] A filled probe separates the candidates; the pivots.md entry's UNVERIFIED part is closed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4616-pivot-total-order-3 (excel-graph, 2026-11-06), recorded in pivots.md "Summation order of
Sum, Average and Product totals" and "A Count total over an error field is the error when the pivot
mixes aggregates":
- **2-D Sum, Sum alone (S1, S4, S5, S6):** row and column totals are the child cells added in display
  item order. The grand-grand total adds the row totals in display order (S1 E8 25408.25; S4 E8
  25408.249999999993). In S1, every other candidate is refuted: cells in reverse or ascending order,
  items in any order, and record order or its reverse. This explains t4135 A_sum!S6.
- **2-D Sum beside Average (S2):** the same data gives item-by-item record accumulation in display order
  (S2 H8 871.3099999999998 against S1 E7 871.3100000000001). Average's totals follow that
  accumulation, alone (S3) or beside Sum.
- **Var/StdDev:** cells and totals use the textbook formula (Σx² − (Σx)²/n) / (n − 1), with the sums
  accumulated item by item in display order (V1–V4). StdDev is the square root of that Var.
- **Product:** P1 and P2 settle the display item order and refute ascending value order.
- **Count:** outer-field subtotals follow the mixed rule (I1, I2: East `#N/A`). Count beside CountNums
  is not mixed (M1–M3: counts everywhere).

Left open, T-4634:
- which mixes switch Sum from the cells to the records;
- whether a Sum-only total over one axis field uses the cells or the records;
- the mixed and Var 2-D grand-grand totals.
