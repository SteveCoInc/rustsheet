---
id: T-4614
title: "Probe the accumulation order of pivot Sum and Average totals, and when a Count grand total shows an error"
milestone: M16
lane: research
status: todo
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
- [ ] Filled probes separate the candidates; the two pivots.md entries' UNVERIFIED parts are closed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
