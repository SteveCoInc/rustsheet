---
id: T-4614
title: "Probe the accumulation order of pivot Sum and Average totals, and when a Count grand total shows an error"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md states:
- the record order in which Sum totals and Average totals are accumulated;
- the condition under which a Count grand total over an error-holding field shows the error.

## Context
T-4135 and T-4589 (probes/t4135-pivot-aggregates A_sum, A_average; probes/t4589-pivot-total-order S1, E1-E4) leave these open:
- Sum totals: reverse record order versus ascending item order (column field).
- Average: confirm first-seen item order, the only reading left (descending value is refuted by t4135 A_average!S8).
- Product: record order versus first-seen item order.
- Count grand totals: mixed aggregates versus several data fields with no column field. The separating pair is all-Count with no column field, and mixed aggregates with a column field. Also CountNums without showError.

Use data where the candidates differ. For example, put items in descending alphabetical first-seen order (Z before A) with values where reverse record order, item-ascending and first-seen sums are pairwise distinct; search with Python floats before building. For Count: two Count fields over Err and Num (all Count, two source fields); Count beside Average; Count beside CountNums. The engine's accumulator (SPEC-0032 stage 3, T-4142) depends on it.

## Acceptance criteria
- [x] Filled probes separate the candidates; the two pivots.md entries' UNVERIFIED parts are closed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4614-pivot-total-order-2 (excel-graph, 2026-11-06) holds seven builder-v2 pivots. Recorded in
docs/excel-semantics/pivots.md:
- **Totals over one axis field (S2, S3, P2):** Sum, Average and Product accumulate the records item by
  item in display (first-seen) item order, in record order within each item. S2's Sum is 29.6; record,
  reverse and ascending item order give three other values, all refuted. This also fits T-4589 S1.
- **Sum across a column field:** t4135's East total equals the sum of the row's cells. Item order is
  refuted there, so the 2-D case is T-4616's.
- **Count totals (C1–C4):** mixed aggregates is the condition. All-Count without a column field stays a
  count (C1). Mixed aggregates with a column field errs (C2): East's total, B's grand total and the
  grand total all show `#N/A`, while West's total stays 1. CountNums (C3) and Count beside Average (C4)
  err too. The "no column field" reading is refuted.
