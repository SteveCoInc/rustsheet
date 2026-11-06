---
id: T-4616
title: "Probe the accumulation order of a pivot Sum total across a column field"
milestone: M16
lane: research
status: todo
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

Build a 2-D pivot (rows R, columns K, several records per cell) whose candidate sums all differ (search with Python floats first). Also cover column totals across row items, and the grand-grand total. SPEC-0032 stage 3 (T-4142) depends on it.

## Acceptance criteria
- [ ] A filled probe separates the candidates; the pivots.md entry's UNVERIFIED part is closed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
